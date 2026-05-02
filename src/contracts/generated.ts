/**
 * @docs ARCHITECTURE:Contracts
 * 
 * ### AI Assist Note
 * **Auto-Generated Contracts**: Authoritative TypeScript types mirrored from Rust structs via Specta.
 * This file ensures frontend-backend type parity. Do not edit manually.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Type mismatch if Rust structs drift without triggering a re-generation.
 * - **Telemetry Link**: Not tracked (Static generated types).
 */

export type EngineAgent = { identity: AgentIdentity; models: AgentModels; economics: AgentEconomics; health: AgentHealth; capabilities: AgentCapabilities; state: AgentState; metadata: Partial<{ [key in string]: JsonValue }>; created_at: string | null; requires_oversight: boolean; voice_id: string | null; voice_engine: string | null; connector_configs: ConnectorConfig[]; version: number }

export type AgentIdentity = { id: string; name: string; role: string; department: string; description: string; category: string; themeColor: string | null }

export type AgentModels = { modelId: string | null; model: ModelConfig; model2: string | null; model3: string | null; modelConfig2: ModelConfig | null; modelConfig3: ModelConfig | null; activeModelSlot: number | null }

export type AgentEconomics = { budgetUsd: number; costUsd: number; tokensUsed: number; tokenUsage: TokenUsage }

export type AgentHealth = { status: string; failureCount: number; lastFailureAt: string | null; heartbeatAt: string | null }

export type AgentCapabilities = { skills: string[]; workflows: string[]; mcpTools: string[]; skillManifest: SkillManifest | null }

export type AgentState = { activeMission: JsonValue | null; currentTask: string | null; workingMemory: JsonValue; currentReasoningTurn: number }

export type ModelConfig = { provider: ModelProvider; model_id?: string; api_key?: string | null; base_url?: string | null; system_prompt?: string | null; temperature: number | null; max_tokens?: number | null; external_id?: string | null; rpm?: number | null; rpd?: number | null; tpm?: number | null; tpd?: number | null; skills?: string[] | null; workflows?: string[] | null; mcp_tools?: string[] | null; steering_vectors?: string[] | null; reasoning_depth?: number | null; act_threshold?: number | null; max_turns?: number | null; connector_configs?: ConnectorConfig[] | null; extra_parameters?: Partial<{ [key in string]: JsonValue }> | null }

export type ConnectorConfig = { type: string; uri: string }

export type TokenUsage = { input_tokens?: number; output_tokens?: number; total_tokens?: number }

/**
 * ### 📡 Protocol: ModelProvider
 * Defines the set of supported LLM backend protocols for the Tadpole OS engine.
 */
export type ModelProvider = "openai" | "anthropic" | "google" | "gemini" | "ollama" | "groq" | "mistral" | "perplexity" | "fireworks" | "together" | "deepseek" | "xai" | "inception" | "openrouter" | "cerebras" | "sambanova"

export type RoleBlueprint = { id: string; name: string; department: string; description: string; skills?: string; workflows?: string; mcpTools?: string; requiresOversight?: boolean; modelId?: string | null; createdAt?: string | null }

export type AgentConfigUpdate = { name: string | null; role: string | null; department: string | null; provider: ModelProvider | null; modelId: string | null; modelConfig?: ModelConfig | null; model2?: string | null; model3?: string | null; apiKey: string | null; systemPrompt: string | null; temperature: number | null; baseUrl: string | null; reasoningDepth: number | null; actThreshold: number | null; themeColor: string | null; budgetUsd: number | null; externalId: string | null; skills: string[] | null; workflows: string[] | null; mcpTools: string[] | null; activeModelSlot: number | null; modelConfig2: ModelConfig | null; modelConfig3: ModelConfig | null; voiceId: string | null; voiceEngine: string | null; category: string | null; requiresOversight: boolean | null; connectorConfigs: ConnectorConfig[] | null; inputTokens: number | null; outputTokens: number | null; totalTokens: number | null; tokensUsed: number | null; createdAt: string | null; lastPulse: string | null; currentTask: string | null; metadata: Partial<{ [key in string]: JsonValue }> | null }

export type SkillManifest = { schema_version: string; name: string; display_name: string | null; description: string; version: string; author: string | null; permissions?: Permission[]; toolset_group: string | null; danger_level: DangerLevel; requires_oversight?: boolean; parameters?: Partial<{ [key in string]: SkillParameter }>; hooks: SkillHooks | null; category?: string }

export type DangerLevel = "low" | "medium" | "high" | "critical"

export type Permission = "network:outbound" | "filesystem:read" | "filesystem:write" | "shell:execute" | "budget:spend" | { unknown: string }

export type SkillParameter = { type: string; required: boolean | null; default: JsonValue | null }

export type SkillHooks = { before_execute: string | null; after_execute: string | null }

export type JsonValue = string | number | boolean | null | { [key: string]: JsonValue } | JsonValue[];

// Metadata: [generated]

// Metadata: [generated]
