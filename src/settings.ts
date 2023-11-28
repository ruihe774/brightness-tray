import { reactive } from "vue";

export interface Settings {
    updateInterval: number;
    ddcPowerOffValue: number;
}

export default reactive<Settings>({
    updateInterval: 500,
    ddcPowerOffValue: 6,
});
