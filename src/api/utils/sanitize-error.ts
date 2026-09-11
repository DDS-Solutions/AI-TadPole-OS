/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / sanitize-error
 * - **Primary Entrypoints**: `sanitize_error_detail`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { scrub_string } from './scrub';

export function sanitize_error_detail(detail: string): string {
    if (!detail) return detail;
    let sanitized = detail;

    // 1. Connection strings: e.g. postgres://user:pass@host or http://user:pass@host
    sanitized = sanitized.replace(/[a-zA-Z0-9+-.]+:\/\/[^/:\s]+:[^/:\s]+@[^\s/]+/gi, '[CONNECTION_STRING_REDACTED]');

    // 2. Absolute file paths (both POSIX and Windows directories)
    sanitized = sanitized.replace(/(?:\b[a-zA-Z]:\\|\/)(?:[^\\/\s]+[\\/])+[^\s\\/]+/gi, '[PATH_REDACTED]');

    // 3. Strip "Error:" prefix from start
    sanitized = sanitized.replace(/^Error:\s*/i, '');

    // 4. Strip "at " stack trace lines (using multiline flag to match start of any line)
    sanitized = sanitized.replace(/^\s*at\s+[^\r\n]+/gim, '');

    // 5. Scrub any remaining secrets
    sanitized = scrub_string(sanitized);

    return sanitized.trim();
}
