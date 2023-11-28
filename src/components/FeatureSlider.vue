<script lang="ts">
import { defineComponent } from "vue";
import monitorManager from "../monitor";
import sheet from "../style.module.sass";

const iconMap = {
    luminance: "\uE706",
    contrast: "\uE793",
    brightness: "\uE7E8",
    volume: "\uE767",
    powerstate: "\uE7E8",
};

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
        };
    },
    computed: {
        featureObject() {
            return monitorManager.getFeature(this.monitorId, this.feature);
        },
        readonly() {
            return this.featureObject.readonly;
        },
        featureValue() {
            return this.featureObject.value;
        },
        current() {
            return this.featureValue.current;
        },
        maximum() {
            return this.featureValue.maximum;
        },
        icon() {
            return (iconMap as { [key: string]: string })[this.feature];
        },
    },
    methods: {
        handleInput(event: Event) {
            const e = event as InputEvent;
            const target = e.target! as HTMLInputElement;
            if (target.validity.valid) {
                const value = Number(target.value);
                monitorManager.updateFeature(this.monitorId, this.feature, value);
            }
        },
        handleWheel(event: Event) {
            const e = event as WheelEvent;
            const target = e.currentTarget! as HTMLInputElement;
            if (e.deltaMode == WheelEvent.DOM_DELTA_PIXEL) {
                const offset = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : -e.deltaY;
                const current = Number(target.value);
                monitorManager.updateFeature(
                    this.monitorId,
                    this.feature,
                    Math.max(0, Math.min(this.maximum, Math.round(current + offset * 0.01))),
                );
            }
        },
    },
});
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
            :disabled="readonly"
            :class="[sheet.grow, sheet.slider]"
            :style="`--slider-value: ${(current / maximum) * 100}%`"
            @input="handleInput"
            @wheel="handleWheel"
        />
        <input
            type="number"
            role="status"
            step="1"
            min="0"
            :max="maximum"
            :value="current"
            :disabled="readonly"
            :class="[sheet.borderlessNumber, sheet.titleFont]"
            style="width: 1.7em; text-align: center; margin-inline-start: 0.5em"
            @input="handleInput"
            @wheel.prevent="handleWheel"
        />
    </label>
</template>
