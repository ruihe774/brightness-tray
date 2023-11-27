<script setup lang="ts">
import monitorManager from "../monitor"
import MonitorItem from "./MonitorItem.vue"
import sheet from "../style.module.sass"
const { monitors } = monitorManager
</script>

<template>
    <ul :class="[sheet.resetSpacing, sheet.verticalFlex, sheet.stretchItems]">
        <li v-for="{ id } in monitors" :key="id" :class="sheet.resetSpacing">
            <MonitorItem :monitor-id="id" />
        </li>
    </ul>
</template>
