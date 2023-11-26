<script setup lang="ts">
import { invoke } from "@tauri-apps/api/tauri"
import { computed, ref } from "vue"
import { watchDelayed } from "../util"
import panelState from "../wm.js"

interface Color {
    r: number
    g: number
    b: number
}

interface AccentColors {
    accent: Color
    accentDark1: Color
    accentDark2: Color
    accentDark3: Color
    accentLight1: Color
    accentLight2: Color
    accentLight3: Color
    background: Color
    foreground: Color
}

const accents = ref<AccentColors>()

watchDelayed(
    () => panelState.focused,
    async _focused => {
        accents.value = await invoke<AccentColors>("get_accent_colors")
    },
    { delay: 1000, immediate: true, leading: true },
)

const colors = computed(() => {
    const colors: { [key: string]: string } = {}
    if (accents.value) {
        for (const [name, { r, g, b }] of Object.entries(accents.value)) {
            colors[name] = `rgb(${r},${g},${b})`
        }
    }
    return colors
})
</script>

<template>
    <div class="style-root">
        <slot />
    </div>
</template>

<style scoped lang="sass">
.style-root
    --accent: v-bind(colors.accent)
    --accent-dark1: v-bind(colors.accentDark1)
    --accent-dark2: v-bind(colors.accentDark2)
    --accent-dark3: v-bind(colors.accentDark3)
    --accent-light1: v-bind(colors.accentLight1)
    --accent-light2: v-bind(colors.accentLight2)
    --accent-light3: v-bind(colors.accentLight3)
    --background: v-bind(colors.background)
    --foreground: v-bind(colors.foreground)
</style>
