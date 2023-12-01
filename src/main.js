import { createApp } from "vue";
import { watchThrottled } from "./watchers";
import monitorManager from "./monitor";
import BrightnessPanel from "./components/BrightnessPanel.vue";
import panelState from "./wm";
import "./style";
import "./style.global.sass";

createApp(BrightnessPanel).mount("#root");

watchThrottled(
    () => panelState.focused,
    () => {
        monitorManager.refresh();
    },
    { throttle: 10000, immediate: true },
);

if (import.meta.env.PROD) {
    document.addEventListener("contextmenu", (e) => e.preventDefault());
    document.addEventListener("keydown", (e) => {
        if (!e.metaKey && !e.altKey && (e.ctrlKey ? e.key == "r" : e.key == "F5")) {
            e.preventDefault();
            monitorManager.refreshMonitors();
        }
    });
}
