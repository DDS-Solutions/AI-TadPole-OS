/**
 * @docs ARCHITECTURE:UI-Services:Resilience
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / Resilience / Hex Utils
 * - **Primary Entrypoints**: `generate_hex_id`
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

/**
 * Generates a hex string of specified byte length using cryptographic entropy when available.
 */
export function generate_hex_id(bytes: number): string {
    const array = new Uint8Array(bytes);
    const cryptoObj = typeof crypto !== 'undefined' ? crypto : (typeof globalThis !== 'undefined' && globalThis.crypto ? globalThis.crypto : null);
    if (cryptoObj && cryptoObj.getRandomValues) {
        cryptoObj.getRandomValues(array);
    } else {
        // ⚠️ TELEMETRY ONLY: This fallback is for trace/span IDs exclusively.
        // It is intentionally NOT used for security tokens, session IDs, or resource keys.
        // In all supported environments (Browser, Tauri WebView) crypto.getRandomValues is available;
        // this branch exists only for obscure test/SSR environments.
        for (let i = 0; i < bytes; i++) {
            array[i] = Math.floor(Math.random() * 256);
        }
    }
    return Array.from(array)
        .map(b => b.toString(16).padStart(2, '0'))
        .join('');
}
