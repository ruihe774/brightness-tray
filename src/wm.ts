import { invoke } from "@tauri-apps/api";
import { listen, Event } from "@tauri-apps/api/event";
import { LogicalPosition, LogicalSize, PhysicalPosition, appWindow } from "@tauri-apps/api/window";
import { reactive, watch, watchEffect } from "vue";
import { watchDelayed, watchThrottled } from "./watchers";

const panelState = reactive({
    width: 350,
    height: 200,
    focused: false,
    scaleFactor: await appWindow.scaleFactor(),
    theme: await appWindow.theme(),
});

const resizoObserver = new ResizeObserver((entries) => {
    const entry = entries[0];
    const borderBox = entry.borderBoxSize[0];
    panelState.width = borderBox.inlineSize;
    panelState.height = borderBox.blockSize;
});
resizoObserver.observe(document.querySelector("html")!);

appWindow.onFocusChanged(({ payload }) => {
    panelState.focused = payload;
});

appWindow.onScaleChanged(({ payload }) => {
    panelState.scaleFactor = payload.scaleFactor;
});

appWindow.onThemeChanged(({ payload }) => {
    panelState.theme = payload;
});

interface RawPosition {
    x: number;
    y: number;
}

async function locatePanel(
    positionInMonitor?: RawPosition,
    animated?: boolean,
    oldSize?: { width: number; height: number },
): Promise<Animation[]> {
    const anchorPosition = positionInMonitor ?? (await appWindow.innerPosition());
    const windowSize = new LogicalSize(
        Math.max(300, panelState.width),
        Math.max(50, panelState.height),
    );
    const { width, height } = windowSize;
    const cornerPosition = await invoke<RawPosition>("get_workarea_corner", {
        position: {
            x: anchorPosition.x,
            y: anchorPosition.y,
        },
    });
    let { x: right, y: bottom } = new PhysicalPosition(
        cornerPosition.x,
        cornerPosition.y,
    ).toLogical(await appWindow.scaleFactor());
    right -= 12;
    bottom -= 12;
    const windowPosition = new LogicalPosition(right - width, bottom - height);
    const { x: left, y: top } = windowPosition;
    const animations: Animation[] = [];
    if (animated) {
        const startPosition = new LogicalPosition(left, top + height - (oldSize?.height ?? 0));
        animations.push(animateWindow(startPosition, windowPosition, "ease-out"));
    } else {
        await appWindow.setPosition(windowPosition);
    }
    await appWindow.setSize(windowSize);
    return animations;
}

CSS.registerProperty({
    name: "--window-animation-x",
    syntax: "<number>",
    inherits: false,
    initialValue: "0",
});
CSS.registerProperty({
    name: "--window-animation-y",
    syntax: "<number>",
    inherits: false,
    initialValue: "0",
});

function animateWindow(start: LogicalPosition, end: LogicalPosition, easing?: string): Animation;
function animateWindow(start: LogicalSize, end: LogicalSize, easing?: string): Animation;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function animateWindow(start: any, end: any, easing?: string): Animation {
    const kind: "fly" | "resize" = "width" in start ? "resize" : "fly";
    const rawStart = {
        x: (start.x ?? start.width) as number,
        y: (start.y ?? start.height) as number,
    };
    const rawEnd = {
        x: (end.x ?? end.width) as number,
        y: (end.y ?? end.height) as number,
    };
    const stub = document.createElement("div");
    stub.style.position = "absolute";
    stub.style.visibility = "hidden";
    document.body.appendChild(stub);
    const animation = stub.animate(
        [
            {
                "--window-animation-x": rawStart.x,
                "--window-animation-y": rawStart.y,
            },
            {
                "--window-animation-x": rawEnd.x,
                "--window-animation-y": rawEnd.y,
            },
        ],
        {
            duration: 100,
            easing,
        },
    );
    let finished = false;
    animation.onfinish = () => void (finished = true);
    requestAnimationFrame(function updatePosition() {
        if (finished) {
            update(rawEnd);
            stub.remove();
        } else {
            animation.commitStyles();
            update({
                x: Number(stub.style.getPropertyValue("--window-animation-x")),
                y: Number(stub.style.getPropertyValue("--window-animation-y")),
            });
            requestAnimationFrame(updatePosition);
        }
    });
    function update(state: RawPosition) {
        if (kind == "fly") {
            appWindow.setPosition(new LogicalPosition(state.x, state.y));
        } else {
            appWindow.setSize(new LogicalSize(state.x, state.y));
        }
    }
    return animation;
}

watchThrottled(
    () => ({ width: panelState.width, height: panelState.height }),
    (_new, old) => locatePanel(void 0, !preferReducedMotion(), old),
    {
        throttle: 100,
        trailing: true,
    },
);

watch(
    () => panelState.scaleFactor,
    () => locatePanel(),
);

function preferReducedMotion(): boolean {
    return matchMedia("(prefers-reduced-motion)").matches;
}

async function showWindow(clickPosition?: RawPosition) {
    await locatePanel(clickPosition, !preferReducedMotion());
    await appWindow.show();
    await appWindow.setFocus();
    await invoke("refresh_mica");
}

async function hideWindow() {
    if (!preferReducedMotion()) {
        const windowPosition = (await appWindow.outerPosition()).toLogical(
            await appWindow.scaleFactor(),
        );
        const endPosition = new LogicalPosition(
            windowPosition.x,
            windowPosition.y + panelState.height + 50,
        );
        await animateWindow(windowPosition, endPosition, "ease-in").finished;
    }
    await appWindow.hide();
}

listen("tray-icon-click", async ({ payload: clickPosition }: Event<RawPosition>) => {
    if (await appWindow.isVisible()) {
        await hideWindow();
    } else {
        await showWindow(clickPosition);
    }
});

if (import.meta.env.PROD) {
    watchDelayed(
        () => panelState.focused,
        (focused) => {
            if (!focused) {
                hideWindow();
            }
        },
        100,
    );
}

watchEffect(() => {
    const baseSize = 16;
    const scaledSize = Math.round(baseSize * panelState.scaleFactor);
    const canvas = document.createElement("canvas");
    canvas.width = scaledSize;
    canvas.height = scaledSize;
    const ctx = canvas.getContext("2d")!;
    ctx.font = `${scaledSize}px Segoe Fluent Icons`;
    ctx.fillStyle = panelState.theme == "dark" ? "white" : "black";
    ctx.fillText("\uE706", 0, scaledSize);
    if (panelState.theme != "dark") {
        // make it a little bolder
        ctx.fillText("\uE706", 0, scaledSize);
    }
    const imageData = ctx.getImageData(0, 0, scaledSize, scaledSize);
    invoke("set_tray_icon", {
        icon: {
            rgba: Array.from(imageData.data),
            width: imageData.width,
            height: imageData.height,
        },
    });
});

export default panelState;
