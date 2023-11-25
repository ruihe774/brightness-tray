import { observer } from "mobx-react-lite"
import React, { useCallback } from "react"
import monitorManager from "./monitor"
import { makeSliderStyle, sheet } from "./style"
import classNames from "classnames"
import { css } from "@emotion/css"

interface MonitorProps {
    monitorId: string
}

const MonitorItem = observer(function MonitorItem(props: MonitorProps) {
    const { monitorId } = props

    const monitor = monitorManager.getMonitor(monitorId)
    const name = monitor.name ?? monitor.id.split("#")[1]

    const powerButton = monitorManager.hasFeature(monitorId, "powerstate") ? (
        <button type="button" className={sheet.borderlessButton}>
            <span className={sheet.icon} aria-hidden>
                &#xE7E8;
            </span>
            <span>Power off</span>
        </button>
    ) : null

    const nameRow = (
        <div
            className={classNames(
                sheet.horizontalFlex,
                sheet.spreadContent,
                sheet.centerItems,
                sheet.cozyLine,
            )}
        >
            <div>
                <span className={sheet.bigIcon} aria-label="monitor">
                    &#xE7F4;
                </span>
                <span
                    className={classNames(
                        sheet.titleFont,
                        css`
                            margin-left: 0.15em;
                        `,
                    )}
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
                padding-right: 0.6em;
            `}
        >
            {nameRow}
            <ul
                className={classNames(
                    sheet.resetSpacing,
                    sheet.verticalFlex,
                    sheet.stretchItems,
                )}
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
            className={classNames(
                sheet.resetSpacing,
                sheet.verticalFlex,
                sheet.stretchItems,
            )}
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
            const value = Number(e.target.value)
            monitorManager.setFeature(monitorId, feature, value)
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
        <label className={classNames(sheet.flex, sheet.cozyLine)}>
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
                className={classNames(sheet.grow, makeSliderStyle(current))}
            />
        </label>
    )
})
