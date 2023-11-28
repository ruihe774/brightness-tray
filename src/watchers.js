import { watch } from "vue";
import { debounce } from "lodash-es";

function createWatchOptions(options, leadingDefault, trailingDefault, renameKey) {
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
    return { debounceOptions, watchOptions: options };
}

export function watchDelayed(source, cb, options) {
    const { debounceOptions, watchOptions } = createWatchOptions(options, false, true, "delay");
    return watch(source, debounce(cb, debounceOptions.maxWait, debounceOptions), watchOptions);
}

export function watchThrottled(source, cb, options) {
    const { debounceOptions, watchOptions } = createWatchOptions(options, true, false, "throttle");
    return watch(source, debounce(cb, debounceOptions.maxWait, debounceOptions), watchOptions);
}
