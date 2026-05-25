/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * Authentication and sanitization utilities for knowledge graph operations.
 */

/**
 * Gets the simulated user session role.
 * Defaults to 'Cognitive Architect' to allow local developer access.
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
    redacted = redacted.replace(/\/home\/[^\/]+/gi, '/home/<Redacted>');
    redacted = redacted.replace(/\/Users\/[^\/]+/gi, '/Users/<Redacted>');
    return redacted;
};
