import type { WatchCallback, WatchOptions, WatchStopHandle, WatchSource } from "vue";
import { watch } from "vue";
import { debounce } from "lodash-es";

/* eslint-disable @typescript-eslint/no-explicit-any */

function watchInner(
    source: any,
    cb: any,
    options: any,
    leadingDefault: boolean,
    trailingDefault: boolean,
    renameKey: string,
): WatchStopHandle {
    if (typeof options != "object") {
        options = {
            [renameKey]: options,
        };
    }
    options = {
        leading: leadingDefault,
        trailing: trailingDefault,
        ...options,
    };
    const debounceOptions = {
        leading: options.leading,
        trailing: options.trailing,
        maxWait: options[renameKey],
    };
    delete options.leading;
    delete options.trailing;
    delete options[renameKey];

    const voidSymbol = Symbol();
    let oldValue: any = voidSymbol;
    const debounced = debounce(
        (newV, onCleanup) => {
            const oldV = oldValue;
            oldValue = voidSymbol;
            return cb(newV, oldV, onCleanup);
        },
        debounceOptions.maxWait,
        debounceOptions,
    );

    return watch(
        source,
        (newV, oldV, onCleanup) => {
            if (oldValue === voidSymbol) {
                oldValue = oldV;
            }
            return debounced(newV, onCleanup);
        },
        options,
    );
}

type MultiWatchSources = (WatchSource<unknown> | object)[];

type MapSources<T, Immediate> = {
    [K in keyof T]: T[K] extends WatchSource<infer V>
        ? Immediate extends true
            ? V | undefined
            : V
        : T[K] extends object
          ? Immediate extends true
              ? T[K] | undefined
              : T[K]
          : never;
};

interface WatchDelayedOptions<Immediate> extends WatchOptions<Immediate> {
    delay?: number;
    trailing?: boolean;
    leading?: boolean;
}

export function watchDelayed<
    T extends MultiWatchSources,
    Immediate extends Readonly<boolean> = false,
>(
    sources: [...T],
    cb: WatchCallback<MapSources<T, false>, MapSources<T, Immediate>>,
    options?: WatchDelayedOptions<Immediate> | number,
): WatchStopHandle;

export function watchDelayed<
    T extends Readonly<MultiWatchSources>,
    Immediate extends Readonly<boolean> = false,
>(
    source: T,
    cb: WatchCallback<MapSources<T, false>, MapSources<T, Immediate>>,
    options?: WatchDelayedOptions<Immediate> | number,
): WatchStopHandle;

export function watchDelayed<T, Immediate extends Readonly<boolean> = false>(
    source: WatchSource<T>,
    cb: WatchCallback<T, Immediate extends true ? T | undefined : T>,
    options?: WatchDelayedOptions<Immediate> | number,
): WatchStopHandle;

export function watchDelayed<T extends object, Immediate extends Readonly<boolean> = false>(
    source: T,
    cb: WatchCallback<T, Immediate extends true ? T | undefined : T>,
    options?: WatchDelayedOptions<Immediate> | number,
): WatchStopHandle;

export function watchDelayed(
    source: any,
    cb: any,
    options?: WatchDelayedOptions<any> | number,
): WatchStopHandle {
    return watchInner(source, cb, options, false, true, "delay");
}

interface WatchThrottledOptions<Immediate> extends WatchOptions<Immediate> {
    throttle?: number;
    trailing?: boolean;
    leading?: boolean;
}

export function watchThrottled<
    T extends MultiWatchSources,
    Immediate extends Readonly<boolean> = false,
>(
    sources: [...T],
    cb: WatchCallback<MapSources<T, false>, MapSources<T, Immediate>>,
    options?: WatchThrottledOptions<Immediate> | number,
): WatchStopHandle;

export function watchThrottled<
    T extends Readonly<MultiWatchSources>,
    Immediate extends Readonly<boolean> = false,
>(
    source: T,
    cb: WatchCallback<MapSources<T, false>, MapSources<T, Immediate>>,
    options?: WatchThrottledOptions<Immediate> | number,
): WatchStopHandle;

export function watchThrottled<T, Immediate extends Readonly<boolean> = false>(
    source: WatchSource<T>,
    cb: WatchCallback<T, Immediate extends true ? T | undefined : T>,
    options?: WatchThrottledOptions<Immediate> | number,
): WatchStopHandle;

export function watchThrottled<T extends object, Immediate extends Readonly<boolean> = false>(
    source: T,
    cb: WatchCallback<T, Immediate extends true ? T | undefined : T>,
    options?: WatchThrottledOptions<Immediate> | number,
): WatchStopHandle;

export function watchThrottled(
    source: any,
    cb: any,
    options?: WatchThrottledOptions<any> | number,
): WatchStopHandle {
    return watchInner(source, cb, options, true, false, "throttle");
}
