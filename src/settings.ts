import { invoke } from "@tauri-apps/api/tauri"
import { observable } from "mobx"

export interface Settings {
    updateInterval: number
}

export default observable<Settings>({
    updateInterval: 500,
})
