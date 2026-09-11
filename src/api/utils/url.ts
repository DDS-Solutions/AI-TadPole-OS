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

function is_rfc1918_or_local(hostname: string): boolean {
    const clean = hostname.toLowerCase().trim();
    if (clean.endsWith('.local') || clean.endsWith('.lan') || clean.endsWith('.internal')) {
        return true;
    }
    const ipv4_match = clean.match(/^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/);
    if (ipv4_match) {
        const octets = ipv4_match.slice(1, 5).map(Number);
        if (octets.some(o => o < 0 || o > 255)) return false;
        const [o1, o2] = octets;
        if (o1 === 10) return true;
        if (o1 === 172 && o2 >= 16 && o2 <= 31) return true;
        if (o1 === 192 && o2 === 168) return true;
    }
    return false;
}

export function validate_and_sanitize_url(url_str: string, allow_private_network = false): string {
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

    const is_private = allow_private_network && is_rfc1918_or_local(clean_hostname);

    if (protocol !== 'https:' && !is_loopback && !is_private) {
        throw new Error(`Insecure transmission blocked: external connection to ${hostname} must use HTTPS.`);
    }

    return parsed.toString().replace(/\/$/, '');
}
