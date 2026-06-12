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
 * Severities allowed for error logging.
 */
export type Log_Severity = 'error' | 'warning';

/**
 * Helper to determine if an error is an intentional fetch abortion or cancellation
 * and should be ignored to prevent log pollution.
 */
const is_ignorable_error = (err: unknown): boolean => {
    if (err instanceof Error) {
        return err.name === 'AbortError' || err.name === 'CanceledError';
    }
    if (typeof err === 'object' && err !== null && 'name' in err) {
        const name = (err as Record<string, unknown>).name;
        return name === 'AbortError' || name === 'CanceledError';
    }
    return false;
};

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
): void => {
    // 1. Silent Abort Guard: Discard normal fetch cancellations.
    if (is_ignorable_error(error)) {
        return;
    }

    let detail: string;

    if (error instanceof Error) {
        detail = `\nERROR DETAIL: ${error.message}\nSTACK TRACE: ${error.stack || ''}`;
    } else if (typeof error === 'object' && error !== null) {
        try {
            // Use compact JSON serialization to optimize hot path performance
            detail = `\nERROR OBJECT: ${JSON.stringify(error)}`;
        } catch {
            detail = `\nERROR OBJECT: [Unserializable]`;
        }
    } else {
        detail = `\nUNKNOWN ERROR: ${String(error)}`;
    }

    const full_message = `[${source}] ${message}${detail}`;

    // 2. Emit to global event bus with a try-catch boundary to prevent logger failures
    try {
        event_bus.emit_log({
            source: 'System',
            text: full_message,
            severity,
            metadata: {
                subsystem: source
            }
        });
    } catch (e) {
        console.error('[log_error] Critical logging failure:', e, {
            source,
            message,
            error,
            severity
        });
    }
};

// Metadata: [system_utils]

// Metadata: [system_utils]
