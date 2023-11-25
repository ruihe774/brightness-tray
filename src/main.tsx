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

const documentSize = observable({
    width: 0,
    height: 0,
})

const resizoObserver = new ResizeObserver(entries => {
    const entry = entries[0]
    const borderBox = entry.borderBoxSize[0]
    runInAction(() => {
        documentSize.width = borderBox.inlineSize
        documentSize.height = borderBox.blockSize
    })
})
resizoObserver.observe(document.getElementsByTagName("html")[0])

autorun(
    () => {
        let { width, height } = documentSize
        if (width * height > 30000) {
            appWindow.setSize(new LogicalSize(width, height))
        }
    },
    {
        delay: 500,
    },
)

await appWindow.onFocusChanged(({ payload: focused }) => {
    if (!focused) {
        appWindow.hide()
    }
})
