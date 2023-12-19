import { invoke } from "@tauri-apps/api";
import { reactive, watch, DeepReadonly } from "vue";
import { watchThrottled } from "./watchers";
import settings from "./settings";

export interface Reply {
    current: number;
    maximum: number;
}

export interface Feature {
    name: string;
    value: Reply;
    syncing: boolean;
    readonly: boolean;
    written: boolean;
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

const featureTestTime = 200;

export class Manager {
    readonly monitors: DeepReadonly<Monitor[]> = reactive([]);
    private refreshing: boolean = false;

    private async doRefresh(): Promise<void> {
        const monitors = this.monitors as Monitor[];
        await invoke("refresh_monitors");
        const monitorIds = await invoke<string[]>("get_monitors");
        const monitorMap = new Map(monitors.map((monitor) => [monitor.id, monitor]));
        monitors.splice(0);
        for (const id of monitorIds) {
            monitors.push(
                monitorMap.get(id) ?? {
                    id,
                    name: null,
                    features: [],
                },
            );
        }
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
                            await timeout(featureTestTime);
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
                                if (!item.syncing) {
                                    item.value = value;
                                }
                            } else {
                                monitor.features.push({
                                    name,
                                    value,
                                    syncing: false,
                                    readonly: false,
                                    written: false,
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
        const monitor = this.monitors.find((monitor) => monitor.id == id);
        if (monitor) {
            return monitor;
        }
        throw new Error(`no such monitor: '${id}'`);
    }

    getFeature(id: string, name: string): DeepReadonly<Feature> {
        const monitor = this.getMonitor(id);
        const feature = monitor.features.find((f) => f.name == name);
        if (feature) {
            return feature;
        }
        throw new Error(`monitor '${id}' has no such feature: '${name}'`);
    }

    updateFeature(id: string, name: string, value: number): void {
        const feature = this.getFeature(id, name) as Feature;
        if (feature.value.current != value) {
            feature.value.current = value;
            feature.syncing = true;
        }
    }
}

const monitorManager = new Manager();

watch(
    () => settings.updateInterval,
    (updateInterval, _old, onCleanup) => {
        onCleanup(
            watchThrottled(
                monitorManager.monitors as Monitor[],
                (monitors) => {
                    for (const monitor of monitors) {
                        for (const feature of monitor.features) {
                            if (feature.syncing) {
                                (async () => {
                                    try {
                                        feature.syncing = false;
                                        if (!feature.written) {
                                            feature.readonly = true;
                                        }
                                        await invoke("set_monitor_feature", {
                                            id: monitor.id,
                                            feature: feature.name,
                                            value: feature.value.current,
                                        });
                                        if (!feature.written) {
                                            await timeout(featureTestTime);
                                            const value = await invoke<Reply>(
                                                "get_monitor_feature",
                                                {
                                                    id: monitor.id,
                                                    feature: feature.name,
                                                },
                                            );
                                            if (value.current == feature.value.current) {
                                                feature.written = true;
                                                feature.readonly = false;
                                            } else {
                                                feature.value = value;
                                            }
                                        }
                                    } catch (e) {
                                        console.error(e);
                                        feature.readonly = true;
                                        monitorManager.refresh();
                                    }
                                })();
                            }
                        }
                    }
                },
                { throttle: updateInterval, trailing: true },
            ),
        );
    },
    { immediate: true },
);

export default monitorManager;
