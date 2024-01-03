import { invoke } from "@tauri-apps/api";
import { reactive, toRaw, DeepReadonly } from "vue";
import settings from "./settings";

export interface Reply {
    current: number;
    maximum: number;
    source: "ddcci" | "wmi";
}

export interface Feature {
    name: string;
    value: Reply;
}

export interface Monitor {
    id: string;
    name: string | null;
    features: Feature[];
}

function timeout(millis: number): Promise<undefined> {
    return new Promise((resolve) => {
        setTimeout(resolve, millis);
    });
}

export class Manager {
    readonly monitors: DeepReadonly<Monitor[]> = reactive([]);
    private refreshing = false;

    private async doRefresh(): Promise<void> {
        const monitors = this.monitors as Monitor[];
        await invoke("refresh_monitors");
        const monitorIds = await invoke<string[]>("get_monitors");
        const monitorMap = new Map(monitors.map((monitor) => [monitor.id, monitor]));
        Object.assign(
            monitors,
            monitorIds.map(
                (id) =>
                    monitorMap.get(id) ?? {
                        id,
                        name: null,
                        features: [],
                    },
            ),
        );
        const pool: Promise<void>[] = [];
        for (const monitor of monitors) {
            if (monitor.name == null) {
                pool.push(
                    (async () => {
                        try {
                            monitor.name = await invoke<string | null>(
                                "get_monitor_user_friendly_name",
                                { id: monitor.id },
                            );
                        } catch {}
                    })(),
                );
            }
        }
        for (const monitor of monitors) {
            pool.push(
                (async () => {
                    const featureNames = monitor.features.length
                        ? monitor.features.map((feature) => feature.name)
                        : ["luminance", "contrast", "brightness", "volume", "powerstate"];
                    let first = true;
                    for (const name of featureNames) {
                        if (!first) {
                            await timeout(settings.updateInterval);
                        }
                        let value: Reply | undefined;
                        try {
                            value = await invoke<Reply>("get_monitor_feature", {
                                id: monitor.id,
                                feature: name,
                            });
                        } catch {}
                        const idx = monitor.features.findIndex((feature) => feature.name == name);
                        if (value) {
                            const item = monitor.features[idx];
                            if (item) {
                                item.value = value;
                            } else {
                                monitor.features.push({
                                    name,
                                    value,
                                });
                            }
                        } else if (idx != -1) {
                            monitor.features.splice(idx, 1);
                        }
                        first = false;
                    }
                })(),
            );
        }
        await Promise.allSettled(pool);
    }

    async refresh(): Promise<void> {
        if (!this.refreshing) {
            this.refreshing = true;
            try {
                await this.doRefresh();
            } finally {
                this.refreshing = false;
            }
        }
    }

    getMonitor(id: string): DeepReadonly<Monitor> {
        const idx = toRaw(this.monitors).findIndex((monitor) => monitor.id == id);
        if (idx == -1) {
            throw new Error(`no such monitor: '${id}'`);
        }
        return this.monitors[idx];
    }

    getFeature(id: string, name: string): DeepReadonly<Feature> {
        const monitor = this.getMonitor(id);
        const idx = toRaw(monitor.features).findIndex((f) => f.name == name);
        if (idx == -1) {
            throw new Error(`monitor '${id}' has no such feature: '${name}'`);
        }
        return monitor.features[idx];
    }

    async setFeature(id: string, name: string, value: number): Promise<void> {
        const feature = this.getFeature(id, name) as Feature;
        if (feature.value.current != value) {
            await invoke<void>("set_monitor_feature", {
                id,
                feature: name,
                value,
            });
            await timeout(settings.updateInterval);
            feature.value = await invoke<Reply>("get_monitor_feature", {
                id,
                feature: name,
            });
        }
    }
}

const manager = new Manager();

export default manager;
