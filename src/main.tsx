import { LogicalSize, appWindow } from "@tauri-apps/api/window"
import { autorun, observable, runInAction } from "mobx"
import React from "react"
import ReactDOM from "react-dom/client"
import Panel from "./Panel"
import monitorManager from "./monitor"
import "./style.css"

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
        <Panel />
    </React.StrictMode>,
)

monitorManager.refreshMonitors()

document.addEventListener("contextmenu", e => e.preventDefault())

const panelState = observable({
    width: 0,
    height: 0,
    focused: false,
})

const resizoObserver = new ResizeObserver(entries => {
    const entry = entries[0]
    const borderBox = entry.borderBoxSize[0]
    runInAction(() => {
        panelState.width = borderBox.inlineSize
        panelState.height = borderBox.blockSize
    })
})
resizoObserver.observe(document.getElementsByTagName("html")[0])

appWindow.onFocusChanged(({ payload }) => {
    runInAction(() => {
        panelState.focused = payload
    })
})

autorun(
    () => {
        let { width, height } = panelState
        if (width * height > 30000) {
            appWindow.setSize(new LogicalSize(width, height))
        }
    },
    {
        delay: 500,
    },
)

autorun(
    () => {
        if (!panelState.focused) {
            appWindow.hide()
        }
    },
    {
        delay: 100,
    },
)
