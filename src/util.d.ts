import { WatchCallback, WatchOptions, WatchStopHandle, WatchSource } from "vue"

declare interface WatchDelayedOptions<Immediate>
    extends WatchOptions<Immediate> {
    delay?: number
    trailing?: boolean
    leading?: boolean
}

type MultiWatchSources = (WatchSource<unknown> | object)[]

type MapSources<T, Immediate> = {
    [K in keyof T]: T[K] extends WatchSource<infer V>
        ? Immediate extends true
            ? V | undefined
            : V
        : T[K] extends object
          ? Immediate extends true
              ? T[K] | undefined
              : T[K]
          : never
}

declare function watchDelayed<
    T extends MultiWatchSources,
    Immediate extends Readonly<boolean> = false,
>(
    sources: [...T],
    cb: WatchCallback<MapSources<T, false>, MapSources<T, Immediate>>,
    options?: WatchDelayedOptions<Immediate> | number,
): WatchStopHandle

declare function watchDelayed<
    T extends Readonly<MultiWatchSources>,
    Immediate extends Readonly<boolean> = false,
>(
    source: T,
    cb: WatchCallback<MapSources<T, false>, MapSources<T, Immediate>>,
    options?: WatchDelayedOptions<Immediate> | number,
): WatchStopHandle

declare function watchDelayed<T, Immediate extends Readonly<boolean> = false>(
    source: WatchSource<T>,
    cb: WatchCallback<T, Immediate extends true ? T | undefined : T>,
    options?: WatchDelayedOptions<Immediate> | number,
): WatchStopHandle

declare function watchDelayed<
    T extends object,
    Immediate extends Readonly<boolean> = false,
>(
    source: T,
    cb: WatchCallback<T, Immediate extends true ? T | undefined : T>,
    options?: WatchDelayedOptions<Immediate> | number,
): WatchStopHandle
