import { watchThrottled } from "@vueuse/core"

export function watchDelayed(source, cb, options) {
    if (typeof options == "number") {
        options = {
            throttle: options,
        }
    }
    options = options ?? {}
    options.leading = false
    options.trailing = true
    return watchThrottled(source, cb, options)
}
