/* eslint-disable @typescript-eslint/no-explicit-any */
declare module "*.vue" {
    import { Component, ComputedOptions, MethodOptions } from "vue";
    export default Component<any, any, any, ComputedOptions, MethodOptions>;
}
