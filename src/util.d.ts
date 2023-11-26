import type {
    MapSources,
    MapOldSources,
    WatchThrottledOptions,
} from "@vueuse/shared"
import type { WatchSource, WatchCallback, WatchStopHandle } from "vue"

declare function watchDelayed<
    T extends Readonly<WatchSource<unknown>[]>,
    Immediate extends Readonly<boolean> = false,
>(
    sources: [...T],
    cb: WatchCallback<MapSources<T>, MapOldSources<T, Immediate>>,
    options?: WatchThrottledOptions<Immediate>,
): WatchStopHandle

declare function watchDelayed<T, Immediate extends Readonly<boolean> = false>(
    source: WatchSource<T>,
    cb: WatchCallback<T, Immediate extends true ? T | undefined : T>,
    options?: WatchThrottledOptions<Immediate>,
): WatchStopHandle

declare function watchDelayed<
    T extends object,
    Immediate extends Readonly<boolean> = false,
>(
    source: T,
    cb: WatchCallback<T, Immediate extends true ? T | undefined : T>,
    options?: WatchThrottledOptions<Immediate>,
): WatchStopHandle

declare function watchDelayed<
    T extends Readonly<WatchSource<unknown>[]>,
    Immediate extends Readonly<boolean> = false,
>(
    sources: [...T],
    cb: WatchCallback<MapSources<T>, MapOldSources<T, Immediate>>,
    throttle: number,
): WatchStopHandle

declare function watchDelayed<T, Immediate extends Readonly<boolean> = false>(
    source: WatchSource<T>,
    cb: WatchCallback<T, Immediate extends true ? T | undefined : T>,
    throttle: number,
): WatchStopHandle

declare function watchDelayed<
    T extends object,
    Immediate extends Readonly<boolean> = false,
>(
    source: T,
    cb: WatchCallback<T, Immediate extends true ? T | undefined : T>,
    throttle: number,
): WatchStopHandle
