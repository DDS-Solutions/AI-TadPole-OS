/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[url]` in observability traces.
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

// Metadata: [url]
