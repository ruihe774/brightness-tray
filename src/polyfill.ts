export function toBase64(array: Uint8Array | Uint8ClampedArray): string {
    const base64: string[] = [];
    const characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    const padding = array.length % 3;
    let i = 0;
    for (; i < array.length - padding; i += 3) {
        const combined = (array[i] << 16) | (array[i + 1] << 8) | array[i + 2];
        base64.push(
            characters[(combined >>> 18) & 63],
            characters[(combined >>> 12) & 63],
            characters[(combined >>> 6) & 63],
            characters[combined & 63],
        );
    }
    switch (padding) {
        case 2: {
            const combined = (array[i] << 8) | array[i + 1];
            base64.push(
                characters[(combined >>> 10) & 63],
                characters[(combined >>> 4) & 63],
                characters[(combined << 2) & 63],
                "=",
            );
            break;
        }
        case 1: {
            const combined = array[i];
            base64.push(
                characters[(combined >>> 2) & 63],
                characters[(combined << 4) & 63],
                "=",
                "=",
            );
            break;
        }
    }
    return base64.join("");
}
