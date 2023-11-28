import { createApp } from "vue";
import monitorManager from "./monitor";
import BrightnessPanel from "./components/BrightnessPanel.vue";
import "./wm";
import "./style.global.sass";

createApp(BrightnessPanel).mount("#root");

monitorManager.refreshMonitors();

if (import.meta.env.PROD) {
    document.addEventListener("contextmenu", (e) => e.preventDefault());
    document.addEventListener("keydown", (e) => {
        if (!e.metaKey && !e.altKey && (e.ctrlKey ? e.key == "r" : e.key == "F5")) {
            e.preventDefault();
            monitorManager.refreshMonitors();
        }
    });
}
