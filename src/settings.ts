import { reactive } from "vue";

export interface Settings {
    updateInterval: number;
    ddcPowerOffValue: number;
    writingMode: string;
}

export default reactive<Settings>({
    updateInterval: 500,
    ddcPowerOffValue: 6,
    writingMode: "horizontal-tb",
});
