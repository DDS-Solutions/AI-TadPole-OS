/**
 * @docs ARCHITECTURE:Types
 * 
 * ### AI Assist Note
 * **Rust-Parity Type Registry**: Strict schemas for `Tadpole_OS_Agent`, `Model_Config`, and `Swarm_Pulse`. 
 * Orchestrates the binary-to-object mapping for high-speed telemetry and OpenTelemetry trace propagation.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: `Tadpole_OS_Model_Config` mismatch (missing `api_key` or `base_url`), or pulse status enum drift.
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

export interface Paginated_Response<T> {
    data: T[];
    page: number;
    per_page: number;
    total: number;
    total_pages: number;
    _links?: {
        self: string;
        next: string | null;
        prev: string | null;
    };
}

export const Mission_Status = {
    PENDING: "pending",
    ACTIVE: "active",
    COMPLETED: "completed",
    FAILED: "failed",
    PAUSED: "paused"
} as const;

export type Mission_Status = typeof Mission_Status[keyof typeof Mission_Status];

export interface Telemetry_Metrics {
    tool_latency_p50: number;
    tool_latency_p95: number;
    tool_latency_p99: number;
    sample_count: number;
    timestamp: string;
}

export interface Tadpole_OS_Workspace {
    id: string;
    path: string;
    bootstrap_files: Bootstrap_File[];
    sandbox?: {
        scope: "session" | "agent" | "shared";
        allow_network?: boolean;
        allow_fs?: boolean;
    };
}

export interface Bootstrap_File {
    name: "AGENTS.md" | "SOUL.md" | "TOOLS.md" | "IDENTITY.md" | "USER.md" | "MEMORY.md" | string;
    content: string;
}

export interface Tadpole_OS_Channel {
    id: string;
    type: Channel_Type;
    enabled: boolean;
    dm_policy: "pairing" | "open" | "closed";
    allow_from?: string[];
    config: Record<string, unknown>;
}

export type Channel_Type = "whatsapp" | "telegram" | "slack" | "discord" | "signal" | "msteams" | "webchat" | "terminal";

export interface Tadpole_OS_Session {
    id: string;
    agent_id: string;
    channel_id: string;
    status: "active" | "archived" | "pending";
    preview?: string;
    last_message_at: string;
    context: {
        user_id: string;
        user_name?: string;
    };
}


// Metadata: [tadpoleos]

// Metadata: [tadpoleos]
