import { invoke } from "@tauri-apps/api"
import { listen, Event } from "@tauri-apps/api/event"
import {
    LogicalPosition,
    LogicalSize,
    PhysicalPosition,
    appWindow,
} from "@tauri-apps/api/window"
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

interface RawPosition {
    x: number
    y: number
}

async function locatePanel(positionInMonitor?: RawPosition) {
    const anchorPosition =
        positionInMonitor ?? (await appWindow.innerPosition())
    const windowSize = new LogicalSize(
        Math.max(300, panelState.width),
        Math.max(50, panelState.height),
    )
    const { width, height } = windowSize
    const cornerPosition = await invoke<RawPosition>("get_workarea_corner", {
        position: {
            x: anchorPosition.x,
            y: anchorPosition.y,
        },
    })
    let { x: right, y: bottom } = new PhysicalPosition(
        cornerPosition.x,
        cornerPosition.y,
    ).toLogical(await appWindow.scaleFactor())
    right -= 12
    bottom -= 12
    const windowPosition = new LogicalPosition(right - width, bottom - height)
    await appWindow.setPosition(windowPosition)
    await appWindow.setSize(windowSize)
}

watchDelayed(() => (panelState.width, panelState.height, void 0), locatePanel, {
    delay: 500,
    leading: true,
})

appWindow.onScaleChanged(() => locatePanel())

listen(
    "tray-icon-click",
    async ({ payload: clickPosition }: Event<RawPosition>) => {
        if (await appWindow.isVisible()) {
            await appWindow.hide()
        } else {
            await locatePanel(clickPosition)
            await appWindow.show()
            await appWindow.setFocus()
            await invoke("refresh_mica")
        }
    },
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
