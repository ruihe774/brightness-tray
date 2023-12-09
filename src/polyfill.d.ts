interface Math {
    clamp(x: number, lower: number, upper: number): number;
}

interface Uint8Array {
    toBase64(options?: { alphabet?: "base64" | "base64url" }): string;
}
