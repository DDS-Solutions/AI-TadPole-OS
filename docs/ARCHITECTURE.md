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
> Traceability via `parity_guard.py`.

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is the high-fidelity semantic hub for the Tadpole OS Engine.
> - **@docs ARCHITECTURE:Hub**
> - **Telemetry Link**: Verified via `execution/parity_guard.py`.

# 🏗️ Tadpole OS Architecture: Technical Hub

**Intelligence Level**: High (ECC Optimized)  
**Version**: 1.2.1  
**Last Hardened**: 2026-05-02 (Unified Tool Registry & Manifest-Driven Discovery)  
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

### 2. Zero-Trust Tool Pipeline (SEC-04)
- **@docs ARCHITECTURE:Security** (`tools/mod.rs`)
- **Capability-Based Security (CBS)**: Replaces ambient authority with explicit, cryptographically signed permission tokens.
- **Write-Ahead Logging (WAL)**: Ensures all tool intents are persisted to the audit trail *before* execution begins.
- **Isolated Context**: Every tool executes within a constrained `ToolContext`, preventing side-channel access to global engine state.

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
- **Role Blueprints**: Declarative templates for rapid swarm bootstrapping.

### 4. Swarm Intelligence & Autonomous Evolution
- **@docs ARCHITECTURE:Persistence** (`swarm_persistence.rs`)
- **@docs ARCHITECTURE:Agent** (`mission_tools.rs`)
- **Hierarchical Coordination**: Peer-to-peer directive delegation and multi-stage audit loops (Critic-Refiner).
- **Dynamic Tool Synthesis**: Autonomous generation and registration of Python/Node micro-scripts (Skills).

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
│   ├── src/security/    # Merkle Audit, Metering, and Scanner
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

[//]: # (Metadata: [ARCHITECTURE])
