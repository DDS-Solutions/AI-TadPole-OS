/**
 * @docs ARCHITECTURE:Observability
 * 
 * ### AI Assist Note
 * **Telemetry Utility**: Standardized wrapper for high-fidelity observability.
 * Implements the **Observe-Call-Audit (OCA)** pattern to ensure all operations are tracked with timing and redacting privacy-sensitive metadata.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: event_bus emission failure, or recursive redaction stack overflow on extremely nested metadata.
 * - **Telemetry Link**: Search for `📡` or `✅` in UI/Backend logs to trace operation lifecycles.
 */

import { event_bus } from '../services/event_bus';

/**
 * Metadata for operational tracking.
 */
interface Operation_Context {
    agent_id?: string;
    mission_id?: string;
    type_id?: string;
    metadata?: Record<string, unknown>;
}

/** 
 * Redacts common sensitive keys from metadata to prevent token exposure. 
 */
function redact_metadata(metadata: Record<string, unknown> = {}): Record<string, unknown> {
    const sensitive_keys = [/key/i, /token/i, /secret/i, /password/i, /auth/i, /bearer/i];
    const redacted = { ...metadata };

    for (const key of Object.keys(redacted)) {
        if (sensitive_keys.some(regex => regex.test(key))) {
            redacted[key] = '[REDACTED]';
        } else if (typeof redacted[key] === 'object' && redacted[key] !== null) {
            redacted[key] = redact_metadata(redacted[key] as Record<string, unknown>);
        }
    }
    return redacted;
}

/**
 * track_operation
 * 
 * Standardized wrapper for high-fidelity observability.
 * Implementation of the Observe-Call-Audit (OCA) pattern.
 * 
 * @param source - The architecture pillar (e.g., 'AgentAPI', 'MissionAPI').
 * @param description - Human-readable description of the intent.
 * @param operation - The async function to execute.
 * @param context - Optional diagnostic context.
 */
export async function track_operation<T>(
    source: string,
    description: string,
    operation: () => Promise<T>,
    context: Operation_Context = {}
): Promise<T> {
    const start_time = Date.now();
    const safe_metadata = redact_metadata(context.metadata);

    // Observe: Log initiation
    event_bus.emit_log({
        source: 'System',
        text: `📡 [${source}] ${description}`,
        severity: 'info',
        agent_id: context.agent_id,
        mission_id: context.mission_id,
        metadata: { ...safe_metadata, phase: 'initiation' }
    });

    try {
        // Call: Execute the core logic
        const result = await operation();

        // Audit: Log success with timing
        const duration = Date.now() - start_time;
        event_bus.emit_log({
            source: 'System',
            text: `✅ [${source}] Success: ${description} (${duration}ms)`,
            severity: 'success',
            agent_id: context.agent_id,
            mission_id: context.mission_id,
            metadata: { ...safe_metadata, duration_ms: duration, phase: 'completion' }
        });

        return result;
    } catch (error) {
        // Audit: Log failure with diagnostics
        const duration = Date.now() - start_time;
        const error_message = error instanceof Error ? error.message : String(error);
        const type_id = (error as { type?: string })?.type || context.type_id || 'system::error';

        event_bus.emit_log({
            source: 'System',
            text: `❌ [${source}] Failed: ${description}. Error: ${error_message}`,
            severity: 'error',
            agent_id: context.agent_id,
            mission_id: context.mission_id,
            type_id,
            metadata: { ...safe_metadata, duration_ms: duration, phase: 'failure', error: error_message }
        });

        throw error;
    }
}
