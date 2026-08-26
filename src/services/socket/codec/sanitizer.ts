/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / sanitizer
 * - **Primary Entrypoints**: `sanitize_object`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export function sanitize_object(val: unknown, depth = 0, max_depth = 50): unknown {
    if (depth > max_depth) {
        return null;
    }
    if (val === null || typeof val !== 'object') {
        return val;
    }
    if (Array.isArray(val)) {
        return val.map(item => sanitize_object(item, depth + 1, max_depth));
    }
    const clean = Object.create(null);
    for (const key of Object.keys(val)) {
        if (key === '__proto__' || key === 'constructor' || key === 'prototype') {
            continue;
        }
        clean[key] = sanitize_object((val as Record<string, unknown>)[key], depth + 1, max_depth);
    }
    return clean;
}
