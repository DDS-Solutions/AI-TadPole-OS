/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Intelligence / authUtils
 * - **Primary Entrypoints**: `get_session_role`, `is_user_authorized`, `redact_sensitive_info`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export const get_session_role = (): string => {
    if (typeof window === 'undefined') return 'Cognitive Architect';
    return localStorage.getItem('tadpole_session_role') || 'Cognitive Architect';
};

/**
 * Checks if the user is authorized to modify memory.
 */
export const is_user_authorized = (): boolean => {
    const role = get_session_role();
    return role === 'Cognitive Architect' || role === 'Admin';
};

/**
 * Redacts potentially sensitive system user paths or identifiers from logs/anomalies.
 */
export const redact_sensitive_info = (text: string): string => {
    if (!text) return '';
    // Redact standard Windows user profile paths
    let redacted = text.replace(/[cC]:\\Users\\[^\\]+/gi, 'C:\\Users\\<Redacted>');
    // Redact typical Linux/macOS home paths
    redacted = redacted.replace(/\/home\/[^/]+/gi, '/home/<Redacted>');
    redacted = redacted.replace(/\/Users\/[^/]+/gi, '/Users/<Redacted>');
    return redacted;
};
