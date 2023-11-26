import { createApp } from "vue"
import monitorManager from "./monitor"
import Panel from "./components/Panel.vue"
import "./style.global.sass"

createApp(Panel).mount("#root")

monitorManager.refreshMonitors()

if (import.meta.env.PROD) {
    document.addEventListener("contextmenu", e => e.preventDefault())
}
