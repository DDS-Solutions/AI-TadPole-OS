# Tadpole OS Agent Contract Specification
/**
 * @docs ARCHITECTURE:Contracts
 * 
 * ### AI Assist Note
 * **Contract Specification**: Master reference for field mapping between 
 * Rust (Wire), TypeScript (Domain), and UI (Form). 
 * This document is the source of truth for normalization and serialization logic.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Discrepancy between this spec and the actual implementation in `normalizers.ts` or `persistence.rs`.
 * - **Telemetry Link**: Not tracked (Architectural Document).
 */


This document serves as the authoritative mapping for all Agent-related fields across the Tadpole OS stack. 

## Architectural Tiers

1.  **Wire (AgentDto)**: The JSON representation exchanged with the Rust backend.
2.  **Domain (Agent)**: The normalized model used by frontend stores and business logic.
3.  **Form (AgentFormState)**: The UI-optimized state for the configuration panel.

---

## Authoritative Field Matrix

| Field | Wire (AgentDto) | Domain (Agent) | Form (AgentFormState) | Notes / Mapping Rules |
| :--- | :--- | :--- | :--- | :--- |
| **ID** | `id` | `id` | - | Primary Key. UUID string. |
| **Name** | `name` | `name` | `identity.name` | |
| **Role** | `role` | `role` | `identity.role` | |
| **Department** | `department` | `department` | `identity.department` | Map 'QA' string to 'Quality Assurance'. |
| **Description** | `description` | `description` | - | Default: "AI Agent Node". |
| **Status** | `status` | `status` | - | Map 'working' -> 'active'. |
| **Model (Primary)** | `modelId` / `model` | `model` | `slots.primary.model` | |
| **Model Config (1)** | `modelConfig` | `model_config` | `slots.primary` | Nested object mapping to `ModelConfigDto`. |
| **Model (2)** | `model2` | `model_2` | `slots.secondary.model` | |
| **Model Config (2)** | `modelConfig2` | `model_config2` | `slots.secondary` | |
| **Model (3)** | `model3` | `model_3` | `slots.tertiary.model` | |
| **Model Config (3)** | `modelConfig3` | `model_config3` | `slots.tertiary` | |
| **Active Slot** | `activeModelSlot` | `active__slot_id` | `active_tab` | Map 1->primary, 2->secondary, 3->tertiary. |
| **Skills** | `skills` | `skills` | `slots.*.skills` | DTO can be JSON string or array. Norm to array. |
| **Workflows** | `workflows` | `workflows` | `slots.*.workflows` | DTO can be JSON string or array. Norm to array. |
| **MCP Tools** | `mcpTools` | `mcp_tools` | `mcp_tools` | DTO can be JSON string or array. Norm to array. |
| **Budget (USD)** | `budgetUsd` | `budget_usd` | `governance.budget_usd` | |
| **Cost (USD)** | `costUsd` | `cost_usd` | - | Read-only from backend. |
| **Input Tokens** | `tokenUsage.inputTokens` | `input_tokens` | - | |
| **Output Tokens** | `tokenUsage.outputTokens` | `output_tokens` | - | |
| **Total Tokens** | `tokensUsed` | `tokens_used` | - | |
| **Metadata** | `metadata` | `metadata` | - | Catch-all property bag. |
| **Theme Color** | `themeColor` | `theme_color` | `ui.theme_color` | |
| **Voice ID** | `voiceId` | `voice_id` | `voice.voice_id` | |
| **Voice Engine** | `voiceEngine` | `voice_engine` | `voice.voice_engine` | |
| **STT Engine** | `sttEngine` | `stt_engine` | `voice.stt_engine` | Also stored in metadata for some agents. |
| **Category** | `category` | `category` | - | Default: 'user'. |
| **Created At** | `createdAt` | `created_at` | - | |
| **Last Pulse** | `lastPulse` | `last_pulse` | - | Aliased as `heartbeat_at` in Rust. |
| **Oversight** | `requiresOversight` | `requires_oversight` | `governance.requires_oversight` | |
| **Connectors** | `connectorConfigs` | `connector_configs` | `connector_configs` | |
| **Task** | `currentTask` | `current_task` | - | |
| **Failure Count** | `failureCount` | `failure_count` | - | |
| **Reasoning Turn** | `currentReasoningTurn` | `current_reasoning_turn`| - | |
| **Reasoning Depth**| `reasoningDepth` | `reasoning_depth` | `slots.*.reasoning_depth` | |
| **Permissions** | `permissions` | `permissions` | - | Array of `Permission` objects. |
| **Danger Level** | `dangerLevel` | `danger_level` | - | Enum: `Safe`, `Caution`, `High`. |

---

## Skill & Manifest Contracts

The following types are exported from `server-rs/src/agent/skill_manifest.rs` via the **Specta Bridge** to ensure exact parity with the frontend skill registry.

### 🛡️ SkillManifest
Master manifest for all agent capabilities (Skills, Hooks, Workflows).
- `name`: Unique identifier.
- `permissions`: Required system permissions.
- `dangerLevel`: Risk classification for the Oversight Gate.
- `hooks`: Event-driven pulse triggers.

### 🔌 SkillParameter
Input schema for skill execution.
- `name`: Parameter key.
- `required`: Boolean flag.
- `defaultValue`: Optional fallback value.

---

## Ownership Matrix

| Area | Primary Owner | Secondary |
| :--- | :--- | :--- |
| **DTO Shape** | `server-rs/src/agent/types.rs` | `src/contracts/agent/wire.ts` |
| **Domain Logic** | `src/domain/agents/normalizers.ts`| `src/contracts/agent/domain.ts` |
| **Form Logic** | `src/domain/agents/form_state.ts` | `src/contracts/agent/form.ts` |
| **Merge Logic** | `server-rs/src/agent/merge.rs` | N/A |
| **Persistence** | `server-rs/src/agent/persistence.rs`| N/A |

---

## Deprecation & Migration List

- `Tadpole_OS_Agent` (in `tadpoleos.ts`) -> Replace with `AgentDto`.
- `Raw_Agent` (in `agent_mappers.ts`) -> Replace with `AgentDto`.
- `Agent_Override` (in `agent_service.ts`) -> Replace with `Partial<Agent>`.
- `Agent_Config` (in `index.ts`) -> Replace with `AgentUpdateDto` (Wire) or `Partial<Agent>` (Domain).

[//]: # (Metadata: [agent_contract_spec])

[//]: # (Metadata: [agent_contract_spec])
