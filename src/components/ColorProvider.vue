<script setup lang="ts">
import { invoke } from "@tauri-apps/api/tauri"
import { reactive } from "vue"
import { watchDelayed } from "../util"
import panelState from "../wm.js"

interface Color {
    r: number
    g: number
    b: number
}
type Colors = { [key: string]: Color | undefined }

const colors = reactive<Colors>({})

watchDelayed(
    () => panelState.focused,
    async () => {
        Object.assign(colors, await invoke<Colors>("get_accent_colors"))
    },
    { delay: 1000, immediate: true, leading: true },
)

function c2s(color: Color | undefined): string {
    try {
        const { r, g, b } = color!
        return `rgb(${r},${g},${b})`
    } catch {
        return "initial"
    }
}
</script>

<template>
    <div class="style-root">
        <slot />
    </div>
</template>

<style scoped lang="sass">
.style-root
    --accent: v-bind(c2s(colors.accent))
    --accent-dark1: v-bind(c2s(colors.accentDark1))
    --accent-dark2: v-bind(c2s(colors.accentDark2))
    --accent-dark3: v-bind(c2s(colors.accentDark3))
    --accent-light1: v-bind(c2s(colors.accentLight1))
    --accent-light2: v-bind(c2s(colors.accentLight2))
    --accent-light3: v-bind(c2s(colors.accentLight3))
    --background: v-bind(c2s(colors.background))
    --foreground: v-bind(c2s(colors.foreground))
</style>
