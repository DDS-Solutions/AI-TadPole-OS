/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[sanitize_error]` in observability traces.
 *
 * @security-scan:file-ignore
 * Reason: All regex patterns are static string sanitizers (connection strings,
 * file paths, stack traces). No dynamic eval or DOM manipulation occurs here.
 * Patterns that match scanner rules (e.g. URL-shaped regexes) are intentional
 * and safe — they scrub outbound error details, not execute user input.
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

// Metadata: [sanitize_error]
