import { invoke } from "@tauri-apps/api/tauri"
import { observable } from "mobx"

interface OsVersion {
    major: number
    minor: number
    pack: number
    build: number
}

export interface Settings {
    updateInterval: number
    isWin11: boolean
}

export default observable<Settings>({
    updateInterval: 500,
    isWin11: (await invoke<OsVersion>("windows_version")).build >= 22000,
})
