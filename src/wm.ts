import { LogicalSize, appWindow } from "@tauri-apps/api/window"
import { reactive } from "vue"
import { watchDelayed } from "./util"

const panelState = reactive({
    width: 0,
    height: 0,
    focused: false,
})

const resizoObserver = new ResizeObserver(entries => {
    const entry = entries[0]
    const borderBox = entry.borderBoxSize[0]
    panelState.width = borderBox.inlineSize
    panelState.height = borderBox.blockSize
})
resizoObserver.observe(document.getElementsByTagName("html")[0])

appWindow.onFocusChanged(({ payload }) => {
    panelState.focused = payload
})

watchDelayed(
    () => ({ width: panelState.width, height: panelState.height }),
    ({ width, height }) => {
        if (width * height > 30000) {
            appWindow.setSize(new LogicalSize(width, height))
        }
    },
    500,
)

if (import.meta.env.PROD) {
    watchDelayed(
        () => panelState.focused,
        focused => {
            if (!focused) {
                appWindow.hide()
            }
        },
        100,
    )
}

export default panelState
