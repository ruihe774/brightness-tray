<script lang="ts">
import { defineComponent } from "vue";
import { clamp } from "lodash-es";
import monitorManager from "../monitor";
import sheet from "../style.module.sass";

const iconMap: { [key: string]: string } = {
    luminance: "\uE706",
    contrast: "\uE793",
    brightness: "\uE706",
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
    data(): {
        input: number | null;
        updating: Promise<void>;
    } {
        return {
            input: null,
            updating: Promise.resolve(),
        };
    },
    computed: {
        feature() {
            return monitorManager.getFeature(this.monitorId, this.featureName);
        },
        current() {
            return this.feature.value.current;
        },
        maximum() {
            return this.feature.value.maximum;
        },
        icon() {
            return iconMap[this.featureName];
        },
    },
    methods: {
        handleInput(event: Event) {
            const e = event as InputEvent;
            const target = e.target! as HTMLInputElement;
            if (!e.isComposing && target.validity.valid) {
                this.input = Number(target.value);
                this.update();
            }
        },
        handleWheel(event: Event) {
            const e = event as WheelEvent;
            const target = e.currentTarget! as HTMLInputElement;
            if (e.deltaMode == WheelEvent.DOM_DELTA_PIXEL) {
                const offset = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : -e.deltaY;
                const current = Number(target.value);
                this.input = clamp(Math.round(current + offset * 0.01), 0, this.maximum);
                this.update();
                this.sync();
            }
        },
        update() {
            this.updating = this.updating.then(() => {
                if (this.input != null) {
                    try {
                        return monitorManager.setFeature(
                            this.monitorId,
                            this.featureName,
                            this.input,
                        );
                    } catch (e) {
                        console.error(e);
                    }
                }
            });
        },
        sync() {
            this.updating = this.updating.then(() => {
                this.input = null;
            });
        },
    },
});
</script>

<template>
    <label :class="[sheet.flex, sheet.cozyLine]">
        <span :class="sheet.bigIcon" :aria-label="featureName">
            {{ icon }}
        </span>
        <input
            type="range"
            step="1"
            min="0"
            :max="maximum"
            :value="input ?? current"
            :class="[sheet.grow, sheet.slider]"
            :style="`--slider-value: ${(current / maximum) * 100}%`"
            @input="handleInput"
            @change="sync"
            @wheel.prevent="handleWheel"
        />
        <input
            type="number"
            role="status"
            step="1"
            min="0"
            :max="maximum"
            :value="input ?? current"
            :class="[sheet.borderlessNumber, sheet.titleFont, 'number']"
            style="width: 1.7em"
            @input="handleInput"
            @change="sync"
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
