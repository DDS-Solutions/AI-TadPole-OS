/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / oversight
 * - **Primary Entrypoints**: `ToolCall`, `OversightEntry`, `LedgerEntry`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export interface ToolCall {
    id: string;
    agent_id: string;
    cluster_id?: string;
    mission_id?: string;
    skill: string;
    description: string;
    params: Record<string, unknown>;
    timestamp: string;
}

export interface OversightEntry extends Partial<Omit<ToolCall, 'id'>> {
    id: string;
    tool_call?: ToolCall;
    decision: 'pending' | 'approved' | 'rejected' | 'auto_approved';
    decided_by?: string;
    decided_at?: string;
    created_at: string;
    mission_id?: string;
    cluster_id?: string;
}

export interface LedgerEntry extends Partial<Omit<ToolCall, 'id'>> {
    id: string;
    tool_call?: ToolCall;
    decision: 'approved' | 'rejected' | 'auto_approved';
    decided_by?: string;
    auto_approved?: boolean;
    approval_type?: 'hitl' | 'auto';
    requires_oversight?: boolean;
    result?: {
        success: boolean;
        output: string;
        error?: string;
        duration_ms: number;
    };
    timestamp: string;
    created_at?: string;
    decided_at?: string;
    mission_id?: string;
    cluster_id?: string;
}
