<script lang="ts">
import { defineComponent } from "vue"
import monitorManager from "../monitor"
import settings from "../settings"
import FeatureSlider from "./FeatureSlider.vue"
import sheet from "../style.module.sass"

export default defineComponent({
    components: {
        FeatureSlider,
    },
    props: {
        monitorId: {
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
        monitor() {
            return monitorManager.getMonitor(this.monitorId)
        },
        name() {
            return this.monitor.name ?? this.monitor.id.split("#")[1]
        },
        powerState() {
            try {
                return monitorManager.getFeature(this.monitorId, "powerstate")
                    .value
            } catch {
                return void 0
            }
        },
    },
    methods: {
        handlePowerOff() {
            monitorManager.setFeature(
                this.monitorId,
                "powerstate",
                Math.min(settings.ddcPowerOffValue, this.powerState!.maximum),
            )
        },
    },
})
</script>

<template>
    <div style="padding-inline-end: 0.6em">
        <div
            :class="[
                sheet.horizontalFlex,
                sheet.spreadContent,
                sheet.centerItems,
                sheet.cozyLine,
            ]"
        >
            <div>
                <span :class="sheet.bigIcon" aria-label="monitor">
                    &#xE7F4;
                </span>
                <span
                    :class="sheet.titleFont"
                    style="margin-inline-start: 0.15em"
                >
                    {{ name }}
                </span>
            </div>
            <button
                v-if="
                    powerState &&
                    powerState.maximum >= 4 &&
                    powerState.current < 4
                "
                type="button"
                :class="sheet.borderlessButton"
                @click="handlePowerOff"
            >
                <span :class="sheet.icon" aria-label="power off">
                    &#xE7E8;
                </span>
            </button>
        </div>
        <ul
            :class="[
                sheet.resetSpacing,
                sheet.verticalFlex,
                sheet.stretchItems,
            ]"
        >
            <template v-for="{ name: feature, value } in monitor.features">
                <li
                    v-if="feature != 'powerstate' && value.maximum"
                    :key="feature"
                    :class="sheet.resetSpacing"
                >
                    <FeatureSlider :monitor-id="monitorId" :feature="feature" />
                </li>
            </template>
        </ul>
    </div>
</template>
