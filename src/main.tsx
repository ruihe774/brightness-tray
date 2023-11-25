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
