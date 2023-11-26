<script lang="ts">
import { css } from "@emotion/css"
import { defineComponent } from "vue"
import { sheet } from "../style"
import monitorManager from "../monitor"
import settings from "../settings"
import FeatureSlider from "./FeatureSlider.vue"

export default defineComponent({
    props: {
        monitorId: {
            type: String,
            required: true,
        },
    },
    computed: {
        monitor() {
            return monitorManager.getMonitor(this.monitorId)
        },
        name() {
            return this.monitor.name ?? this.monitor.id.split("#")[1]
        },
        powerStateMaximum() {
            try {
                return monitorManager.getFeature(this.monitorId, "powerstate")
                    .value.maximum
            } catch {
                return 0
            }
        },
        css() {
            return css
        },
        sheet() {
            return sheet
        },
    },
    methods: {
        handlePowerOff() {
            monitorManager.setFeature(
                this.monitorId,
                "powerstate",
                Math.min(settings.ddcPowerOffValue, this.powerStateMaximum),
            )
        },
    },
    components: {
        FeatureSlider,
    },
})
</script>

<template>
    <div
        :class="
            css`
                padding-inline-end: 0.6em;
            `
        "
    >
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
                    :class="[
                        sheet.titleFont,
                        css`
                            margin-inline-start: 0.15em;
                        `,
                    ]"
                >
                    {{ name }}
                </span>
            </div>
            <button
                v-if="powerStateMaximum >= 4"
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
            <template v-for="{ name, value } in monitor.features">
                <li
                    v-if="name != 'powerstate' && value.maximum"
                    :key="name"
                    :class="sheet.resetSpacing"
                >
                    <FeatureSlider :monitorId="monitorId" :feature="name" />
                </li>
            </template>
        </ul>
    </div>
</template>
