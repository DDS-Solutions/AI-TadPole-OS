/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:Types:Oversight**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[oversight]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:Types:Oversight
 * 
 * ### Oversight Domain Types
 * Defines the structure for human-in-the-loop (HITL) actions and decision 
 * recording within the TadpoleOS engine.
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

// Metadata: [oversight]
