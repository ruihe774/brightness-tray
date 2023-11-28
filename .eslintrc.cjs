require("@rushstack/eslint-patch/modern-module-resolution")

module.exports = {
    extends: [
        "eslint:recommended",
        "plugin:vue/vue3-recommended",
        "@vue/eslint-config-typescript/recommended",
        "@vue/eslint-config-prettier",
    ],
    rules: {
        "no-empty": ["error", { allowEmptyCatch: true }],
        "@typescript-eslint/explicit-function-return-type": ["error", { allowExpressions: true }],
    }
}
