import { createApp } from "vue"
import monitorManager from "./monitor"
import BrightnessPanel from "./components/BrightnessPanel.vue"
import "./style.global.sass"

createApp(BrightnessPanel).mount("#root")

monitorManager.refreshMonitors()

if (import.meta.env.PROD) {
    document.addEventListener("contextmenu", e => e.preventDefault())
}
