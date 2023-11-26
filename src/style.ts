import { css } from "@emotion/css"
import { invoke } from "@tauri-apps/api/tauri"
import { autorun, observable, runInAction } from "mobx"
import panelState from "./wm"

export interface Color {
    r: number
    g: number
    b: number
}

export interface AccentColors {
    accent: Color
    accentDark1: Color
    accentDark2: Color
    accentDark3: Color
    accentLight1: Color
    accentLight2: Color
    accentLight3: Color
    background: Color
    foreground: Color
}

export const colors = observable(
    await invoke<AccentColors>("get_accent_colors"),
)

function colorToCSS(color: Color): string {
    return `rgb(${color.r}, ${color.g}, ${color.b})`
}

export const sheet = observable<{ [key: string]: string }>({})

autorun(() =>
    runInAction(() => {
        const icon = css`
            display: inline-block;
            font-family: "Segoe Fluent Icons", "Segoe MDL2 Assets";
            transform: scale(1.1) translateY(0.12em);
            width: 1.1em;
            margin: 0 0.4em;
            text-align: center;
        `

        const resetSpacing = css`
            @layer reset {
                margin: 0;
                padding: 0;
            }
        `

        const resetAppearance = css`
            @layer reset {
                appearance: none;
            }
        `

        const resetInput = css`
            ${resetSpacing};
            ${resetAppearance};
            @layer reset {
                font: inherit;
                color: inherit;
                box-sizing: content-box;
                background: none;
                border: none;
            }
        `

        const borderlessButton = css`
            ${resetInput};
            cursor: pointer;
            opacity: 0.7;

            &:hover {
                opacity: 1;
            }
        `

        const borderlessNumber = css`
            ${resetInput};
            &::-webkit-inner-spin-button,
            &::-webkit-outer-spin-button {
                ${resetAppearance};
                margin: 0;
            }
        `

        const block = css`
            display: block;
        `

        const flex = css`
            display: flex;
            & > * {
                ${block};
            }
        `

        const horizontalFlex = css`
            ${flex};
            flex-direction: row;
        `

        const verticalFlex = css`
            ${flex};
            flex-direction: column;
        `

        const spreadContent = css`
            justify-content: space-between;
            & > * {
                flex-grow: 0;
            }
        `

        const stretchContent = css`
            justify-content: stretch;
        `

        const centerItems = css`
            align-items: center;
        `

        const stretchItems = css`
            align-items: stretch;
        `

        const fillWidth = css`
            width: 100%;
        `

        const titleFont = css`
            font-size: 1.1rem;
            font-variation-settings: "wght" 350;
        `

        const bigIcon = css`
            ${icon};
            font-weight: 300;
            transform: scale(1.6) translateY(0.12em);
            width: 1.6em;
            margin: 0 0.6em;
        `

        const cozyLine = css`
            line-height: 2.2;
        `

        const grow = css`
            flex-grow: 1;
        `

        Object.assign(sheet, {
            icon,
            resetSpacing,
            resetAppearance,
            resetInput,
            borderlessButton,
            borderlessNumber,
            block,
            flex,
            horizontalFlex,
            verticalFlex,
            spreadContent,
            stretchContent,
            centerItems,
            stretchItems,
            fillWidth,
            titleFont,
            bigIcon,
            cozyLine,
            grow,
        })
    }),
)

export function makeSliderStyle(value: number) {
    return css`
        --slider-thumb-color: ${colorToCSS(colors.accentDark1)};
        --slider-thumb-border: #ffffff;
        --slider-track-color: #868686;
        @media screen and (prefers-color-scheme: dark) {
            --slider-thumb-color: ${colorToCSS(colors.accentLight2)};
            --slider-thumb-border: #414141;
            --slider-track-color: #9b9b9b;
        }

        ${sheet.resetInput};
        margin: 0.15rem;

        &::-webkit-slider-thumb {
            ${sheet.resetInput};
            box-sizing: content-box;
            height: 0.7rem;
            width: 0.7rem;
            border-radius: 1em;
            background: var(--slider-thumb-color);
            box-shadow:
                0 0 0 0.3rem var(--slider-thumb-border),
                0 0.04rem 0.2rem 0 rgba(0, 0, 0, 0.6);
            transform: translateY(-0.2rem) scale(1);
            &:hover {
                box-shadow:
                    0 0 0 0.15rem var(--slider-thumb-border),
                    0 0.04rem 0.2rem 0 rgba(0, 0, 0, 0.6);
                transform: translateY(-0.2rem) scale(1.3);
            }
            transition:
                box-shadow 0.15s,
                transform 0.15s;
        }
        &::-webkit-slider-runnable-track {
            ${sheet.resetInput};
            background: linear-gradient(
                to right,
                var(--slider-thumb-color),
                var(--slider-thumb-color) ${value}%,
                var(--slider-track-color) ${value}%,
                var(--slider-track-color)
            );
            height: 0.3rem;
            border-radius: 0.6rem;
        }
    `
}

autorun(
    () => {
        if (panelState.focused) {
            invoke<AccentColors>("get_accent_colors").then(newColors =>
                runInAction(() => {
                    Object.assign(colors, newColors)
                }),
            )
        }
    },
    {
        delay: 10,
    },
)
