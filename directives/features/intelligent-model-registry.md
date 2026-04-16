# Feature: Intelligent Model Registry & Capability-Aware Sync (IMR-01)

**Status:** `PLANNED`
**Priority:** P1 — High Impact
**Pillar:** Local-First Sovereignty / Zero-Friction Orchestration
**Roadmap Phase:** Phase 2

---

## 🎯 Problem Statement

The current `ModelEntry` struct and `ProviderConfig` system in Tadpole OS requires **fully manual model registration**. Users must know every model ID string, input rate limits by hand, and the engine has zero awareness of what a model is actually *capable of* — no context window size, no tool support, no vision support. This creates three hard failure modes:

1. **Silent API Errors**: An agent with a `phi3:latest` model attempts a tool call, the Ollama bridge (OpenAI-compat) rejects it with a 400 error, and the swarm stalls. The UI shows nothing actionable.
2. **Invisible Capability Gaps**: The user drags an image into Sovereign Chat — but the active model is `llama3:8b`, which doesn't support vision. The error comes from the provider, not from Tadpole OS with a friendly, actionable message.
3. **Stale Registries**: A user adds an Ollama provider, but they have to manually type in every model they've pulled. When they pull a new model via `ollama pull`, Tadpole OS never knows.

## ✅ Goals

- Automatically **discover** available models from a registered provider endpoint on demand.
- Automatically **enrich** discovered models with a capability profile (tools, vision, context window, embedding support, etc.) using a built-in Capability Matrix.
- Expose a **structured `ModelCapabilities` type** in `ModelEntry` so the frontend and agent runner can make intelligent routing decisions.
- **Save discovered models** to the existing registry, making them immediately available to agents.
- Support all provider types, with specific handling for **local/self-hosted** (Ollama, LM Studio, vLLM, llama.cpp) and cloud providers (OpenAI, Anthropic, Google, Groq, Deepseek, xAI, OpenRouter).

---

## 🏗️ Architecture

### Layer 1 — New Type: `ModelCapabilities`

Add to `server-rs/src/agent/types.rs`. This is the core truth object all downstream logic will consume.

```rust
/// Describes the functional capabilities of a specific AI model.
/// Populated at sync time from a combination of live API data and
/// the engine's built-in Capability Matrix.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCapabilities {
    /// Whether the model natively accepts tool/function declarations.
    pub supports_tools: bool,

    /// Whether the model can process image inputs.
    pub supports_vision: bool,

    /// Whether the model supports structured/JSON output modes.
    pub supports_structured_output: bool,

    /// Whether this model produces vector embeddings (not a chat model).
    pub is_embedding_model: bool,

    /// Maximum context window in tokens (prompt + completion).
    /// None = unknown.
    pub context_window: Option<u32>,

    /// Maximum tokens the model can generate in a single response.
    pub max_output_tokens: Option<u32>,

    /// Whether the model is recommended for agentic/tool-heavy tasks.
    pub is_agentic_recommended: bool,

    /// Free-form tags for UI display (e.g., "fast", "reasoning", "code").
    #[serde(default)]
    pub tags: Vec<String>,

    /// Source of truth for these capabilities:
    /// "matrix" (built-in), "api" (live from provider), "manual" (user-set).
    #[serde(default = "default_caps_source")]
    pub source: String,
}

fn default_caps_source() -> String { "matrix".to_string() }
```

**Extend `ModelEntry` to include this:**

```rust
pub struct ModelEntry {
    // ... existing fields ...

    /// Capability profile enriched at sync time.
    #[serde(default)]
    pub capabilities: ModelCapabilities,

    /// ISO timestamp of the last capability sync.
    #[serde(default)]
    pub last_synced_at: Option<DateTime<Utc>>,
}
```

---

### Layer 2 — Capability Matrix (`capability_matrix.rs`)

**New file**: `server-rs/src/agent/capability_matrix.rs`

A pure Rust module containing a large `fn enrich(model_id: &str, provider: ModelProvider) -> ModelCapabilities` function. Uses pattern matching on model ID strings and provider type to return a best-known capability profile.

This module is the single source of truth for "what can this model do?" It is updated by the engineering team as new models are released.

```rust
// Pseudocode representation of the match logic

pub fn enrich(model_id: &str, provider: ModelProvider) -> ModelCapabilities {
    let id = model_id.to_lowercase();

    // --- OPENAI FAMILY ---
    if matches!(provider, ModelProvider::Openai) || id.starts_with("gpt-") || id.starts_with("o1") || id.starts_with("o3") || id.starts_with("o4") {
        if id.contains("embed") || id.contains("embedding") {
            return embedding_caps();
        }
        if id.starts_with("o1") || id.starts_with("o3") || id.starts_with("o4") {
            return ModelCapabilities { supports_tools: true, supports_vision: id.contains("mini") || id.contains("ultra") || id == "o4-mini", context_window: Some(200_000), max_output_tokens: Some(100_000), is_agentic_recommended: true, supports_structured_output: true, tags: vec!["reasoning".into()], ..Default::default() };
        }
        if id.contains("gpt-4o") {
            return ModelCapabilities { supports_tools: true, supports_vision: true, context_window: Some(128_000), max_output_tokens: Some(16_384), is_agentic_recommended: true, supports_structured_output: true, tags: vec!["flagship".into()], ..Default::default() };
        }
        if id.contains("gpt-4-turbo") {
            return ModelCapabilities { supports_tools: true, supports_vision: true, context_window: Some(128_000), ..Default::default() };
        }
        if id.contains("gpt-4") {
             return ModelCapabilities { supports_tools: true, supports_vision: false, context_window: Some(8_192), ..Default::default() };
        }
        if id.contains("gpt-3.5") {
            return ModelCapabilities { supports_tools: true, context_window: Some(16_385), ..Default::default() };
        }
    }

    // --- ANTHROPIC FAMILY ---
    if matches!(provider, ModelProvider::Anthropic) || id.starts_with("claude-") {
        if id.contains("claude-3-5") || id.contains("claude-3.5") {
            return ModelCapabilities { supports_tools: true, supports_vision: true, context_window: Some(200_000), max_output_tokens: Some(8_192), is_agentic_recommended: true, supports_structured_output: true, tags: vec!["flagship".into()], ..Default::default() };
        }
        if id.contains("claude-3") {
           return ModelCapabilities { supports_tools: true, supports_vision: true, context_window: Some(200_000), max_output_tokens: Some(4_096), ..Default::default() };
        }
    }

    // --- GOOGLE / GEMINI FAMILY ---
    if matches!(provider, ModelProvider::Google | ModelProvider::Gemini) || id.starts_with("gemini-") {
        if id.contains("embedding") { return embedding_caps(); }
        if id.contains("2.0") || id.contains("2.5") || id.contains("pro") {
            return ModelCapabilities { supports_tools: true, supports_vision: true, context_window: Some(1_048_576), max_output_tokens: Some(65_536), is_agentic_recommended: true, supports_structured_output: true, tags: vec!["long-context".into()], ..Default::default() };
        }
        if id.contains("flash") {
            return ModelCapabilities { supports_tools: true, supports_vision: true, context_window: Some(1_000_000), max_output_tokens: Some(8_192), tags: vec!["fast".into()], ..Default::default() };
        }
    }

    // --- GROQ ---
    if matches!(provider, ModelProvider::Groq) {
        if id.contains("llama3") || id.contains("llama-3") {
            return ModelCapabilities { supports_tools: true, context_window: Some(131_072), tags: vec!["fast".into(), "groq-native".into()], ..Default::default() };
        }
        if id.contains("mixtral") {
            return ModelCapabilities { supports_tools: true, context_window: Some(32_768), ..Default::default() };
        }
    }

    // --- DEEPSEEK ---
    if matches!(provider, ModelProvider::Deepseek) || id.starts_with("deepseek-") {
        if id.contains("r1") {
            return ModelCapabilities { supports_tools: false, context_window: Some(128_000), max_output_tokens: Some(64_000), tags: vec!["reasoning".into()], ..Default::default() };
        }
        if id.contains("chat") || id.contains("v3") {
            return ModelCapabilities { supports_tools: true, context_window: Some(64_000), ..Default::default() };
        }
    }

    // --- XAI GROK ---
    if matches!(provider, ModelProvider::Xai) || id.starts_with("grok-") {
        return ModelCapabilities { supports_tools: true, supports_vision: id.contains("vision"), context_window: Some(131_072), ..Default::default() };
    }

    // --- OPENROUTER (pass-thru, enriched from OpenRouter API metadata if available) ---
    if matches!(provider, ModelProvider::Openrouter) {
        return ModelCapabilities { supports_tools: id.contains("instruct") || id.contains("chat"), ..Default::default() };
    }

    // =========================================================
    // LOCAL MODELS (Ollama, LM Studio, vLLM, llama.cpp)
    // =========================================================

    // --- LLAMA FAMILY (Meta) ---
    if id.contains("llama3") || id.contains("llama-3") {
        let ctx = if id.contains("70b") || id.contains("405b") { 131_072 } else { 8_192 };
        return ModelCapabilities { supports_tools: !id.contains("text") && !id.contains("base"), context_window: Some(ctx), tags: vec!["open-weight".into(), "meta".into()], ..Default::default() };
    }
    if id.contains("llama2") || id.contains("llama-2") {
        return ModelCapabilities { supports_tools: false, context_window: Some(4_096), tags: vec!["open-weight".into()], ..Default::default() };
    }

    // --- MICROSOFT PHI FAMILY ---
    if id.contains("phi4") || id.contains("phi-4") {
        return ModelCapabilities { supports_tools: true, context_window: Some(16_384), tags: vec!["efficient".into(), "microsoft".into()], ..Default::default() };
    }
    if id.contains("phi3") || id.contains("phi-3") {
        // Known tool-incompatible via OpenAI bridge (tracked in ModelConfig::supports_native_tools)
        return ModelCapabilities { supports_tools: false, context_window: Some(4_096), tags: vec!["efficient".into(), "microsoft".into()], ..Default::default() };
    }
    if id.contains("phi2") || id.contains("phi-2") {
        return ModelCapabilities { supports_tools: false, context_window: Some(2_048), tags: vec!["efficient".into()], ..Default::default() };
    }

    // --- MISTRAL FAMILY ---
    if id.contains("mistral") {
        if id.contains("nemo") { return ModelCapabilities { supports_tools: true, context_window: Some(128_000), ..Default::default() }; }
        if id.contains("large") { return ModelCapabilities { supports_tools: true, context_window: Some(32_768), ..Default::default() }; }
        return ModelCapabilities { supports_tools: id.contains("instruct"), context_window: Some(32_768), tags: vec!["open-weight".into()], ..Default::default() };
    }
    if id.contains("mixtral") {
        return ModelCapabilities { supports_tools: true, context_window: Some(32_768), tags: vec!["moe".into()], ..Default::default() };
    }

    // --- QWEN FAMILY (Alibaba) ---
    if id.contains("qwen") {
        let ctx = if id.contains("72b") { 131_072 } else { 32_768 };
        return ModelCapabilities { supports_tools: id.contains("chat") || id.contains("instruct"), context_window: Some(ctx), tags: vec!["qwen".into()], ..Default::default() };
    }

    // --- DEEPSEEK LOCAL (deepseek-coder, deepseek-r1 local distills) ---
    if id.contains("deepseek") {
        return ModelCapabilities { supports_tools: id.contains("chat"), context_window: Some(16_384), tags: vec!["code".into()], ..Default::default() };
    }

    // --- GOOGLE GEMMA FAMILY ---
    if id.contains("gemma") {
        return ModelCapabilities { supports_tools: false, context_window: Some(8_192), tags: vec!["google".into(), "open-weight".into()], ..Default::default() };
    }

    // --- STARCODER / CODESTRAL (code-specific) ---
    if id.contains("starcoder") || id.contains("codestral") || id.contains("codellama") {
        return ModelCapabilities { supports_tools: false, context_window: Some(16_384), tags: vec!["code".into()], ..Default::default() };
    }

    // --- EMBEDDING MODELS (local via Ollama) ---
    if id.contains("embed") || id.contains("bge-") || id.contains("nomic-embed") || id.contains("all-minilm") || id.contains("mxbai-embed") {
        return embedding_caps();
    }

    // --- WHISPER (ASR, not a chat model) ---
    if id.contains("whisper") {
        return ModelCapabilities { tags: vec!["asr".into(), "audio".into()], ..Default::default() };
    }

    // Default: Unknown model, safest conservative defaults
    ModelCapabilities::default()
}

fn embedding_caps() -> ModelCapabilities {
    ModelCapabilities {
        is_embedding_model: true,
        supports_tools: false,
        tags: vec!["embeddings".into()],
        ..Default::default()
    }
}
```

---

### Layer 3 — Sync Service (`model_sync.rs`)

**New file**: `server-rs/src/routes/model_sync.rs`

This module contains the logic for pinging a provider endpoint and importing its model list.

**Provider-specific discovery strategies:**

| Provider Type | Discovery Endpoint | Response Format | Notes |
|---|---|---|---|
| `openai` / `groq` / `deepseek` / `xai` / `inception` / `openrouter` | `GET {base_url}/models` | OpenAI `Model` list JSON | Standard bearer token auth |
| `anthropic` | Hardcoded static list | N/A | Anthropic has no public `/models` endpoint |
| `google` | `GET generativelanguage.googleapis.com/v1beta/models` | Google list format | `x-goog-api-key` header |
| `ollama` | `GET {base_url}/api/tags` | `{"models":[{"name":"...","size":...}]}` | No auth needed for local |
| `lm_studio` | `GET {base_url}/v1/models` | OpenAI format | OpenAI-compat API |
| `vllm` | `GET {base_url}/v1/models` | OpenAI format | OpenAI-compat API |
| `llama_cpp` | `GET {base_url}/v1/models` | OpenAI format | Optional API key |

**The sync flow:**
```
POST /api/providers/{id}/sync
  1. Load ProviderConfig from registry
  2. Determine strategy from provider.protocol
  3. Fetch model list from remote endpoint (with timeout: 10s)
  4. Normalize raw response -> Vec<(model_id, model_name)>
  5. For each discovered model:
     a. Run through capability_matrix::enrich(model_id, provider.protocol)
     b. Check if model already exists in registry:
        - If yes AND capabilities.source == "manual": preserve manual caps, only update timestamps
        - If yes AND capabilities.source == "matrix": update with fresh matrix data
        - If no: create new ModelEntry
  6. Upsert all models into state.registry.models
  7. Call state.save_models().await
  8. Return { synced_count, new_count, skipped_count, provider_id, models: [...] }
```

**New route handler signature:**
```rust
pub async fn sync_provider_models(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError>
```

**New route (added to router):**
```
POST /api/providers/:id/sync
```

---

### Layer 4 — Frontend Enforcement

#### 4a. TypeScript Type Update (`src/types/index.ts`)

```typescript
export interface ModelCapabilities {
  supports_tools: boolean;
  supports_vision: boolean;
  supports_structured_output: boolean;
  is_embedding_model: boolean;
  context_window: number | null;
  max_output_tokens: number | null;
  is_agentic_recommended: boolean;
  tags: string[];
  source: 'matrix' | 'api' | 'manual';
}

export interface ModelEntry {
  id: string;
  name: string;
  provider_id: string;
  provider?: string;
  rpm?: number;
  tpm?: number;
  rpd?: number;
  tpd?: number;
  modality: string;
  capabilities: ModelCapabilities;          // NEW
  last_synced_at?: string;                   // NEW
}
```

#### 4b. Model Manager UI updates (`Model_Manager.tsx`)

- **"Sync Models" button** next to each provider in the provider list. On click: calls `POST /api/providers/{id}/sync`, shows a loading spinner, then refreshes the model list.
- **Capabilities Badge Strip** on each model card: small pill badges for `🛠 Tools`, `👁 Vision`, `📐 Structured`, `🧠 Reasoning`, `📦 Embedding`.
- **Context Window display**: shown as `128K ctx` in the model card metadata row.
- **"Source" indicator**: subtle label showing `matrix`, `api`, or `manual` so users understand how the caps were determined.

#### 4c. Agent Manager enforcement (`Agent_Config_Panel.tsx`)

When a user selects a model for an agent:
- If `capabilities.supports_tools === false`: Disable the "MCP Tools" panel, show tooltip: *"This model does not support tool calling. Remove tools or select a capable model."*
- If `capabilities.supports_vision === false` AND the agent's skills include any vision/image tool: Show a warning badge.
- If `capabilities.is_embedding_model === true`: Block assignment entirely with message: *"Embedding models cannot be assigned as primary agents."*

#### 4d. Sovereign Chat enforcement (`Sovereign_Chat.tsx`)

- When an image is pasted/dropped into the chat and the active agent's model has `supports_vision === false`: Show inline warning toast rather than sending to provider.

---

## 🔧 Implementation Plan

### Sprint 1: Core Types & Matrix (Backend)
- [ ] Add `ModelCapabilities` struct to `types.rs`
- [ ] Extend `ModelEntry` with `capabilities` and `last_synced_at`
- [ ] Create `server-rs/src/agent/capability_matrix.rs` with full match logic for all providers and local models listed above
- [ ] Wire `ModelConfig::supports_native_tools()` to delegate to `ModelCapabilities.supports_tools` for consistency

### Sprint 2: Sync Service (Backend)
- [ ] Create `server-rs/src/routes/model_sync.rs`
- [ ] Implement per-provider discovery strategies (OpenAI format, Google format, Ollama tags, Anthropic static list)
- [ ] Implement upsert logic with manual-override preservation
- [ ] Register `POST /api/providers/:id/sync` route in `main.rs`
- [ ] Add `tokio::time::timeout` to all provider pings (10s max)

### Sprint 3: Frontend Types & UI
- [ ] Update `ModelEntry` and `ModelCapabilities` types in `src/types/index.ts`
- [ ] Update TS frontend type (`src/types/tadpoleos.ts` if separate)
- [ ] Add capability badges to Model Manager model cards
- [ ] Add "Sync" button to provider entries in Model Manager
- [ ] Implement `POST /api/providers/:id/sync` call in `model_store.ts`

### Sprint 4: Agent & Chat Enforcement
- [ ] Add capability guards to `Agent_Config_Panel.tsx`
- [ ] Add vision guard to `Sovereign_Chat.tsx`
- [ ] Update `ModelConfig::supports_native_tools()` to check registry caps first

### Sprint 5: Auto-Sync on Startup (Optional / Phase 2.5)
- [ ] On engine startup, for every registered provider with `auto_sync: true` in `ProviderConfig`, attempt a silent background sync
- [ ] Add `auto_sync: Option<bool>` field to `ProviderConfig`
- [ ] Emit a `models_synced` WebSocket event to trigger UI refreshes

---

## 🗂️ Files Affected

| File | Change |
|---|---|
| `server-rs/src/agent/types.rs` | ADD `ModelCapabilities` struct, EXTEND `ModelEntry` |
| `server-rs/src/agent/capability_matrix.rs` | NEW FILE |
| `server-rs/src/routes/model_sync.rs` | NEW FILE |
| `server-rs/src/routes/model_manager.rs` | ADD sync route handler call |
| `server-rs/src/main.rs` | Register new `/api/providers/:id/sync` route |
| `src/types/index.ts` | ADD `ModelCapabilities`, EXTEND `ModelEntry` |
| `src/stores/model_store.ts` | ADD `sync_provider_models()` action |
| `src/pages/Model_Manager.tsx` | ADD sync button + capability badges |
| `src/components/Agent_Config_Panel.tsx` | ADD capability guards |
| `src/components/Sovereign_Chat.tsx` | ADD vision guard |

---

## 🤔 Design Decisions & Rationale

### Why a Static Capability Matrix instead of a live registry?
Provider APIs (OpenAI, Anthropic, Ollama) **do not** expose model capabilities over the wire. The `/v1/models` endpoint returns IDs and names only. A static matrix that maps model ID patterns to known capabilities is the industry-standard approach (used by frameworks like LiteLLM, OpenRouter, and LangChain). This is the **least-latency**, **most-reliable** approach. The matrix is versioned with the codebase.

### Why preserve "manual" source capabilities?
Power users may want to set `supports_tools: false` for a model that the matrix incorrectly marks as tool-capable (e.g., a quantized variant that strops on function declarations). Manual overrides must survive re-syncs.

### Why Ollama uses `/api/tags` not `/v1/models`?
Ollama's OpenAI-compat `/v1/models` endpoint exists but returns limited data. The native `/api/tags` endpoint returns model size (`size_vram`), model digest, and other local metadata that is useful for the registry.

### Local Model Coverage Strategy
The capability matrix key insight for local models is that most are pulled via an `ollama pull <name>` command using the **tag format** (`model:variant`). Pattern matching on the name prefix covers the vast majority of cases. We will document the matrix clearly so the community can open PRs to extend it.

---

## 📊 Acceptance Criteria

- [ ] `POST /api/providers/ollama-local/sync` correctly discovers all locally-pulled Ollama models and saves them to the registry with capability data.
- [ ] A `gpt-4o` model entry has `supports_tools: true`, `supports_vision: true`, `context_window: 128000`.
- [ ] A `phi3:latest` model entry has `supports_tools: false` (consistent with existing `ModelConfig::supports_native_tools()` behavior).
- [ ] A `nomic-embed-text` model entry has `is_embedding_model: true`.
- [ ] Assigning an embedding model as an agent's primary model is blocked in the UI.
- [ ] The "Sync" button in Model Manager correctly shows a loading state during the HTTP request.
- [ ] Syncing an Anthropic provider (no live endpoint) inserts the static model list with capability data.
- [ ] Manual capability overrides (`source: "manual"`) survive a re-sync.

---

*Created: April 2026*
*Feature ID: IMR-01*
*Author: Antigravity (Tadpole OS Agent)*
