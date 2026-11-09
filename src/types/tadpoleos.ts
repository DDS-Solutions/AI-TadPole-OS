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
    error?: string;
    error_reason?: string;
    'error.reason'?: string;
    http_status?: number;
}

export interface Trace_Node extends Trace_Span {
    children: Trace_Node[];
}

export interface Span_Error_Details {
    error?: string;
    reason?: string;
    http_status?: number;
    stack?: string;
}

export const get_span_error_details = (span: Trace_Span): Span_Error_Details => {
    const error = span.error || (typeof span.attributes?.error === 'string' ? span.attributes.error : undefined);
    const reason = span.error_reason || span['error.reason'] || (typeof span.attributes?.['error.reason'] === 'string' ? span.attributes['error.reason'] : undefined);
    const http_status = span.http_status || (typeof span.attributes?.http_status === 'number' ? span.attributes.http_status : undefined);
    const stack = (typeof span.attributes?.['exception.stacktrace'] === 'string' ? span.attributes['exception.stacktrace'] : undefined)
        || (typeof span.attributes?.stack_trace === 'string' ? span.attributes.stack_trace : undefined)
        || (typeof span.attributes?.['error.stack'] === 'string' ? span.attributes['error.stack'] : undefined);
    return { error, reason, http_status, stack };
};
