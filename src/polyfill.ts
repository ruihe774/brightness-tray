export function toBase64(array: Uint8Array | Uint8ClampedArray): string {
    let base64 = "";
    const characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    const padding = array.length % 3;

    for (let i = 0; i < array.length - padding; i += 3) {
        const combined = (array[i] << 16) | (array[i + 1] << 8) | array[i + 2];
        base64 +=
            characters[(combined >> 18) & 63] +
            characters[(combined >> 12) & 63] +
            characters[(combined >> 6) & 63] +
            characters[combined & 63];
    }

    if (padding === 2) {
        const combined = (array[array.length - 2] << 8) | array[array.length - 1];
        base64 +=
            characters[(combined >> 10) & 63] +
            characters[(combined >> 4) & 63] +
            characters[(combined << 2) & 63] +
            "=";
    } else if (padding === 1) {
        const combined = array[array.length - 1];
        base64 += characters[(combined >> 2) & 63] + characters[(combined << 4) & 63] + "==";
    }

    return base64;
}
