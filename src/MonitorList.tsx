import { css } from "@emotion/css"
import { observer } from "mobx-react-lite"
import React, { useCallback } from "react"
import monitorManager from "./monitor"
import settings from "./settings"
import { makeSliderStyle, sheet } from "./style"

interface MonitorProps {
    monitorId: string
}

const MonitorItem = observer(function MonitorItem(props: MonitorProps) {
    const { monitorId } = props

    const monitor = monitorManager.getMonitor(monitorId)
    const name = monitor.name ?? monitor.id.split("#")[1]

    let powerStateMaximum = 0
    try {
        powerStateMaximum = monitorManager.getFeature(monitorId, "powerstate")
            .value.maximum
    } catch {}

    const handlePowerOff = useCallback(() => {
        monitorManager.setFeature(
            monitorId,
            "powerstate",
            Math.min(settings.ddcPowerOffValue, powerStateMaximum),
        )
    }, [monitorId])

    const powerButton =
        powerStateMaximum >= 4 ? (
            <button
                type="button"
                className={sheet.borderlessButton}
                onClick={handlePowerOff}
            >
                <span className={sheet.icon} aria-label="power off">
                    &#xE7E8;
                </span>
            </button>
        ) : null

    const nameRow = (
        <div
            classList={[
                sheet.horizontalFlex,
                sheet.spreadContent,
                sheet.centerItems,
                sheet.cozyLine,
            ]}
        >
            <div>
                <span className={sheet.bigIcon} aria-label="monitor">
                    &#xE7F4;
                </span>
                <span
                    classList={[
                        sheet.titleFont,
                        css`
                            margin-inline-start: 0.15em;
                        `,
                    ]}
                >
                    {name}
                </span>
            </div>
            {powerButton}
        </div>
    )

    const sliders = monitor.features
        .filter(
            feature => feature.name != "powerstate" && feature.value.maximum,
        )
        .map(({ name }) => (
            <li key={name} className={sheet.resetSpacing}>
                <FeatureSlider monitorId={monitorId} feature={name} />
            </li>
        ))

    return (
        <div
            className={css`
                padding-inline-end: 0.6em;
            `}
        >
            {nameRow}
            <ul
                classList={[
                    sheet.resetSpacing,
                    sheet.verticalFlex,
                    sheet.stretchItems,
                ]}
            >
                {sliders}
            </ul>
        </div>
    )
})

export default observer(function MonitorList() {
    const { monitors } = monitorManager

    return (
        <ul
            classList={[
                sheet.resetSpacing,
                sheet.verticalFlex,
                sheet.stretchItems,
            ]}
        >
            {monitors.map(({ id }) => (
                <li key={id} className={sheet.resetSpacing}>
                    <MonitorItem monitorId={id} />
                </li>
            ))}
        </ul>
    )
})

interface FeatureSliderProps {
    monitorId: string
    feature: string
}

const FeatureSlider = observer(function FeatureSlider(
    props: FeatureSliderProps,
) {
    const { monitorId, feature } = props

    const {
        value: { current, maximum },
    } = monitorManager.getFeature(monitorId, feature)

    const handleChange = useCallback(
        (e: React.ChangeEvent<HTMLInputElement>) => {
            if (e.target.validity.valid) {
                const value = Number(e.target.value)
                monitorManager.setFeature(monitorId, feature, value)
            }
        },
        [monitorId, feature],
    )

    const handleWheel = useCallback(
        (e: React.WheelEvent<HTMLInputElement>) => {
            if (e.deltaMode == WheelEvent.DOM_DELTA_PIXEL) {
                e.preventDefault()
                const offset =
                    Math.abs(e.deltaX) > Math.abs(e.deltaY)
                        ? e.deltaX
                        : -e.deltaY
                const current = Number(e.currentTarget.value)
                monitorManager.setFeature(
                    monitorId,
                    feature,
                    Math.max(
                        0,
                        Math.min(maximum, Math.round(current + offset * 0.01)),
                    ),
                )
            }
        },
        [monitorId, feature],
    )

    const iconMap: { [key: string]: string } = {
        luminance: "\uE706",
        contrast: "\uE793",
        brightness: "\uE7E8",
        volume: "\uE767",
        powerstate: "\uE7E8",
    }

    return (
        <label classList={[sheet.flex, sheet.cozyLine]}>
            <span className={sheet.bigIcon} aria-label={feature}>
                {iconMap[feature]}
            </span>
            <input
                type="range"
                step="1"
                min="0"
                max={maximum}
                value={current}
                onChange={handleChange}
                onWheel={handleWheel}
                classList={[sheet.grow, makeSliderStyle(current)]}
            />
            <input
                type="number"
                step="1"
                min="0"
                max={maximum}
                value={current}
                onChange={handleChange}
                onWheel={handleWheel}
                role="status"
                classList={[
                    sheet.borderlessNumber,
                    sheet.titleFont,
                    css`
                        width: 1.7em;
                        text-align: center;
                        margin-inline-start: 0.5em;
                    `,
                ]}
            />
        </label>
    )
})
