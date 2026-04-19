/**
 * @docs ARCHITECTURE:Services
 * 
 * ### AI Assist Note
 * **Core functional element for the Tadpole OS engine.**
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path: Runtime logic error or state corruption.**
 * - **Telemetry Link**: Search `[system_utils.ts]` in tracing logs.
 */

/**
 * @docs ARCHITECTURE:DiagnosticUtilities
 *
 * ### AI Assist Note
 * **Root Log Aggregator**: Standardizes error propagation across the entire Tadpole OS.
 * Surfaces failures to the `event_bus` with full diagnostic metadata, including stack traces and object state.
 */

import { event_bus } from './event_bus';

/**
 * Standard levels for system telemetry.
 */
export type Log_Severity = 'error' | 'warning' | 'info' | 'success';

/**
 * Safely logs an error with full diagnostic metadata to the system log stream.
 * 
 * @param source - The subsystem or component generating the error (e.g., 'AgentStore').
 * @param message - High-level description of what failed.
 * @param error - The actual error object or details.
 * @param severity - Classification for UI/UX alerting.
 */
export const log_error = (
    source: string, 
    message: string, 
    error: unknown, 
    severity: Log_Severity = 'error'
) => {
    // 1. Silent Abort Guard: Discard normal fetch cancellations to prevent log pollution.
    if (error instanceof Error && (error.name === 'AbortError' || error.name === 'CanceledError')) {
        return;
    }
    // Also catch plain objects that might be passed from some fetch implementations
    if (typeof error === 'object' && error !== null && ('name' in error) && ((error as { name?: string }).name === 'AbortError' || (error as { name?: string }).name === 'CanceledError')) {
        return;
    }

    let full_message = `[${source}] ${message}`;

    if (error instanceof Error) {
        full_message += `\nERROR DETAIL: ${error.message}\nSTACK TRACE: ${error.stack}`;
    } else if (typeof error === 'object' && error !== null) {
        try {
            full_message += `\nERROR OBJECT: ${JSON.stringify(error, null, 2)}`;
        } catch {
            full_message += `\nERROR OBJECT: [Unserializable]`;
        }
    } else {
        full_message += `\nUNKNOWN ERROR: ${String(error)}`;
    }

    // Emit to global event bus for the Oversight Dashboard and UI notifications
    event_bus.emit_log({
        source: 'System',
        text: full_message,
        severity
    });
};
