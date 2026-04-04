# 🗺️ Codebase Map (AI Navigator)

> **Status**: Super AI-Awakened (Level 5)  
> **Version**: 1.5.0  
> **Last Updated**: 2026-04-04  
> **Classification**: Sovereign  

---

This map is designed to help AI assistants (and human developers) navigate the Tadpole OS project structure and understand component relationships. The backend architecture follows the **Super AI-Awakened** header protocol for 100% internal transparency and agent-fluent reasoning.

## Backend: Rust Engine (`server-rs/`)

| Directory / File | Purpose | Key Notes |
| :--- | :--- | :--- |
| `src/agent/runner/mod.rs` | **Execution Core** | **The Heartbeat**: Mission lifecycle, OTel tracing, and budget gates. |
| `src/agent/runner/lifecycle.rs` | **Lifecycle Mgmt** | Core state transitions and mission startup/shutdown hooks. |
| `src/agent/runner/tools.rs` | **Tool Execution** | Parallel tool execution via `FuturesUnordered`. |
| `src/agent/runner/analysis.rs` | **Root Cause Analysis** | Post-tool failure diagnostics and recovery logic. |
| `src/agent/runner/synthesis.rs` | **Prompt Engineering** | Prompt engineering + Context pruning. |
| `src/agent/runner/external_tools.rs` | **External Tools** | External service tool handlers (web, API, fetch). |
| `src/agent/runner/fs_tools.rs` | **File System Tools** | Sandboxed file I/O tool handlers. |
| `src/agent/runner/mission_tools.rs` | **Mission Tools** | Mission-specific tool handlers (recruit, delegate, reflect). |
| `src/agent/runner/swarm.rs` | **Swarm Orchestration** | Multi-agent coordination and recursive recruitment. |
| `src/agent/runner/provider.rs` | **Provider Routing** | Unified LLM provider resolution (Gemini/OpenAI/Local-ONNX). |
| `src/agent/runner/oversight.rs` | **Oversight Logic** | Human-in-the-loop gatekeeping for sensitive operations. |
| `src/agent/runner/context.rs` | **Run Context** | State management for recursive agent executions. |
| `src/agent/memory.rs` | **Vector Memory** | **The Soul**: LanceDB RAG Scopes and High-Fidelity metadata retrieval. |
| `src/routes/memory.rs` | **Memory API** | **Advanced RAG**: Multi-Factor Scoring (MFS) with Mission Affinity (1.2x). |
| `src/agent/hooks.rs` | **Lifecycle Hooks** | `HooksManager` for `pre-tool` and `post-tool` auditing |
| `src/agent/gemini.rs` | **Google Provider** | Concurrent tool call support via `generate` |
| `src/agent/groq.rs` | **Groq Provider** | Shared client + Llama tool-call recovery; unused struct fields removed |
| `src/agent/openai.rs` | **OpenAI Provider** | Standard integration for GPT-4o and O1 models |
| `src/agent/rate_limiter.rs` | **API Quota Guard** | `Semaphore`-based RPM + `AtomicU32` TPM; auto-enforced in `call_provider` |
| `src/agent/mission.rs` | **Mission CRUD** | `row_to_mission()` helper eliminates 3× DRY violation; `str_to_status()` |
| `src/agent/backlog.rs` | **Mission Backlog** | Queue management for pending and scheduled tasks |
| `src/agent/persistence.rs` | **Unified Persistence** | SQLite state tracking for all Swarm Agents + SyncManifest |
| `src/agent/connectors.rs` | **SME Data Connectors** | IngestionWorker daemon, FsConnector, SyncManifest-based incremental sync |
| `src/agent/workflows.rs` | **SOP Engine** | Markdown-based deterministic workflow execution (WorkflowExecutionState) |
| `src/agent/context_manager.rs`| **State Windowing** | Sliding-window context management for long missions |
| `src/agent/parser.rs` | **Document Parser** | Layout-aware extraction for .txt, .md, .csv, .pdf with overlap chunking |
| `src/agent/rates.rs` | **Cost Calculator** | USD-per-token rates for all known models |
| `src/agent/skills.rs` | **Dynamic Skills** | File-system registry for Workflows |
| `src/agent/script_skills.rs`| **Skill Parsing** | High-fidelity Markdown parser for individual skills |
| `src/agent/skill_manifest.rs`| **Skill Manifests** | JSON schema validation, automated security gates (`budget:spend`) |
| `src/agent/registry.rs` | **Agent Registry** | Static/dynamic recruitment system and agent discovery |
| `src/agent/continuity/` | **Cron Scheduler** | Autonomous background missions (`scheduler.rs`, `executor.rs`, `workflow.rs`) |
| `src/agent/mcp/` | **MCP Server Hub** | Integration layer for Model Context Protocol servers |
| `src/agent/provider_trait.rs`| **LLM Interface** | Generic trait decoupling the engine from specific API clients |
| `src/agent/null_provider.rs` | **Fallback Provider** | Graceful degradation without API keys (via integration tests) |
| `src/agent/types.rs` | **Agent Schemas** | Source of truth for `TaskPayload`, `EngineAgent`, `requires_oversight`, etc. |
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
| `src/security/audit.rs` | **Merkle Audit Trail** | ED25519-signed SHA-256 hash chain; granular per-record verification |
| `src/security/monitoring.rs` | **Resource Guard** | Real-time system/process RAM and CPU telemetry; sandbox detection |
| `src/security/metering.rs` | **Budget Metering** | Per-agent/mission quotas; debounced persistence with `DashMap` buffers |
| `src/security/scanner.rs` | **Vuln Scanner** | Automated dependency and configuration security checks |
| `src/services/` | **Core Services** | System-wide background services: `discovery.rs`, `privacy.rs` |
| `src/state/` | **AppState Hub** | Modular state hub: `registry`, `resources`, `comms`, `governance`, `security` |
| `src/startup.rs` | **System Init** | Bootstrap logic; log setup; background tasks and flush loops |
| `src/router.rs` | **Decoupled Router** | Axum routing table, CORS policy, and middleware stacks |
| `src/main.rs` | **Minimal Entry** | Entry point delegating to `startup` and `router` |
| `src/db.rs` | **SQLite Engine** | Initialization and migration routing |
| `src/types/` | **System Types** | Shared telemetry (`LogEntry`), `SubsystemStatus`, and `rag_scoring` |
| `src/env_schema.rs` | **Env Validation** | Runtime validation of environment variables |
| `src/secret_redactor.rs` | **Secret Redactor** | Regex-based API key redaction for WebSocket logs |
| `src/telemetry/mod.rs` | **Telemetry Hub** | Registration point for tracing and pulse loops |
| `src/telemetry/pulse.rs` | **Swarm Pulse** | **The Pulse**: 10Hz binary telemetry aggregator (MessagePack) |
| `src/telemetry/pulse_types.rs`| **Pulse Schemas** | Binary data structures for nodes and edges |
| `src/telemetry.rs` | **OpenTelemetry** | Tracing pipeline with OTLP and structured span attributes |
| `src/utils/` | **Core Utilities** | Tooling foundation: `graph`, `parser`, `security`, `serialization` |
| `workspaces/` | **Physical Sandboxes** | One directory per cluster; mapped from `cluster_id` in `RunContext` |

## 🛠️ Operations & Tooling

| Directory / File | Purpose | Key Notes |
| :--- | :--- | :--- |
| `execution/` | **Active Toolset** | Standardized Python scripts with `[OK]` / `[FAIL]` reporting protocol |
| `execution/parity_guard.py` | **Integrity Gate** | Context-aware route discovery (supports Axum nesting) |
| `execution/scout.py` | **Search Engine** | Explorer-scout logic with relative path resolution |
| `execution/verify_all.py` | **Master Suite** | Top-level verification of all system health checks |
| `scripts/deploy/` | **Deployment Ops** | Production deployment and node update automation |
| `logs/` | **Audit History** | Centralized location for system, lint, and security logs |

### Data Flow (State Persistence)
`Registry (Static)` → `DB (Persistence)` → `AppState (DashMap)` → `Runner (Execution)` → `Workspace (Files)`

> **Mission Clusters** (logical organization) are persisted in Frontend **LocalStorage** (`tadpole-workspaces-v3`), not in the Rust backend.

## Frontend: React Dashboard (`src/`)

| Directory | Purpose | Key Files |
| :--- | :--- | :--- |
| `pages/` | **Views** | `OpsDashboard`, `OrgChart`, `Missions`, `Workspaces`, `Skills`, `BenchmarkAnalytics`, `AgentManager`, `Docs`, `ModelStore`, `OversightDashboard`, `SecurityDashboard`, `Standups` |
| `services/` | **Bridge Layers** | `AgentApiService.ts`, `MissionApiService.ts`, `SystemApiService.ts`, `BaseApiService.ts`, `tadpoleosService.ts` (Proxy) |
| `services/` | **Infrastructure** | `socket.ts` (WS Engine), `eventBus.ts`, `voiceClient.ts`, `ProposalService.ts` |
| `stores/` | **Reactive State** | `agentStore.ts` (Zustand), `settingsStore.ts`, `roleStore.ts`, `providerStore.ts`, `tabStore.ts`, `headerStore.ts`, `workspaceStore.ts`, `skillStore.ts` |
| `components/` | **Module Units** | `AgentConfigPanel`, `Sovereign_Chat.tsx`, `CommandPalette.tsx`, `CommandTable.tsx`, `SwarmOversightNode.tsx`, `ImportPreviewModal.tsx`, `LineageStream.tsx`, `Swarm_Visualizer.tsx` |
| `components/` | **Layout/UI** | `layout/` (Sidebar, OrgHeader, TabBar), `ui/` (PageHeader, PortalWindow, ToastCenter), `missions/` (NeuralMap), `dashboard/` (SystemLog, StatMetrics) |
| `hooks/` | **Custom Hooks** | `useEngineStatus.ts`, `useDashboardData.ts`, `useEventBus.ts` (RAF batching), `useApiRequest.ts` (Auth injection), `use_sovereign_store.ts` |

## 🚀 Deployment & Ops

- **`Dockerfile`**: Multi-stage build. Rust release build → `debian:bookworm-slim` runtime with static file serving from Axum.
- **`scripts/deploy/`**: Contains core deployment automation like `deploy.ps1`.
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
