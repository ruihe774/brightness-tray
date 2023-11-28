<script setup lang="ts">
import { invoke } from "@tauri-apps/api";
import { computed, reactive } from "vue";
import { watchDelayed } from "../util";
import panelState from "../wm.js";

interface Color {
    r: number;
    g: number;
    b: number;
}
type Colors = { [key: string]: Color };

const colors = reactive<Colors>({});

watchDelayed(
    () => panelState.focused,
    async () => {
        Object.assign(colors, await invoke<Colors>("get_accent_colors"));
    },
    { delay: 1000, immediate: true, leading: true },
);

const style = computed(() => {
    const style: string[] = [];
    for (const [name, { r, g, b }] of Object.entries(colors)) {
        style.push(
            `--${name.replace(/([a-z])([A-Z])/g, "$1-$2").toLowerCase()}:rgb(${r},${g},${b})`,
        );
    }
    return style.join(";");
});
</script>

<template>
    <div :style="style">
        <slot />
    </div>
</template>
