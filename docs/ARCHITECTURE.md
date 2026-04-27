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
**Version**: 1.1.18  
**Last Hardened**: 2026-04-27 (State & Persistence Governance)  
**Standard Compliance**: ECC-ARA (Enhanced Contextual Clarity)

---

## 🎯 Executive Summary

Tadpole OS is a sovereign, high-performance runtime for multi-agent swarms. It utilizes a **Gateway-Runner-Registry** pattern implemented in Rust to provide a local-first, privacy-respecting intelligence environment.

> [!TIP]
> **New to the codebase?** Start with the [Architecture Overview](./Architecture_Overview.md) for a 3-paragraph executive brief and core system topology.

---

## 🛰️ Documentation Suite

To maintain technical clarity and reduce information density, the architecture is decomposed into the following specialized modules:

| Module | Description | Key Technologies |
|:--- |:--- |:--- |
| **[🏗️ Overview](./Architecture_Overview.md)** | Core topology and system philosophy. | Axum, Mermaid, Sovereign Layer |
| **[🛡️ Security Model](./Security_Model.md)** | Governance, Merkle trails, and Oversight. | ED25519, SubtleCrypto, Audit |
| **[🤖 Agent Runner](./Agent_Runner_Workflow.md)** | Execution lifecycle and Swarm protocols. | Tokio, LLM Dispatch, Recruitment |
| **[🧠 Memory & RAG](./Knowledge_Memory.md)** | Hybrid memory strategy and ingestion. | LanceDB, SQLite, BM25 Reranking |
| **[⚛️ Client Architecture](./Frontend_State_Management.md)** | Frontend state and multi-window sync. | React 19, Zustand, Portals |

---

## ⚙️ Core Engine Subsystems (server-rs)

The Rust engine implements several high-fidelity subsystems to ensure sovereign stability and intelligence.

### 1. Intelligent Model Registry (IMR-01)
- **@docs ARCHITECTURE:Intelligence** (`capability_matrix.rs`)
- **@docs ARCHITECTURE:Agent** (`model_manager.rs`)
- Provides automated discovery and capability inference (Vision, Tools, Reasoning) based on model ID slugs.
- Includes provider handshake validation and secure secret redaction (SEC-02).

### 2. Swarm Persistence & Governance
- **@docs ARCHITECTURE:Persistence** (`persistence.rs`)
- **@docs ARCHITECTURE:Registry** (`recipes.rs`)
- **Agent Reaping**: Automated "Safety Valve" to harvest stale/crashed agent runs (LIF-03).
- **Heartbeat Propagation**: Real-time health tracking for long-running missions.
- **Role Blueprints**: Declarative templates for rapid swarm bootstrapping.
- **Credential Polarization**: Environment variable priority over disk-based JSON (SEC-02).

### 3. Multi-Factor Scoring (MFS)
- **@docs ARCHITECTURE:Retrieval** (`rag_scoring.rs`)
- Enhances RAG relevance by combining semantic similarity, mission affinity, and temporal recency.

### 4. Domain Merge Engine
- **@docs ARCHITECTURE:Domain** (`merge.rs`)
- Authoritative state transition engine for `EngineAgent` status, missions, and telemetry resets.

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
├── directives/         # Layer 1: OS Identity and Rules
├── execution/          # Layer 3: Deterministic Tools & Skills
│   └── agent_generated/ # Provenance Barrier: Agent-authored code
├── server-rs/          # Layer 2: Rust Orchestration Engine
│   ├── src/agent/       # Core Runner, RAG, and Persistence
│   ├── src/security/    # Merkle Audit, Metering, and Scanner
│   └── src/state/       # Modular AppState Hubs
├── docs/               # Technical Specification Suite
├── data/               # Persistent State (SQLite, Vector DB)
└── src/                # Frontend (React/Zustand)
```

---

## 🤖 Context for AI Assistants

1.  **State Ownership**: The Rust engine is the primary source of truth for **agent configurations**.
2.  **Tool Protocol**: All agent tools must return an `anyhow::Result<String>`.
3.  **Workspace Roots**: Never hardcode workspace paths; always use `ctx.workspace_root`.
4.  **Sovereignty**: Enforce the **Oversight Gate** for all destructive file or network operations.
5.  **Rate Limiting**: Never bypass `RateLimiter.acquire()`. It is the only budget enforcement point.

---

## 🔍 Glossary & References

For a comprehensive breakdown of domain terminology (e.g., *Swarm*, *Bunker*, *Oversight Gate*), refer to the [GLOSSARY.md](./GLOSSARY.md).

[//]: # (Metadata: [ARCHITECTURE])

[//]: # (Metadata: [ARCHITECTURE])
