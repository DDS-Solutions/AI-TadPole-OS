> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> <!-- Last Audit Sync: 2026-07-25T14:31:00Z -->
> ### AI Assist Note
> Automated governance and architectural tracking.
>
> ### 🔍 Debugging & Observability
> Traceability via `execution/parity_guard.py`.


# 🏗️ Tadpole OS Architecture: Technical Hub

**Intelligence Level**: High (ECC Optimized)  
**Version**: 1.2.5  
**Last Hardened**: 2026-07-21 (Telemetry module synchronization & Parity Guard compliance)
**Standard Compliance**: ECC-ARA (Enhanced Contextual Clarity)

---

## 🎯 Executive Summary

Tadpole OS is a sovereign, high-performance runtime for multi-agent swarms. It utilizes a **Gateway-Runner-Registry** pattern implemented in Rust to provide a local-first, privacy-respecting intelligence environment. The system has recently been hardened with a **Unified Tool Registry**, centralizing all system capabilities into a single, data-driven discovery point.

> [!TIP]
> **New to the codebase?** Start with the [Architecture Overview](./Architecture_Overview.md) for a 3-paragraph executive brief and core system topology.

---

## 🛰️ Documentation Suite

To maintain technical clarity and reduce information density, the architecture is decomposed into the following specialized modules:

| Module | Description | Key Technologies |
|:--- |:--- |:--- |
| **[🏗️ Overview](./Architecture_Overview.md)** | Core topology and system philosophy. | Axum, Mermaid, Sovereign Layer |
| **[🛡️ Security Model](./Security_Model.md)** | Zero-Trust, CBS, WAL, and Oversight. | ED25519, CBS, WAL, Audit |
| **[🤖 Agent Runner](./Agent_Runner_Workflow.md)** | Execution lifecycle and Swarm protocols. | Tokio, LLM Dispatch, Recruitment |
| **[🧠 Memory & RAG](./Knowledge_Memory.md)** | Hybrid memory strategy and ingestion. | LanceDB, SQLite, BM25 Reranking |
| **[⚛️ Client Architecture](./Frontend_State_Management.md)** | Frontend state and multi-window sync. | React 19, Zustand, Portals |

---

## ⚙️ Core Engine Subsystems (server-rs)

The Rust engine implements several high-fidelity subsystems to ensure sovereign stability and intelligence.

### 1. Unified Tool Registry (NEW)
- **@docs ARCHITECTURE:Registry** (`tools/registry.rs`)
- **Centralized Manifest**: All tools (builtin, categorical, and dynamic) are defined in a unified `ToolManifest` (`tools/manifest.rs`), providing a single source of truth for names, descriptions, and parameter schemas.
- **Discovery Parity**: The `Synthesis` layer now dynamically pulls from the registry, ensuring that the model's understanding of its "toolbelt" is always in 100% sync with the execution engine.
- **Hot-Reload Ready**: Designed for future autonomous evolution, allowing the swarm to register and refine new capabilities without engine restarts.
- **Shared Idempotent Tool Caching (A)**: Cache read-only tool outputs (`read_file`, `grep_search`, `list_file_symbols`, etc.) keyed by name, workspace root, and arguments. Invalidates entries dynamically on mutating file operations. Network/volatile tools are excluded.

### 2. Zero-Trust Tool Pipeline (SEC-04)
- **@docs ARCHITECTURE:Security** (`tools/mod.rs`)
- **Capability-Based Security (CBS)**: Replaces ambient authority with explicit, cryptographically signed permission tokens.
- **CBS Key Rotation**: Production deployments should set `CAPABILITY_KEY_CURR` (active 32-byte hex signing key) and `CAPABILITY_KEY_PREV` (outgoing key for zero-downtime rotation). If unset, a random ephemeral key is generated per boot (development only).
- **Write-Ahead Logging (WAL)**: Ensures all tool intents are persisted to the audit trail *before* execution begins.
- **Cryptographic Oversight Gate**: Enforces operator decision signing via client-side Ed25519 key generation and server-side signature verification across both REST and WebSocket endpoints.
- **Isolated Context**: Every tool executes within a constrained `ToolContext`, preventing side-channel access to global engine state.
- **Concurrent Conflict Locks (B)**: Introduces a lease-lock `ConflictManager` to lease active paths to writing agents. Actively leased files reject concurrent edits, and leases automatically expire after 30 seconds to prevent deadlock states.

### 3. Intelligent Model Registry (IMR-01)
- **@docs ARCHITECTURE:Intelligence** (`capability_matrix.rs`)
- **@docs ARCHITECTURE:Agent** (`model_manager.rs`)
- Provides automated discovery and capability inference (Vision, Tools, Reasoning) based on model ID slugs.
- Includes provider handshake validation and secure secret redaction (SEC-02).

### 4. Swarm Persistence & Governance
- **@docs ARCHITECTURE:Persistence** (`persistence.rs`, `agent_db.rs`, `sync_manifests.rs`, `workspace_store.ts`)
- **@docs ARCHITECTURE:Registry** (`recipes.rs`, `Cluster_Manager_Modal.tsx`, `Cluster_Sidebar.tsx`)
- **Concurrent Active Clusters**: Independent `is_active` state toggles allowing up to `MAX_CLUSTERS = 10` active mission clusters running parallel swarm operations simultaneously.
- **Mode-Switched Utility Cluster Studio**: Dynamic team preset configuration studio featuring $O(1)$ agent lookups (`agent_map`), deletion safety confirmation gates (`Confirm_Dialog`), consolidated form data interfaces (`PresetFormData`), and uppercase team badges (`🏷️ [TEAM: BADGE]`).
- **Transactional Persistence Guard**: `save_agent_db` wraps `execute_save_agent` and `sync_manifests_for_agent` inside a single SQLite transaction (`pool.begin()`), guaranteeing atomic manifest synchronization with agent state.
- **Strict JSON Error Propagation**: `parse_json_field` returns `Result<T, AppError>`, failing fast on corrupted JSON strings during agent loading (`load_agents_db`) to prevent silent data loss or capability wipes.
- **Agent Reaping**: Automated "Safety Valve" to harvest stale/crashed agent runs (LIF-03).
- **Heartbeat Propagation**: Real-time health tracking for long-running missions.
- **Hierarchical RBAC**: Resolves permission policies hierarchically: `Agent-Specific Policy` -> `Role-Based Policy` -> `Global Tool Policy` -> `Prompt (Sovereign Safety)`.
- **Role Blueprints**: Declarative templates for rapid swarm bootstrapping.

### 5. Swarm Intelligence & Autonomous Evolution
- **@docs ARCHITECTURE:Persistence** (`swarm_persistence.rs`)
- **@docs ARCHITECTURE:Agent** (`mission_tools.rs`)
- **Hierarchical Coordination**: Peer-to-peer directive delegation and multi-stage audit loops (Critic-Refiner).
- **Dynamic Tool Synthesis**: Autonomous generation and registration of Python/Node micro-scripts (Skills).
- **Turn Preservation Compaction (D)**: Dialogue-level context compactor that keeps the last 4 reasoning turns raw, truncates/evicts older turns' embedded code logs exceeding 2,000 characters, and falls back to a deterministic regex-based parser on LLM summarizer failure. Optimized to run in $O(N)$ linear time by rebuilding history iteratively instead of performing in-place vector shifts, removing CPU load spikes during long context sequences.

### 6. CodeBase Intelligence & Semantic Graph (MOD-03)
- **@docs ARCHITECTURE:CodeBaseIntelligence** (`utils/parser.rs`, `intelligence/graph.rs`)
- **@docs ARCHITECTURE:Intelligence** (`routes/intelligence.rs`)
- **Tree-sitter Symbol Extraction**: High-fidelity AST parsing for Rust and TypeScript source files. Extracts functions, structs, traits, interfaces, and method signatures into `Symbol` nodes for graph construction.
- **Incremental AST Caching (C)**: Tracks file size and modification times (`mtime`) in `file_metadata`. The build method scans the workspace, cleans deleted files, and parses *only* new or modified files. The directed `petgraph` of nodes and edges is rebuilt in memory in microseconds, bypassing the Tree-sitter file-reading bottleneck.
- **Dependency Graph**: Builds a directed `petgraph` of `SymbolNode` relationships, mapping cross-file references to enable real-time impact analysis.
- **Blast Radius Analysis**: Calculates downstream dependents of any given symbol, enabling safe refactoring and change-impact visualization in the dashboard.
- **Path Obfuscation**: All file paths returned to the frontend are salted and hashed to prevent workspace path leakage (SEC-05).
- **REST Surface**: Exposed via `GET /v1/intelligence/graph` and `GET /v1/intelligence/blast-radius` with path traversal hardening on query inputs.
- **Resilient Client-Side Graph Sanitizer**: A fault-tolerant sanitizer (`graph_sanitizer.ts`) running on the client side that validates raw JSON payloads, coerces line ranges, normalizes kinds/aliases (e.g. `fn`/`func` -> `function`), drops orphaned/dangling links, and guarantees graph referential integrity prior to UI rendering.
- **Interactive BFS Pathfinder Modal**: An in-browser search modal (`PathFinderModal.tsx`) executing client-side Breadth-First Search (BFS) algorithm to compute and highlight downstream/upstream symbol trace paths dynamically.
- **Visualizer Navigation Stack**: Implements a browser-style Back/Forward navigation stack in the UI (`KnowledgeGraph.tsx`) that stores node traversal histories, enabling seamless navigation through complex codebases.
- **Docstring Range Tracking (ADG-04)**: The `Symbol` type now carries an optional `docstring_range: Option<SymbolRange>` field capturing the exact byte and line coordinates of associated JSDoc/Rustdoc comments. This is propagated to `SymbolNode` in the graph, enabling the Active Documentation Guard to perform in-place, back-to-front auto-fixes of mismatched symbol references without invalidating preceding byte coordinates.
- **Tauri IPC Command Validation (ADG-04)**: The parser now emits `tauri_ipc:` prefixed symbols for `#[tauri::command]` annotated Rust functions and matching `invoke()` calls in TSX files. The symbol graph can therefore detect frontend/backend boundary drift by cross-referencing these synthetic nodes.
- **Active Documentation Guard (ADG-SG)**: The `graph_query validate` command enforces symbol-to-docstring parity. It extracts backtick-quoted symbols from Rustdoc/JSDoc comments and verifies they resolve against the symbol graph. Supports `--strict` (fail-fast), `--diff` (validate only git-modified files), and `--fix` (auto-correct drifted names using Jaro-Winkler similarity matching).
- **Scriptable Graph CLI — Modular Architecture (ADG-05)**: The `graph_query` binary has been decomposed from a monolithic 1,393-line file into five single-responsibility modules under `server-rs/src/bin/graph_query/`: `main.rs` (CLI routing), `path_utils.rs` (canonicalization + git diff), `visualizer.rs` (Mermaid + Cytoscape/HTML), `query_manager.rs` (domain queries + exports), and `doc_guard.rs` (linting + auto-fix). Supports `--format mermaid|html` for blast radius visualization exports.

### 7. API Gateways, Router & State Management
- **@docs ARCHITECTURE:Gateways** (`routes/mod.rs`, `routes/intelligence.rs`, `router.rs`)
- **@docs ARCHITECTURE:Networking** (`router.rs`, `routes/mod.rs`, `startup/mod.rs`)
- **@docs ARCHITECTURE:State** (`state/mod.rs`, `services/mod.rs`)
- **Route Delegation**: High-speed endpoint routing powered by Axum. Enforces Bearer token authentication via `NEURAL_TOKEN`.
- **Path Hardening**: Validates all inputs to avoid path-traversal vulnerabilities by verifying resource paths reside strictly within the user's `WORKSPACE_ROOT`.
- **System State Hub**: Manages global shared references to resource registries, database connection pools, agent lifecycles, and configuration options inside `AppState` (`state/mod.rs`).

### 8. Test Suite Isolation & Settings Store Verification
- **@docs ARCHITECTURE:TestSuites** (`stores/settings_store.test.ts`)
- **Test Sandbox Isolation**: Implements strict test isolation in `settings_store.test.ts` via `vi.hoisted` to mock/stub `localStorage` and `atob`/`btoa` globally before store re-evaluation, avoiding persistent state leakages across test cycles.

### 9. System Core & Configuration
- **@docs ARCHITECTURE:Core** (`config.rs`, `utils/deduplicator.rs`, etc.)
- **@docs ARCHITECTURE:Configuration** (`config.rs`)
- **Global Configuration Registry**: Centralizes server settings, database pools, and directory settings in `AppConfig` (`config.rs`). Auto-populates from environment variables and validates them on startup.
- **Deduplication Engine**: Employs SHA-256 content hashing to ensure idempotent ingest across P2P Bunker nodes and prevent knowledge redundancy.

### 10. Institutional Knowledge Store (IKS) & Open Knowledge Format (OKF)
- **@docs ARCHITECTURE:IKS** (`routes/knowledge.rs`, `agent/knowledge_store.rs`)
- **OKF-Aligned Schema**: SQLite-backed storage mapping swarm insights using the **Open Knowledge Format (OKF)** metadata parameters (`concept_type`, `title`, `description`, `resource_uri`, `tags`).
- **Pagination & Filters**: Standardized query interface supporting dynamic offset/limit query parameters and concept-type queries.
- **Visual OKF Force-Graph Toggle**: HUD controller (`KnowledgeGraph.tsx` and `GraphContext.tsx`) that enables toggling the primary visualization between Codebase Symbols and the Semantic OKF Graph. Employs monochromatic Zinc-scaled design tokens (`DESIGN_SYNERGY.md`) for concept types and reserves vibrant accents (`cyber-green`, `cyber-amber`, `cyber-red`) for live node health, verification, and link configuration anomalies.

### 11. Agent-to-Agent (A2A) Economic Layer
- **@docs ARCHITECTURE:Economics** (`agent/runner/a2a_ledger.rs`, `agent/runner/a2a_mailbox.rs`, `agent/runner/a2a_router.rs`)
- **Two-Phase Commit (2PC) Transactions**: Provides a deterministic protocol (`prepare_transaction` $\rightarrow$ `commit_transaction`/`rollback_transaction`) to prevent double-spending or orphaned allocations. Transactions are locked in a pending state until execution confirms success.
- **Mailbox Protocol & Routing**: Standardized message mailboxes (`a2a_mailbox.rs`) and transaction routing (`a2a_router.rs`) that allow agents to request paid services from other agents in the swarm. Preserves model reasoning traces and file artifacts during delivery.
- **SQLite Transaction Ledger**: Persists all financial transactions in the `transaction_ledger` and `transaction_locks` tables under the local database, providing a non-repudiable cryptographically audited transaction ledger.
- **Economic Limit Guards**: Employs a thread-safe global `PaymentRouter` which checks daily spend limit caps across Economic Zones (Dev, Staging, Prod) by evaluating confirmed transactions and active pending locks prior to transaction preparation.

### 12. AI-Tadpole-OS Swarm Orchestration Engine
- **@docs ARCHITECTURE:Orchestration** (`agent/runner/swarm.rs`, `agent/runner/conductor.rs`, `agent/runner/context.rs`, `Dual_Trace_Playback.tsx`)
- **Dual-Trace Telemetry Playback Inspector**: Real-time side-by-side execution trace visualizer comparing baseline directive steps against 100% certified World Model execution state machines (`Dual_Trace_Playback.tsx`). Auto-selects latest runs on `/analytics` navigation.

### 13. Outward A2A Gateway & Customer Knowledge Catalog Architecture
- **@docs ARCHITECTURE:OutwardGateway** (`agent/outward/outward_gateway.rs`, `agent/outward/customer_catalog.rs`, `agent/outward/mod.rs`, `routes/outward_routes.rs`)
- **Zero-Trust Silo Isolation**: Completely isolates public customer-facing agent interactions from internal developer codebase symbol graphs (`petgraph` AST tree) to prevent IP exposure and context exhaustion.
- **Outward Local Model Runner (`gemma4:e4b`)**: Binds customer agents to low-footprint local inference profiles to maximize speed and preserve VRAM on SMB workstations.
- **SMB Catalog Ingestion Engine & Ranked Search**: Quote-aware CSV parser (`parse_csv_line`), QuickBooks schema ingest (`ingest_quickbooks_json`), PDF pricelist parsing, and BM25-style keyword relevance scoring (`search_catalog`).
- **A2A `agent-card.json` Publisher & IP Rate Limiting Guard**: Dynamic publisher generating standard `a2a-protocol.org` agent metadata and skills array over Axum REST endpoints (`/a2a/v1/agent-card.json`, `/a2a/v1/catalog/search`) protected by a thread-safe 60 req/min IP token-bucket rate limiter (`IpRateLimiter`).
- **Heuristic Fast-Path (System 1)**: Analyzes query complexity (`is_fast_path_query`) to bypass the Sentinel Gate and reason within a single step for simple queries, reducing latency.
- **OpenAI-Compatible Gateway**: Routes synchronous completions requests (`POST /v1/agents/chat/completions`) directly to the swarm runner, allowing any OpenAI-compatible client to interface with the agent network.
- **Context Sandboxing**: Sub-agent context is isolated using the `visible_transcript` observation history, preventing parent monologue and raw tool telemetry from leaking into worker context.
- **Builder-Debugger Pair Swapping**: Automatically switches active model slots (e.g., swapping to a debugging model configuration) upon encountering tool execution, compilation, or validation failures.
- **Conductor Plan (DAG)**: Decomposes objectives into structured dependency graphs (`conductor.rs`). Activates Kahn's/DFS sorting (`topological_sort_conductor_steps`) to schedule sub-agents topologically, preventing circular deadlocks.

### 13. Shield Layer (Evasion-Resistant Normalization)
- **@docs ARCHITECTURE:ShieldLayer** (`security/normalizer.rs`)
- **Multi-Pass Normalization**: Performs Unicode NFKC folding, confusable Cyrillic/Greek homoglyph resolution, markdown symbol stripping, leetspeak translation, and recursive base64 decoding.
- **Evasion Detection**: Integrates with the `ShellScanner` to collapse whitespace and normalize inputs, preventing command injection obfuscation.

---

## 🏗️ 2026 Framework Modernization

Tadpole OS leverages a cutting-edge stack to ensure peak performance:
- **React 19**: Adopting modern hydration and zero-jank telemetry patterns.
- **Tailwind CSS v4**: CSS-first theme architecture with sub-millisecond HMR.
- **Axum 0.8**: High-performance Rust networking with decoupled router isolation.
- **Mythos Engine**: Deep Recurrent Reasoning (RDT) with Adaptive Computation (ACT) halting.
- **OTel Tracing**: Enterprise-grade observability via the Telemetry Hub.

---

## 📂 Directory Structure

```
├── server-rs/          # Layer 2: Rust Orchestration Engine
│   ├── src/agent/       # Core Runner, RAG, and Persistence
│   ├── src/agent/runner/tools/ # Zero-Trust Pipeline & CBS
│   ├── src/bin/graph_query/    # Modular intelligence CLI (ADG-05)
│   │   ├── main.rs             #   CLI routing and entry point
│   │   ├── path_utils.rs       #   Canonicalization + git diff
│   │   ├── visualizer.rs       #   Mermaid + Cytoscape/HTML exports
│   │   ├── query_manager.rs    #   Domain lookups + audit exports
│   │   └── doc_guard.rs        #   ADG linter + Jaro-Winkler auto-fix
│   ├── src/error/       # Modular Error Engine (decomposed)
│   │   ├── mod.rs              #   AppError enum + re-exports
│   │   ├── metadata.rs         #   HasErrorMetadata trait
│   │   └── rfc9457.rs          #   ProblemDetails IntoResponse impl
│   ├── src/intelligence/# CodeBase Graph & Blast Radius Analysis
│   ├── src/routes/oversight/   # Governance Gateway (decomposed)
│   │   ├── mod.rs              #   Router aggregation + re-exports
│   │   ├── ledger.rs           #   Pending queue, decisions, crypto
│   │   ├── quotas.rs           #   Agent + mission budget quotas
│   │   └── security.rs         #   Audit trail, integrity, policies
│   ├── src/security/    # Merkle Audit, Metering, Scanner, DependencyGuard, Monitoring
│   ├── src/utils/       # Tree-sitter Parsing & Serialization
│   └── src/state/       # Modular AppState Hubs
├── docs/               # Technical Specification Suite
├── data/               # Persistent State (SQLite, Vector DB)
└── src/                # Frontend (React/Zustand)
```

---

## 🤖 Context for AI Assistants

1.  **State Ownership**: The Rust engine is the primary source of truth for **agent configurations**.
2.  **Tool Protocol**: All agent tools must implement the `Tool` trait and use `ToolContext`.
3.  **Zero-Trust**: No tool has ambient authority; always check for `CapabilityToken` in execution flows.
4.  **Sovereignty**: Enforce the **Oversight Gate** for all destructive file or network operations.
5.  **Rate Limiting**: Never bypass `RateLimiter.acquire()`. It is the only budget enforcement point.
6.  **Identity Governance** *(IDENTITY.md Directive #6)*: All agents operating in this codebase are bound by [`directives/IDENTITY.md v1.2.1`](../directives/IDENTITY.md). Consult it before any complex task. For error analysis, always check [`docs/ERROR_REGISTRY.json`](./ERROR_REGISTRY.json) first — it maps Sovereign error codes (`BUDGET_BREACH`, `STASIS_ACTIVE`, `LOGIC_BLOCKER`, `COMPLIANCE_DRIFT`) to source files and remediation paths.

---

## 🔍 Glossary & References

For a comprehensive breakdown of domain terminology (e.g., *Swarm*, *CBS*, *WAL*, *Oversight Gate*), refer to the [GLOSSARY.md](./GLOSSARY.md).

---

## 🔄 Anneal Changelog

| Date | Version | Changes |
|------|---------|--------|
| 2026-07-16 | 1.2.6 | **Swarm Hardening Sprint**: Cached system memory pressure in background thread to avoid syscalls in hot path; enforced micro-dollars precision (`i64` micros) internally for metering and quota management; limited concurrent scheduled jobs using Semaphore; optimized FsConnector walking logic and error handling. |
| 2026-07-11 | 1.2.5 | **ADG-04/05 Sprint**: Added `docstring_range` to `Symbol`/`SymbolNode`, Tauri IPC cross-language boundary mapping, Active Documentation Guard (`--diff`, `--fix`, `--strict`), Mermaid/Cytoscape blast radius visualizers, Jaro-Winkler auto-fix. Decomposed `graph_query.rs` monolith (1,393 lines) into 5 modules under `src/bin/graph_query/`. 379/379 tests pass. |
| 2026-06-23 | 1.2.4 | Integrated A2A Economics (Two-Phase Commit ledger) and the AI-Tadpole-OS Swarm Orchestration Engine (Fast-Path, OpenAI Completions Gateway, Context Sandboxing, Builder-Debugger slot swapping, and Conductor DAG planning). |
| 2026-06-11 | 1.2.3 | Modularized the workflow engine under `src/agent/continuity/workflow/`, hardened tournament template resolution, removed hardcoded encryption fallback, and added HMAC-SHA256 signature verification to stored contexts. |
| 2026-06-03 | 1.2.2 | Circular dep elimination (`agent_store` ↔ `agent_service` ↔ `socket`). CBS key rotation docs (`CAPABILITY_KEY_CURR/PREV`). Security scanner false-positive hardening (`.tmp/`, `coverage/`, self-exclusion, SQL pattern case-sensitivity). Env var coverage: `PROVIDER_TIMEOUT_SECS`, `TEST_PROVIDER_KEY`. |
| 2026-05-29 | 1.2.1 | IKS Route Parity, Scanner Policy Alignment, AI Assist Note Deduplication. |

[//]: # (Metadata: [ARCHITECTURE])
