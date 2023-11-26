import { invoke } from "@tauri-apps/api"
import { action, autorun, flow, observable, runInAction } from "mobx"
import settings from "./settings"

export interface Reply {
    current: number
    maximum: number
}

export interface Feature {
    name: string
    value: Reply
    syncing: boolean
}

export interface Monitor {
    id: string
    name: string | null
    features: Feature[]
}

function timeout(millis: number): Promise<undefined> {
    return new Promise(resolve => {
        setTimeout(resolve, millis)
    })
}

async function getFeatures(
    id: string,
    featureNames: string[],
    silent?: boolean,
): Promise<Feature[]> {
    const featureTestTime = 200
    const features: Feature[] = []
    for (let i = 0; i < featureNames.length; ++i) {
        if (i) {
            await timeout(featureTestTime)
        }
        const name = featureNames[i]
        let value: Reply
        if (silent) {
            try {
                value = await invoke<Reply>("get_monitor_feature", {
                    id,
                    feature: name,
                })
            } catch {
                continue
            }
        } else {
            value = await invoke<Reply>("get_monitor_feature", {
                id,
                feature: name,
            })
        }
        features.push({ name, value, syncing: false })
    }
    return features
}

export class Manager {
    @observable accessor monitors: Monitor[]

    constructor() {
        this.monitors = []

        autorun(
            () => {
                for (const monitor of this.monitors) {
                    for (const feature of monitor.features) {
                        if (feature.syncing) {
                            runInAction(() => {
                                feature.syncing = false
                            })
                            invoke("set_monitor_feature", {
                                id: monitor.id,
                                feature: feature.name,
                                value: feature.value.current,
                            }).catch(() => {
                                this.refreshFeature(monitor.id, feature.name)
                            })
                        }
                    }
                }
            },
            {
                delay: settings.updateInterval,
            },
        )
    }

    @flow
    *refreshMonitors(): Generator<Promise<any>, void, any> {
        yield invoke("refresh_monitors")
        const monitor_ids: string[] = yield invoke("get_monitors")
        const monitor_names: (string | null)[] = yield Promise.all(
            monitor_ids.map(id => {
                try {
                    return invoke("get_monitor_user_friendly_name", { id })
                } catch {
                    return null
                }
            }),
        )
        const monitor_features: Feature[][] = yield Promise.all(
            monitor_ids.map(id =>
                getFeatures(
                    id,
                    [
                        "luminance",
                        "contrast",
                        "brightness",
                        "volume",
                        "powerstate",
                    ],
                    true,
                ),
            ),
        )
        this.monitors.splice(0)
        for (let i = 0; i < monitor_ids.length; ++i) {
            this.monitors.push({
                id: monitor_ids[i],
                name: monitor_names[i],
                features: monitor_features[i],
            })
        }
    }

    getMonitor(id: string): Monitor {
        const monitor = this.monitors.find(monitor => monitor.id == id)
        if (monitor) {
            return monitor
        }
        throw new Error(`no such monitor: '${id}'`)
    }

    @flow
    *refreshFeature(
        id: string,
        name: string,
    ): Generator<Promise<any>, void, any> {
        const monitor = this.getMonitor(id)
        const features: Feature[] = yield getFeatures(id, [name])
        const feature = features[0]
        const idx = monitor.features.findIndex(f => f.name == feature.name)
        if (idx == -1) {
            monitor.features.push(feature)
        } else {
            monitor.features[idx] = feature
        }
    }

    getFeature(id: string, name: string): Feature {
        const monitor = this.getMonitor(id)
        const feature = monitor.features.find(f => f.name == name)
        if (feature) {
            return feature
        }
        throw new Error(`monitor '${id}' has no such feature: '${name}'`)
    }

    @action
    setFeature(id: string, name: string, value: number) {
        const feature = this.getFeature(id, name)
        feature.value.current = value
        feature.syncing = true
    }
}

export default new Manager()
