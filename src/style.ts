import { invoke } from "@tauri-apps/api";
import { watchEffect, reactive } from "vue";
import { kebabCase } from "lodash-es";
import { watchThrottled } from "./watchers";
import panelState from "./wm";

interface Color {
    r: number;
    g: number;
    b: number;
}
type Colors = { [key: string]: Color };

const colors = reactive<Colors>({});

watchThrottled(
    () => panelState.focused,
    async () => {
        for (const [name, color] of Object.entries(await invoke<Colors>("get_accent_colors"))) {
            colors[name] = Object.assign(colors[name] ?? {}, color);
        }
    },
    { throttle: 1000, immediate: true },
);

const html: HTMLHtmlElement = document.querySelector(":root")!;

watchEffect(() => {
    for (const [name, { r, g, b }] of Object.entries(colors)) {
        html.style.setProperty(`--${kebabCase(name)}`, `rgb(${r},${g},${b})`);
    }
});
