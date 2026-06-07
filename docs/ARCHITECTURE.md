> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Automated governance and architectural tracking.
>
> ### 🔍 Debugging & Observability
> Traceability via `execution/parity_guard.py`.


# 🏗️ Tadpole OS Architecture: Technical Hub

**Intelligence Level**: High (ECC Optimized)  
**Version**: 1.2.2  
**Last Hardened**: 2026-06-03 (Circular Dependency Elimination, CBS Key Rotation Docs, Security Scanner Hardening)
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

### 2. Intelligent Model Registry (IMR-01)
- **@docs ARCHITECTURE:Intelligence** (`capability_matrix.rs`)
- **@docs ARCHITECTURE:Agent** (`model_manager.rs`)
- Provides automated discovery and capability inference (Vision, Tools, Reasoning) based on model ID slugs.
- Includes provider handshake validation and secure secret redaction (SEC-02).

### 3. Swarm Persistence & Governance
- **@docs ARCHITECTURE:Persistence** (`persistence.rs`)
- **@docs ARCHITECTURE:Registry** (`recipes.rs`)
- **Agent Reaping**: Automated "Safety Valve" to harvest stale/crashed agent runs (LIF-03).
- **Heartbeat Propagation**: Real-time health tracking for long-running missions.
- **Hierarchical RBAC**: Resolves permission policies hierarchically: `Agent-Specific Policy` -> `Role-Based Policy` -> `Global Tool Policy` -> `Prompt (Sovereign Safety)`.
- **Role Blueprints**: Declarative templates for rapid swarm bootstrapping.

### 4. Swarm Intelligence & Autonomous Evolution
- **@docs ARCHITECTURE:Persistence** (`swarm_persistence.rs`)
- **@docs ARCHITECTURE:Agent** (`mission_tools.rs`)
- **Hierarchical Coordination**: Peer-to-peer directive delegation and multi-stage audit loops (Critic-Refiner).
- **Dynamic Tool Synthesis**: Autonomous generation and registration of Python/Node micro-scripts (Skills).
- **Turn Preservation Compaction (D)**: Dialogue-level context compactor that keeps the last 4 reasoning turns raw, truncates/evicts older turns' embedded code logs exceeding 2,000 characters, and falls back to a deterministic regex-based parser on LLM summarizer failure.

### 5. CodeBase Intelligence & Semantic Graph (MOD-03)
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

### 6. API Gateways & Router
- **@docs ARCHITECTURE:Gateways** (`routes/mod.rs`, `routes/intelligence.rs`, `router.rs`)
- **Route Delegation**: High-speed endpoint routing powered by Axum. Enforces Bearer token authentication via `NEURAL_TOKEN`.
- **Path Hardening**: Validates all inputs to avoid path-traversal vulnerabilities by verifying resource paths reside strictly within the user's `WORKSPACE_ROOT`.

### 7. Test Suite Isolation & Settings Store Verification
- **@docs ARCHITECTURE:TestSuites** (`stores/settings_store.test.ts`)
- **Test Sandbox Isolation**: Implements strict test isolation in `settings_store.test.ts` via `vi.hoisted` to mock/stub `localStorage` and `atob`/`btoa` globally before store re-evaluation, avoiding persistent state leakages across test cycles.

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
│   ├── src/intelligence/# CodeBase Graph & Blast Radius Analysis
│   ├── src/security/    # Merkle Audit, Metering, and Scanner
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

---

## 🔍 Glossary & References

For a comprehensive breakdown of domain terminology (e.g., *Swarm*, *CBS*, *WAL*, *Oversight Gate*), refer to the [GLOSSARY.md](./GLOSSARY.md).

---

## 🔄 Anneal Changelog

| Date | Version | Changes |
|------|---------|--------|
| 2026-06-03 | 1.2.2 | Circular dep elimination (`agent_store` ↔ `agent_service` ↔ `socket`). CBS key rotation docs (`CAPABILITY_KEY_CURR/PREV`). Security scanner false-positive hardening (`.tmp/`, `coverage/`, self-exclusion, SQL pattern case-sensitivity). Env var coverage: `PROVIDER_TIMEOUT_SECS`, `TEST_PROVIDER_KEY`. |
| 2026-05-29 | 1.2.1 | IKS Route Parity, Scanner Policy Alignment, AI Assist Note Deduplication. |

[//]: # (Metadata: [ARCHITECTURE])
 


[//]: # (Metadata: [ARCHITECTURE])
