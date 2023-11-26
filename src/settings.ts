import { observable } from "mobx"

export interface Settings {
    updateInterval: number
    ddcPowerOffValue: number
}

export default observable<Settings>({
    updateInterval: 500,
    ddcPowerOffValue: 6,
})
