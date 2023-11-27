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

async function locatePanel(
    positionInMonitor?: RawPosition,
    flyIn?: boolean,
): Promise<Animation | undefined> {
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
    const { x: left, y: top } = windowPosition
    let animation: Animation | undefined
    if (flyIn) {
        const startPosition = new LogicalPosition(left, top + height)
        animation = fly(startPosition, windowPosition, "ease-out")
    } else {
        await appWindow.setPosition(windowPosition)
    }
    await appWindow.setSize(windowSize)
    return animation
}

function fly(
    startPosition: LogicalPosition,
    endPosition: LogicalPosition,
    easing?: string,
): Animation {
    const stub = document.createElement("div")
    stub.style.position = "absolute"
    stub.style.visibility = "hidden"
    document.body.appendChild(stub)
    const animation = stub.animate(
        [
            {
                left: startPosition.x + "px",
                top: startPosition.y + "px",
            },
            {
                left: endPosition.x + "px",
                top: endPosition.y + "px",
            },
        ],
        {
            duration: 100,
            easing,
        },
    )
    let finished = false
    animation.onfinish = () => void (finished = true)
    requestAnimationFrame(function updatePosition() {
        if (finished) {
            appWindow.setPosition(endPosition)
            stub.remove()
        } else {
            animation.commitStyles()
            const { left, top } = stub.style
            appWindow.setPosition(
                new LogicalPosition(parseFloat(left), parseFloat(top)),
            )
            requestAnimationFrame(updatePosition)
        }
    })
    return animation
}

watchDelayed(() => (panelState.width, panelState.height, void 0), locatePanel, {
    delay: 500,
    leading: true,
})

appWindow.onScaleChanged(() => locatePanel())

function preferReducedMotion(): boolean {
    return matchMedia("(prefers-reduced-motion)").matches
}

async function showWindow(clickPosition?: RawPosition) {
    await locatePanel(clickPosition, !preferReducedMotion())
    await appWindow.show()
    await appWindow.setFocus()
    await invoke("refresh_mica")
}

async function hideWindow() {
    if (!preferReducedMotion()) {
        const windowPosition = (await appWindow.outerPosition()).toLogical(
            await appWindow.scaleFactor(),
        )
        const endPosition = new LogicalPosition(
            windowPosition.x,
            windowPosition.y + panelState.height + 50,
        )
        await fly(windowPosition, endPosition, "ease-in").finished
    }
    await appWindow.hide()
}

listen(
    "tray-icon-click",
    async ({ payload: clickPosition }: Event<RawPosition>) => {
        if (await appWindow.isVisible()) {
            await hideWindow()
        } else {
            await showWindow(clickPosition)
        }
    },
)

if (import.meta.env.PROD) {
    watchDelayed(
        () => panelState.focused,
        focused => {
            if (!focused) {
                hideWindow()
            }
        },
        100,
    )
}

export default panelState
