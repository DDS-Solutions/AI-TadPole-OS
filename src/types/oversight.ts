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
    skill: string;
    description: string;
    params: Record<string, unknown>;
    timestamp: string;
}

export interface OversightEntry extends Partial<Omit<ToolCall, 'id'>> {
    id: string;
    tool_call?: ToolCall;
    decision: 'pending' | 'approved' | 'rejected';
    decided_by?: string;
    decided_at?: string;
    created_at: string;
}

export interface LedgerEntry extends Partial<Omit<ToolCall, 'id'>> {
    id: string;
    tool_call?: ToolCall;
    decision: 'approved' | 'rejected';
    result?: {
        success: boolean;
        output: string;
        error?: string;
        duration_ms: number;
    };
    timestamp: string;
}

export interface OversightResponse {
    data: OversightEntry[];
    status: string;
}

export interface LedgerResponse {
    data: LedgerEntry[];
    status: string;
}

// Metadata: [oversight]
