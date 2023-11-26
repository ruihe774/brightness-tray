import { LogicalSize, appWindow } from "@tauri-apps/api/window"
import { autorun, observable, runInAction } from "mobx"

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
        const { width, height } = panelState
        if (width * height > 30000) {
            appWindow.setSize(new LogicalSize(width, height))
        }
    },
    {
        delay: 500,
    },
)

if (import.meta.env.PROD) {
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
}

export default panelState
