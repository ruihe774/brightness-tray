import { invoke } from "@tauri-apps/api";
import { listen, Event } from "@tauri-apps/api/event";
import { LogicalPosition, LogicalSize, PhysicalPosition, appWindow } from "@tauri-apps/api/window";
import { reactive, watch, watchEffect } from "vue";
import { watchDelayed } from "./util";

const panelState = reactive({
    width: 0,
    height: 0,
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
    flyIn?: boolean,
): Promise<Animation | undefined> {
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
    let animation: Animation | undefined;
    if (flyIn) {
        const startPosition = new LogicalPosition(left, top + height);
        animation = fly(startPosition, windowPosition, "ease-out");
    } else {
        await appWindow.setPosition(windowPosition);
    }
    await appWindow.setSize(windowSize);
    return animation;
}

CSS.registerProperty({
    name: "--fly-animation-x",
    syntax: "<number>",
    inherits: false,
    initialValue: "0",
});
CSS.registerProperty({
    name: "--fly-animation-y",
    syntax: "<number>",
    inherits: false,
    initialValue: "0",
});

function fly(
    startPosition: LogicalPosition,
    endPosition: LogicalPosition,
    easing?: string,
): Animation {
    const stub = document.createElement("div");
    stub.style.position = "absolute";
    stub.style.visibility = "hidden";
    document.body.appendChild(stub);
    const animation = stub.animate(
        [
            {
                "--fly-animation-x": startPosition.x,
                "--fly-animation-y": startPosition.y,
            },
            {
                "--fly-animation-x": endPosition.x,
                "--fly-animation-y": endPosition.y,
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
            appWindow.setPosition(endPosition);
            stub.remove();
        } else {
            animation.commitStyles();
            const left = stub.style.getPropertyValue("--fly-animation-x");
            const top = stub.style.getPropertyValue("--fly-animation-y");
            appWindow.setPosition(new LogicalPosition(Number(left), Number(top)));
            requestAnimationFrame(updatePosition);
        }
    });
    return animation;
}

watchDelayed(() => void (panelState.width, panelState.height), locatePanel, {
    delay: 500,
    leading: true,
});

watch(() => void panelState.scaleFactor, locatePanel);

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
        await fly(windowPosition, endPosition, "ease-in").finished;
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
