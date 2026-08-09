/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[scrub]` in observability traces.
 */

export function scrub_string(str: string): string {
    return str
        .replace(/sk-[a-zA-Z0-9-_]{12,}/g, '[REDACTED]')
        .replace(/Bearer\s+[a-zA-Z0-9-_.]+/gi, 'Bearer [REDACTED]');
}

export function isFormData(val: unknown): val is FormData {
    if (!val || typeof val !== 'object') {
        return false;
    }
    const obj = val as Record<string, unknown>;
    return (
        val instanceof FormData ||
        obj.constructor?.name === 'FormData' ||
        (typeof obj.append === 'function' && typeof obj.entries === 'function')
    );
}

export function scrub_secrets(body: unknown): unknown {
    if (body === null || body === undefined) {
        return body;
    }
    try {
        if (typeof body === 'string') {
            if (body.length > 65536) {
                return `${scrub_string(body.substring(0, 1024))}... [TELEMETRY BODY OVERFLOW: LARGE PAYLOAD REDACTED]`;
            }
            try {
                const parsed = JSON.parse(body);
                const cloned = typeof structuredClone !== 'undefined' ? structuredClone(parsed) : JSON.parse(JSON.stringify(parsed));
                const scrubbed = scrub_secrets_object(cloned);
                return JSON.stringify(scrubbed);
            } catch {
                return scrub_string(body);
            }
        }
        if (
            body instanceof Blob ||
            body instanceof ArrayBuffer ||
            (typeof ArrayBuffer !== 'undefined' && ArrayBuffer.isView(body))
        ) {
            return '[BINARY DATA]';
        }
        if (isFormData(body)) {
            const scrubbed = new FormData();
            for (const [key, val] of body.entries()) {
                if (/^(key|token|secret|password|auth|bearer)$/i.test(key) || /\b(api_key|authorization|token|apiKey|access_token)\b/i.test(key)) {
                    scrubbed.append(key, '[REDACTED]');
                } else if (typeof val === 'string') {
                    if (val.length > 65536) {
                        scrubbed.append(key, `${scrub_string(val.substring(0, 1024))}... [LARGE KEY VALUE TRUNCATED]`);
                    } else {
                        scrubbed.append(key, scrub_string(val));
                    }
                } else if (val instanceof Blob) {
                    scrubbed.append(key, `[FILE: ${val.name || 'blob'} (${val.size} bytes)]`);
                } else {
                    scrubbed.append(key, val);
                }
            }
            return scrubbed;
        }
        if (typeof body === 'object') {
            let serialized: string;
            try {
                serialized = JSON.stringify(body);
            } catch {
                return '[UNSCRUBBABLE: Circular/Function]';
            }
            if (serialized.length > 65536) {
                return `${serialized.substring(0, 1024)}... [TELEMETRY BODY OVERFLOW: LARGE OBJECT REDACTED]`;
            }
            const parsed = JSON.parse(serialized);
            const scrubbed = scrub_secrets_object(parsed);
            return scrubbed;
        }
    } catch {
        return '[UNSCRUBBABLE: Circular/Function]';
    }
    return body;
}

export function is_sensitive_key(key: string): boolean {
    const sensitive = /^(key|token|secret|password|auth|authorization|cookie|jwt|bearer)s?$/i;
    if (sensitive.test(key)) {
        return true;
    }
    if (/(?:_|-)(key|token|secret|password|auth|authorization|cookie|jwt|bearer)s?$/i.test(key)) {
        return true;
    }
    if (/^(key|token|secret|password|auth|authorization|cookie|jwt|bearer)s?(?:_|-)/i.test(key)) {
        return true;
    }
    const camelMatch = key.match(/[a-z](Key|Token|Secret|Password|Auth|Authorization|Cookie|Jwt|Bearer)s?$/);
    if (camelMatch) {
        return true;
    }
    return false;
}

export function scrub_secrets_object(obj: unknown): unknown {
    if (obj === null || obj === undefined) {
        return obj;
    }
    if (Array.isArray(obj)) {
        return obj.map(item => scrub_secrets_object(item));
    }
    if (typeof obj === 'object') {
        const record = obj as Record<string, unknown>;
        for (const key of Object.keys(record)) {
            const val = record[key];
            if (is_sensitive_key(key)) {
                record[key] = '[REDACTED]';
            } else if (typeof val === 'string') {
                record[key] = scrub_string(val);
            } else if (typeof val === 'object') {
                record[key] = scrub_secrets_object(val);
            }
        }
    }
    return obj;
}

// Metadata: [scrub]
