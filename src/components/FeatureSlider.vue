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
        featureName: {
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
        feature() {
            return monitorManager.getFeature(this.monitorId, this.featureName);
        },
        readonly() {
            return this.feature.readonly;
        },
        current() {
            return this.feature.value.current;
        },
        maximum() {
            return this.feature.value.maximum;
        },
        icon() {
            return (iconMap as { [key: string]: string })[this.featureName];
        },
    },
    methods: {
        handleInput(event: Event) {
            const e = event as InputEvent;
            const target = e.target! as HTMLInputElement;
            if (target.validity.valid) {
                const value = Number(target.value);
                monitorManager.updateFeature(this.monitorId, this.featureName, value);
            }
        },
        handleWheel(event: Event) {
            if (this.readonly) {
                return;
            }
            const e = event as WheelEvent;
            const target = e.currentTarget! as HTMLInputElement;
            if (e.deltaMode == WheelEvent.DOM_DELTA_PIXEL) {
                const offset = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : -e.deltaY;
                const current = Number(target.value);
                monitorManager.updateFeature(
                    this.monitorId,
                    this.featureName,
                    Math.clamp(Math.round(current + offset * 0.01), 0, this.maximum),
                );
            }
        },
    },
});
</script>

<template>
    <label :class="[sheet.flex, sheet.cozyLine, { [sheet.inactive]: readonly }]">
        <span :class="sheet.bigIcon" :aria-label="featureName">
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
            :class="[sheet.borderlessNumber, sheet.titleFont, 'number']"
            style="width: 1.7em"
            @input="handleInput"
            @wheel.prevent="handleWheel"
        />
    </label>
</template>

<style scoped lang="sass">
.number
    margin-left: 0.5em
    [data-writing-mode^=vertical] &
        margin-left: 0
        margin-top: -0.3em
        transform: translate(-0.15em, 0.2em)
</style>
