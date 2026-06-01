/**
 * @docs ARCHITECTURE:Contracts
 * 
 * ### AI Assist Note
 * **Wire Layer (DTO)**: Authoritative TypeScript representation of the backend JSON.
 * Strictly aligned with `server-rs/src/agent/types.rs` camelCase serialization.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Serialization mismatch between frontend camelCase (DTO) and backend snake_case (Serde).
 * - **Telemetry Link**: Not tracked (Static Wire DTO).
 */

import type { Agent_Connector_Config } from './shared';

export interface ModelConfigDto {
    provider: string;
    modelId: string;
    apiKey?: string;
    baseUrl?: string;
    systemPrompt?: string;
    temperature?: number;
    maxTokens?: number;
    externalId?: string;
    rpm?: number;
    rpd?: number;
    tpm?: number;
    tpd?: number;
    skills?: string[];
    workflows?: string[];
    mcpTools?: string[];
    reasoningDepth?: number;
    actThreshold?: number;
    connectorConfigs?: Agent_Connector_Config[];
    extraParameters?: Record<string, unknown>;
}

export interface AgentDto {
    id: string;
    name: string;
    role?: string;
    department?: string;
    description?: string;
    provider?: string;
    status?: string;
    tokensUsed?: number;
    budgetUsd?: number;
    costUsd?: number;
    tokenUsage?: {
        inputTokens?: number;
        outputTokens?: number;
        totalTokens?: number;
    };
    skills?: string | string[];
    workflows?: string | string[];
    mcpTools?: string | string[];
    themeColor?: string;
    requiresOversight?: boolean;
    modelId?: string;
    model?: string;
    modelConfig?: ModelConfigDto;
    model2?: string;
    model3?: string;
    modelConfig2?: ModelConfigDto;
    modelConfig3?: ModelConfigDto;
    activeModelSlot?: number;
    failureCount?: number;
    lastFailureAt?: string;
    createdAt?: string;
    lastPulse?: string | null; // Aliased to heartbeatAt in Rust
    currentTask?: string | null;
    connectorConfigs?: Agent_Connector_Config[];
    metadata?: Record<string, unknown>;
    voiceId?: string;
    voiceEngine?: string;
    sttEngine?: string;
    category?: string;
    currentReasoningTurn?: number;
    reasoningDepth?: number;
    workspace?: string;
}

export type AgentUpdateDto = Partial<AgentDto>;

export interface Raw_Agent_Memory_Entry {
    id: string;
    text?: string;
    content?: string;
    mission_id?: string;
    timestamp?: number | string;
    metadata?: Record<string, unknown>;
}

// Metadata: [wire]

// Metadata: [wire]
