/**
 * @docs ARCHITECTURE:Types
 * 
 * ### AI Assist Note
 * **Rust-Parity Type Registry**: Strict schemas for `Tadpole_OS_Agent`, Model_Config, and Swarm_Pulse. 
 * Orchestrates the binary-to-object mapping for high-speed telemetry and OpenTelemetry trace propagation.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: `Tadpole_OS_Model_Config` mismatch (missing api_key or base_url), or pulse status enum drift.
 * - **Telemetry Link**: Search for `Tadpole_OS_Model_Config` in trace attributes.
 */

/**
 * Tadpole_OS_Model_Config
 * Core types derived from the TadpoleOS Rust backend.
 * Refactored for strict snake_case compliance for backend parity.
 */



// Consolidated Agent Contracts (Phase 2 Migration)
export type { 
    AgentDto as Tadpole_OS_Agent, 
    ModelConfigDto as Tadpole_OS_Model_Config 
} from '../contracts/agent';


// Metadata: [tadpoleos]


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
    attributes: Record<string, string | number | boolean>;
}

export interface Trace_Node extends Trace_Span {
    children: Trace_Node[];
}
