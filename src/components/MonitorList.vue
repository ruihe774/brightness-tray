<script setup lang="ts">
import { sheet } from "../style"
import monitorManager from "../monitor"
import MonitorItem from "./MonitorItem.vue"
const { monitors } = monitorManager
</script>

<template>
    <ul :class="[sheet.resetSpacing, sheet.verticalFlex, sheet.stretchItems]">
        <li v-for="{ id } in monitors" :key="id" :class="sheet.resetSpacing">
            <MonitorItem :monitorId="id" />
        </li>
    </ul>
</template>
