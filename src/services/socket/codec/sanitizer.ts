/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Recursive key sanitization helper to strip __proto__, prototype, and constructor keys**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[sanitizer]` in observability traces.
 */

/** Recursive key sanitization helper to strip __proto__, prototype, and constructor keys */
export function sanitize_object(val: unknown): unknown {
    if (val === null || typeof val !== 'object') {
        return val;
    }
    if (Array.isArray(val)) {
        return val.map(sanitize_object);
    }
    const clean = Object.create(null);
    for (const key of Object.keys(val)) {
        if (key === '__proto__' || key === 'constructor' || key === 'prototype') {
            continue;
        }
        clean[key] = sanitize_object((val as Record<string, unknown>)[key]);
    }
    return clean;
}

// Metadata: [sanitizer]
