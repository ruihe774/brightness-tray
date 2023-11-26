import { LogicalSize, appWindow } from "@tauri-apps/api/window"
import { reactive } from "vue"
import { watchDebounced } from "@vueuse/core"

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

watchDebounced(
    () => ({ width: panelState.width, height: panelState.height }),
    ({ width, height }) => {
        if (width * height > 30000) {
            appWindow.setSize(new LogicalSize(width, height))
        }
    },
    { debounce: 500 },
)

if (import.meta.env.PROD) {
    watchDebounced(
        () => panelState.focused,
        focused => {
            if (!focused) {
                appWindow.hide()
            }
        },
        { debounce: 100 },
    )
}

export default panelState
