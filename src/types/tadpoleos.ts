/**
 * @docs ARCHITECTURE:Types
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / tadpoleos
 * - **Primary Entrypoints**: `Trace_Span`, `Trace_Node`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export interface Trace_Span {
    id: string;
    trace_id: string;
    parent_id?: string;
    name: string;
    agent_id: string;
    mission_id: string;
    status: 'running' | 'success' | 'error';
    start_time: number;
    end_time?: number;
    last_activity_at?: number;
    timeout_seconds?: number;
    attributes: Record<string, string | number | boolean>;
}

export interface Trace_Node extends Trace_Span {
    children: Trace_Node[];
}
