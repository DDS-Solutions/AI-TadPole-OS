# 🗺️ Codebase Map (AI Navigator)

This map is designed to help AI assistants (and human developers) navigate the Tadpole OS project structure and understand component relationships.

## Backend: Rust Engine (`server-rs/`)

| Directory / File | Purpose | Key Notes |
| :--- | :--- | :--- |
| `src/agent/runner/mod.rs` | **Execution Core** | Main execution loop, Mission Analysis trigger, Drift Detection, and Semantic Pruning |
| `src/agent/runner/tools.rs` | **Tool Execution** | Parallel tool execution via `FuturesUnordered` |
| `src/agent/runner/synthesis.rs` | **Prompt Engineering** | Prompt engineering + Context pruning |
| `src/agent/runner/external_tools.rs` | **External Tools** | External service tool handlers (web, API, fetch) |
| `src/agent/runner/fs_tools.rs` | **File System Tools** | Sandboxed file I/O tool handlers |
| `src/agent/runner/mission_tools.rs` | **Mission Tools** | Mission-specific tool handlers (recruit, delegate, reflect) |
| `src/agent/runner/swarm.rs` | **Swarm Orchestration** | Multi-agent coordination, agent recruitment, and delegation |
| `src/agent/runner/provider.rs` | **Provider Routing** | Unified LLM provider resolution and API key management |
| `src/agent/memory.rs` | **Vector Memory** | LanceDB RAG Scopes, Hybrid RAG (vector + keyword), Heuristic Reranker, Gemini Embeddings, Deduplication |
| `src/agent/hooks.rs` | **Lifecycle Hooks** | `HooksManager` for `pre-tool` and `post-tool` auditing |
| `src/agent/gemini.rs` | **Google Provider** | Concurrent tool call support via `generate` |
| `src/agent/groq.rs` | **Groq Provider** | Shared client + Llama tool-call recovery; unused struct fields removed |
| `src/agent/openai.rs` | **OpenAI Provider** | Standard integration for GPT-4o and O1 models |
| `src/agent/rate_limiter.rs` | **API Quota Guard** | `Semaphore`-based RPM + `AtomicU32` TPM; auto-enforced in `call_provider` |
| `src/agent/mission.rs` | **Mission CRUD** | `row_to_mission()` helper eliminates 3× DRY violation; `str_to_status()` |
| `src/agent/persistence.rs` | **Disk Sync** | SQLite primary; supports `working_memory` and debounced flushing |
| `src/agent/persistence_tests.rs` | **Persistence Tests** | Unit tests for registry loading and template sync |
| `src/agent/benchmarks.rs` | **Performance Logic** | Data layer for recording and retrieving benchmark results |
| `src/agent/persistence.rs` | **Unified Persistence** | SQLite state tracking for all Swarm Agents + SyncManifest |
| `src/agent/connectors.rs` | **SME Data Connectors** | IngestionWorker daemon, FsConnector, SyncManifest-based incremental sync |
| `src/agent/workflows.rs` | **SOP Engine** | Markdown-based deterministic workflow execution (WorkflowExecutionState) |
| `src/agent/parser.rs` | **Document Parser** | Layout-aware extraction for .txt, .md, .csv, .pdf with overlap chunking |
| `src/agent/rates.rs` | **Cost Calculator** | USD-per-token rates for all known models |
| `src/agent/skills.rs` | **Dynamic Skills** | File-system registry for Workflows |
| `src/agent/script_skills.rs`| **Skill Parsing** | High-fidelity Markdown parser for individual skills |
| `src/agent/skill_manifest.rs`| **Skill Manifests** | JSON schema validation, automated security gates (`budget:spend`) |
| `src/agent/continuity/` | **Cron Scheduler** | Autonomous background missions (`scheduler.rs`, `executor.rs`, `workflow.rs`) |
| `src/agent/provider_trait.rs`| **LLM Interface** | Generic trait decoupling the engine from specific API clients |
| `src/agent/null_provider.rs` | **Fallback Provider** | Graceful degradation without API keys (via integration tests) |
| `src/agent/types.rs` | **Type Definitions** | Source of truth for `TaskPayload`, `ModelConfig`, `EngineAgent` (incl. `requires_oversight`), etc. |
| `src/agent/sanitizer.rs` | **Input Sanitizer** | Regex-driven prompt injection and exfiltration defense |
| `src/agent/archival.rs` | **Data Purger** | Cold-storage archival for completed missions |
| `src/adapter/filesystem.rs` | **Workspace I/O** | Sandboxed ops; `canonicalize`-based symlink-safe containment check |
| `src/adapter/vault.rs` | **Vault Persistence** | Appends Markdown files to `vault/` directory |
| `src/adapter/discord.rs` | **Discord Webhook** | Sends alerts via `DISCORD_WEBHOOK` env var |
| `src/agent/audio.rs` | **Neural Voice** | Local Piper TTS, Whisper STT, and Silero VAD sessions |
| `src/agent/audio_cache.rs`| **Bunker Cache** | SQLite-backed semantic audio caching for zero-latency replay |
| `src/middleware/auth.rs` | **Auth Logic** | `NEURAL_TOKEN` verification and session gatekeeping |
| `src/routes/ws.rs` | **WebSocket Hub** | Multiplexes JSON logs and **Binary PCM Audio Streams** |
| `src/routes/audio.rs` | **Voice APIs** | Hybrid Transcription (Groq/Local) and Synthesis (OpenAI/Piper) |
| `src/routes/agent.rs` | **Agent REST** | Agent lifecycle, mission control, and oversight interaction |
| `src/routes/benchmarks.rs` | **Bench REST** | Performance data ingestion and historical analysis |
| `src/routes/skills.rs` | **Cap REST** | Custom skills and workflows CRUD |
| `src/routes/continuity.rs` | **Cron REST** | Management of scheduled jobs and autonomous triggers |
| `src/routes/deploy.rs` | **Deploy API** | Remote bunker deployment and node updates |
| `src/routes/env_schema.rs` | **Env API** | Runtime environment metadata and exposure |
| `src/routes/model_manager.rs`| **Model API** | AI Provider testing and node handshake logic |
| `src/routes/oversight.rs` | **Oversight API** | Human-in-the-loop decision routing |
| `src/routes/templates.rs` | **Template API** | GitHub Native Swarm discovery and installation |
| `src/routes/nodes.rs` | **Node Discovery** | Bunker node detection and system-wide broadcast |
| `src/security/audit.rs` | **Merkle Audit Trail** | ED25519-signed SHA-256 hash chain; supports full-chain and granular per-record verification |
| `src/security/monitoring.rs` | **Resource Guard** | Real-time system/process RAM and CPU telemetry; sandbox (Docker/K8s) detection |
| `src/security/metering.rs` | **Budget Metering** | Per-agent/mission quotas; debounced persistence with `DashMap` buffers |
| `src/security/scanner.rs` | **Vuln Scanner** | Automated dependency and configuration security checks |
| `src/state.rs` | **AppState** | Modular hub architecture: `registry`, `resources`, `comms`, `governance`, `security` |
| `src/startup.rs` | **System Init** | Bootstrap logic; log setup; background tasks (including debounced flush loop) |
| `src/router.rs` | **Decoupled Router** | Axum routing table, CORS policy, and middleware stacks |
| `src/main.rs` | **Minimal Entry** | Highly simplified entry point that delegates to `startup` and `router` |
| `src/db.rs` | **SQLite Engine** | Initialization and migration routing |
| `src/db_tests.rs` | **Database Tests** | Unit tests for DB initialization and migrations |
| `src/env_schema.rs` | **Env Validation** | Runtime validation of required/optional environment variables |
| `src/secret_redactor.rs` | **Secret Redactor** | Regex-based API key redaction for WebSocket log broadcasts |
| `src/telemetry.rs` | **OpenTelemetry** | Tracing pipeline with OTLP, structured span attributes |
| `src/utils/serialization.rs` | **Serialization** | SanitizedJson with automated 2KB string pruning |
| `src/utils.rs` | **Logic Tools** | Global helper functions and extension traits |
| `workspaces/` | **Physical Sandboxes** | One directory per cluster; mapped from `cluster_id` in `RunContext` |

### Data Flow (State Persistence)
`Registry (Static)` → `DB (Persistence)` → `AppState (DashMap)` → `Runner (Execution)` → `Workspace (Files)`

> **Mission Clusters** (logical organization) are persisted in Frontend **LocalStorage** (`tadpole-workspaces-v3`), not in the Rust backend.

## Frontend: React Dashboard (`src/`)

| Directory | Purpose | Key Files |
| :--- | :--- | :--- |
| `pages/` | **Views** | `OpsDashboard.tsx` (Modular), `OrgChart.tsx`, `Missions.tsx` (Modular), `Workspaces.tsx`, `Skills.tsx`, `BenchmarkAnalytics.tsx` |
| `services/` | **Specialized APIs** | `AgentApiService.ts`, `MissionApiService.ts`, `SystemApiService.ts`, `BaseApiService.ts` |
| `services/` | **Proxy & Socket** | `tadpoleosService.ts` (Proxy), `tadpoleosSocket.ts` (WS), `skillStore.ts` |
| `services/` | **Reactive State** | `agentStore.ts` (Zustand), `settingsStore.ts`, `roleStore.ts`, `providerStore.ts`, `tabStore.ts`, `headerStore.ts` |
| `components/` | **Units** | `AgentConfigPanel.tsx` (incl. Budget & Oversight tab), `SovereignChat.tsx`, `CommandPalette.tsx`, `PageHeader.tsx`, `TabBar.tsx`, `TabItem.tsx`, `PortalWindow.tsx`, `CommandTable.tsx`, `SwarmOversightNode.tsx`, `ImportPreviewModal.tsx` |
| `components/` | **Layouts** | `missions/` (NeuralMap, ClusterSidebar), `dashboard/` (SystemLog, StatMetrics), `hierarchy/` (NodeHeader - Neural Oversight Toggle) |
| `hooks/` | **Custom Hooks** | `useEngineStatus.ts`, `useDashboardData.ts`, `useEventBus.ts` (RAF batching), `useApiRequest.ts` (Auth injection) |

## Deployment & Ops

- **`Dockerfile`**: Multi-stage build. Rust release build → `debian:bookworm-slim` runtime with static file serving from Axum.
- **`deploy.ps1`**: Hybrid automation (Local Frontend Build → Remote Docker Rebuild).
- **`.env`**: Core security boundary. `NEURAL_TOKEN` is **required** — engine panics at startup in release builds if missing.

## AI Navigation Tips

- **Find mission schema changes**: `grep` on `server-rs/src/agent/types.rs`
- **Trace a tool call**: Start in `runner/mod.rs:execute_tool()`, then follow to modular handlers in `runner/fs_tools.rs`, `runner/external_tools.rs`, `runner/mission_tools.rs`
- **Rate limit config**: `ModelConfig.rpm` / `ModelConfig.tpm` → enforced in `runner.rs:call_provider()` via `rate_limiter.rs`
- **Workspace path**: Always `ctx.workspace_root` — never hardcode
- **State audit**: Check `state.rs` — `AppState` is the single source of truth for all live data
- **Sovereignty flow**: See `Standups.tsx` for audio capture → `routes/audio.rs` → Groq Whisper → `runner.rs:handle_issue_alpha_directive()`
- **Neural Oversight flow**: `workspaceStore.activeProposals` → `HierarchyNode.tsx` passes props → `NodeHeader.tsx` (Brain Icon) → `SwarmOversightNode.tsx` (Floating Modal).
- **Security boundary**: `adapter/filesystem.rs:get_safe_path()` — do not bypass
- **API Versioning**: All business endpoints (agents, oversight, infra, benchmarks, mcp) are versioned under `/v1/`. Engine-level routes (health, kill, shutdown, ws) remain at root.
