/**
 * @docs ARCHITECTURE:Contracts
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / wire
 * - **Primary Entrypoints**: `ModelConfigDto`, `AgentDto`, `Raw_Agent_Memory_Entry`, `AgentUpdateDto`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
    planningSlot?: ModelConfigDto;
    executionSlot?: ModelConfigDto;
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
    economicZone?: string;
    dailySpendLimit?: number;
    dailySpentAccumulated?: number;
    balance?: number;
    inventory?: Array<{
        assetId: string;
        assetName: string;
        assetData?: string;
    }>;
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
