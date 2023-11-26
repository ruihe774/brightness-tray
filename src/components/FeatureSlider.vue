<script setup lang="ts">
import { css } from "@emotion/css"
import { computed } from "vue"
import { sheet, makeSliderStyle } from "../style"
import monitorManager from "../monitor"

const { monitorId, feature } = defineProps<{
    monitorId: string
    feature: string
}>()

const featureRef = computed(
    () => monitorManager.getFeature(monitorId, feature).value,
)

function handleInput(event: Event) {
    const e = event as InputEvent
    const target = e.target! as HTMLInputElement
    if (target.validity.valid) {
        const value = Number(target.value)
        monitorManager.setFeature(monitorId, feature, value)
    }
}

function handleWheel(event: Event) {
    const e = event as WheelEvent
    const target = e.currentTarget! as HTMLInputElement
    if (e.deltaMode == WheelEvent.DOM_DELTA_PIXEL) {
        const offset =
            Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : -e.deltaY
        const current = Number(target.value)
        monitorManager.setFeature(
            monitorId,
            feature,
            Math.max(
                0,
                Math.min(
                    featureRef.value.maximum,
                    Math.round(current + offset * 0.01),
                ),
            ),
        )
    }
}

const iconMap: { [key: string]: string } = {
    luminance: "\uE706",
    contrast: "\uE793",
    brightness: "\uE7E8",
    volume: "\uE767",
    powerstate: "\uE7E8",
}
</script>

<template>
    <label :class="[sheet.flex, sheet.cozyLine]">
        <span :class="sheet.bigIcon" :aria-label="feature">
            {{ iconMap[feature] }}
        </span>
        <input
            type="range"
            step="1"
            min="0"
            :max="featureRef.maximum"
            :value="featureRef.current"
            @input="handleInput"
            @wheel="handleWheel"
            :class="[sheet.grow, makeSliderStyle(featureRef.current)]"
        />
        <input
            type="number"
            step="1"
            min="0"
            :max="featureRef.maximum"
            :value="featureRef.current"
            @input="handleInput"
            @wheel.prevent="handleWheel"
            role="status"
            :class="[
                sheet.borderlessNumber,
                sheet.titleFont,
                css`
                    width: 1.7em;
                    text-align: center;
                    margin-inline-start: 0.5em;
                `,
            ]"
        />
    </label>
</template>
