/**
 * @docs ARCHITECTURE:Observability
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utilities / telemetry
 * - **Primary Entrypoints**: `track_operation`, `track_agent_slot_swap`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { event_bus } from '../services/event_bus';
import { scrub_string } from '../services/base_api_service';

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

        // 🛡️ [Observability] Suppress noise from aborted requests (component unmounts/navigation)
        const is_abort = error instanceof Error && (
            error.name === 'AbortError' || 
            error.message.includes('aborted') || 
            error.message === 'signal is aborted without reason'
        );

        if (is_abort) {
            throw error;
        }

        const raw_error = error instanceof Error ? error.message : String(error);
        const error_message = scrub_string(raw_error);
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

/**
 * Emits a structured telemetry log event when an agent switches operational model slots.
 */
export function track_agent_slot_swap(
    agent_id: string,
    agent_name: string,
    from_slot: number,
    to_slot: number,
    model_name: string
): void {
    event_bus.emit_log({
        source: 'System',
        text: `🔀 [AGENT_SLOT_SWAPPED] Agent "${agent_name}" (${agent_id}) swapped operational slot: Slot ${from_slot} → Slot ${to_slot} (${model_name})`,
        severity: 'info',
        agent_id,
        metadata: {
            from_slot,
            to_slot,
            model_name,
            phase: 'mode_transition'
        }
    });
}
