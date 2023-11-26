<script lang="ts">
import { defineComponent } from "vue"
import monitorManager from "../monitor"
import sheet from "../style.module.sass"

const iconMap = {
    luminance: "\uE706",
    contrast: "\uE793",
    brightness: "\uE7E8",
    volume: "\uE767",
    powerstate: "\uE7E8",
}

export default defineComponent({
    props: {
        monitorId: {
            type: String,
            required: true,
        },
        feature: {
            type: String,
            required: true,
        },
    },
    setup() {
        return {
            sheet,
        }
    },
    computed: {
        featureValue() {
            return monitorManager.getFeature(this.monitorId, this.feature).value
        },
        current() {
            return this.featureValue.current
        },
        maximum() {
            return this.featureValue.maximum
        },
        icon() {
            return (iconMap as { [key: string]: string })[this.feature]
        },
    },
    methods: {
        handleInput(event: Event) {
            const e = event as InputEvent
            const target = e.target! as HTMLInputElement
            if (target.validity.valid) {
                const value = Number(target.value)
                monitorManager.setFeature(this.monitorId, this.feature, value)
            }
        },
        handleWheel(event: Event) {
            const e = event as WheelEvent
            const target = e.currentTarget! as HTMLInputElement
            if (e.deltaMode == WheelEvent.DOM_DELTA_PIXEL) {
                const offset =
                    Math.abs(e.deltaX) > Math.abs(e.deltaY)
                        ? e.deltaX
                        : -e.deltaY
                const current = Number(target.value)
                monitorManager.setFeature(
                    this.monitorId,
                    this.feature,
                    Math.max(
                        0,
                        Math.min(
                            this.maximum,
                            Math.round(current + offset * 0.01),
                        ),
                    ),
                )
            }
        },
    },
})
</script>

<template>
    <label :class="[sheet.flex, sheet.cozyLine]">
        <span :class="sheet.bigIcon" :aria-label="feature">
            {{ icon }}
        </span>
        <input
            type="range"
            step="1"
            min="0"
            :max="maximum"
            :value="current"
            @input="handleInput"
            @wheel="handleWheel"
            :class="[sheet.grow, sheet.slider]"
            :style="`--slider-value: ${(current / maximum) * 100}%`"
        />
        <input
            type="number"
            step="1"
            min="0"
            :max="maximum"
            :value="current"
            @input="handleInput"
            @wheel.prevent="handleWheel"
            role="status"
            :class="[sheet.borderlessNumber, sheet.titleFont]"
            style="width: 1.7em; text-align: center; margin-inline-start: 0.5em"
        />
    </label>
</template>
