import { useCallback } from "react"
import { observer } from "mobx-react-lite"
import monitorManager from "./monitor"

export default observer(function App() {
    const { monitors } = monitorManager

    const refreshHandler = useCallback(() => {
        monitorManager.refreshMonitors()
    }, [])

    const changeHandle = useCallback(
        (e: React.ChangeEvent<HTMLInputElement>) => {
            const {
                value,
                dataset: { monitor: id, feature: name },
            } = e.target
            monitorManager.setFeature(id!, name!, Number(value))
        },
        [],
    )

    return (
        <div className="container">
            <button type="button" onClick={refreshHandler}>
                Refresh
            </button>
            <ul>
                {monitors.map(monitor => (
                    <li key={monitor.id}>
                        <b>{monitor.name}</b>: <code>{monitor.id}</code>
                        <table>
                            {monitor.features
                                .filter(({ value }) => value.maximum)
                                .map(({ name, value }) => (
                                    <label
                                        key={name}
                                        style={{
                                            display: "table-row",
                                            verticalAlign: "middle",
                                        }}
                                    >
                                        <td>{name}</td>
                                        <td>
                                            <input
                                                type="range"
                                                min="0"
                                                max={value.maximum}
                                                step="1"
                                                value={value.current}
                                                data-monitor={monitor.id}
                                                data-feature={name}
                                                onChange={changeHandle}
                                            />
                                        </td>
                                        <td>
                                            <output>{value.current}</output>
                                        </td>
                                    </label>
                                ))}
                        </table>
                    </li>
                ))}
            </ul>
        </div>
    )
})
