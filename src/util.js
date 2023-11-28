import { watch } from "vue";
import { debounce } from "lodash-es";

export function watchDelayed(source, cb, options) {
    if (typeof options != "object") {
        options = {
            delay: options,
        };
    }
    options = {
        leading: false,
        trailing: true,
        ...options,
    };
    const debounceOptions = {
        leading: options.leading,
        trailing: options.trailing,
        maxWait: options.delay,
    };
    delete options.leading;
    delete options.trailing;
    delete options.delay;
    return watch(source, debounce(cb, debounceOptions.maxWait, debounceOptions), options);
}
