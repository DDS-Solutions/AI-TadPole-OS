/**
 * TadpoleOS Core Types
 * Derived from TadpoleO./TadpoleOS repository
 * 
 * These types represent the strict schema expected by the TadpoleOS backend.
 */

export interface PaginatedResponse<T> {
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

export interface TadpoleOSAgent {
    id: string;           // Unique identifier for the agent
    name: string;        // Display name
    role?: string;        // Top-level role from Rust
    department?: string;  // Top-level department from Rust
    description?: string; // Agent persona/description
    workspace?: string;   // Path to the agent's dedicated workspace directory
    default?: boolean;    // Whether this is the default agent for new sessions
    model?: string;           // Primary model ID lookup (Mapped from Rust model)
    modelConfig?: TadpoleOSModelConfig; // Full primary config (Mapped from Rust modelConfig)
    skills?: string[];    // Allowlist of skill IDs
    status?: AgentStatus; // Inferred runtime state
    themeColor?: string;  // UI Extension: Custom HEX color
    metadata?: {
        role?: string;
        department?: string;
        [key: string]: unknown;
    }; // Flexible metadata for UI extension
    budgetUsd?: number;   // Primary budget limit
    costUsd?: number;     // Accumulated cost
    tokensUsed?: number;  // Accumulated tokens (CamelCase for JS)
    tokens_used?: number; // Accumulated tokens (Legacy snake_case fallback)
    tokenUsage?: {
        inputTokens?: number;
        outputTokens?: number;
        totalTokens?: number;
    };
    model2?: string;
    model3?: string;
    modelConfig2?: TadpoleOSModelConfig;
    modelConfig3?: TadpoleOSModelConfig;
    activeModelSlot?: number;
    failureCount?: number;
    lastFailureAt?: string;
    requiresOversight?: boolean;
    connectorConfigs?: { type: string; uri: string }[];
    category?: string;
    createdAt?: string;
}

export interface TadpoleOSModelConfig {
    modelId: string;      // e.g., "claude-3-5-sonnet", "gpt-4o"
    provider: string;     // e.g., "anthropic", "openai", "ollama"
    temperature?: number;
    systemPrompt?: string; // The system-level instructions defining agent behavior
    apiKey?: string;      // Optional override
    baseUrl?: string;     // For local LLMs
    skills?: string[];    // Per-model skill assignments
    workflows?: string[]; // Per-model workflow assignments
    rpm?: number;
    rpd?: number;
    tpm?: number;
    tpd?: number;
    maxTokens?: number;
    externalId?: string;
    extraParameters?: Record<string, unknown>;
    connectorConfigs?: { type: string; uri: string }[];
}

export type AgentStatus = "online" | "offline" | "busy" | "working" | "paused" | "idle";

export const MissionStatus = {
    PENDING: "pending",
    ACTIVE: "active",
    COMPLETED: "completed",
    FAILED: "failed",
    PAUSED: "paused"
} as const;

export type MissionStatus = typeof MissionStatus[keyof typeof MissionStatus];

export interface TelemetryMetrics {
    tool_latency_p50: number;
    tool_latency_p95: number;
    tool_latency_p99: number;
    sample_count: number;
    timestamp: string;
}

export interface TadpoleOSWorkspace {
    id: string;           // Usually maps 1:1 with Agent ID
    path: string;         // Absolute path to the sandbox on disk
    bootstrapFiles: BootstrapFile[];
    sandbox?: {
        scope: "session" | "agent" | "shared";
        allowNetwork?: boolean;
        allowFs?: boolean;
    };
}

export interface BootstrapFile {
    name: "AGENTS.md" | "SOUL.md" | "TOOLS.md" | "IDENTITY.md" | "USER.md" | "MEMORY.md" | string;
    content: string;
}

export interface TadpoleOSChannel {
    id: string;           // Unique channel ID (e.g., "discord-prod")
    type: ChannelType;
    enabled: boolean;
    dmPolicy: "pairing" | "open" | "closed";
    allowFrom?: string[];
    config: Record<string, unknown>;
}

export type ChannelType = "whatsapp" | "telegram" | "slack" | "discord" | "signal" | "msteams" | "webchat" | "terminal";

export interface TadpoleOSSession {
    id: string;           // Session/Task ID
    agentId: string;      // The agent assigned to this task
    channelId: string;    // The channel where the task originated
    status: "active" | "archived" | "pending";
    preview?: string;     // Short snippet of the last action/message
    lastMessageAt: string; // ISO timestamp
    context: {
        userId: string;
        userName?: string;
    };
}


