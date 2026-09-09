/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / url
 * - **Primary Entrypoints**: `validate_and_sanitize_url`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export function validate_and_sanitize_url(url_str: string): string {
    const trimmed = url_str.trim();
    if (!trimmed) {
        throw new Error('URL is empty');
    }

    let parsed: URL;
    try {
        parsed = new URL(trimmed);
    } catch {
        throw new Error(`Invalid URL format: ${trimmed}`);
    }

    // Strip basic auth credentials
    parsed.username = '';
    parsed.password = '';

    const protocol = parsed.protocol.toLowerCase();
    const hostname = parsed.hostname.toLowerCase();

    const clean_hostname = hostname.replace(/^\[|\]$/g, '');
    const is_loopback = 
        clean_hostname === 'localhost' || 
        clean_hostname === '127.0.0.1' || 
        clean_hostname === '::1' ||
        clean_hostname.endsWith('.localhost');

    if (protocol !== 'https:' && !is_loopback) {
        throw new Error(`Insecure transmission blocked: external connection to ${hostname} must use HTTPS.`);
    }

    return parsed.toString().replace(/\/$/, '');
}
