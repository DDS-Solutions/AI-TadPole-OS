# 🗺️ Codebase Map (AI Navigator)

> **Intelligence Level**: Super AI-Awakened (Level 5)  
> **Status**: Verified Production-Ready  
> **Version**: 1.7.0  
> **Last Hardened**: 2026-04-10  
> **Classification**: Sovereign  

---

## 📖 Overview
This map serves as the definitive structural and navigational authority for Tadpole OS. It is designed following the **Hybrid Sovereign** framework—combining the structural clarity of Diátaxis with the high-fidelity semantic tokens required for Level 5 AI agent reasoning.

---

## 📂 Project Organization (Exploration)

Tadpole OS is architected as a high-performance **Sovereign Multi-Agent Swarm** platform. The system separates concerns into three distinct layers:

1.  **Directive Layer**: Instructions and SOPs (YAML/Markdown).
2.  **Orchestration Layer**: Intelligent routing and state management (Rust Axum / React Zustand).
3.  **Execution Layer**: Deterministic tool execution and sandboxed operations (Python/Bash).

### Top-Level Architecture
- **`server-rs/`**: The Core Engine. High-concurrency Rust backend managing agent lifecycles, vector memory, and oversight gates.
- **`src/`**: The Neural Dashboard. React 19 / Tailwind v4 frontend for real-time swarm visualization and mission management.
- **`workspaces/`**: Physical sandboxes where agents execute missions, isolated per cluster.
- **`execution/`**: The deterministic "Tool Belt" for agents (Standardized Python scripts).
- **`docs/`**: The Knowledge Hub (this directory).

---

## ⚙️ Coding Conventions (Sovereign Standards)

### Backend (Rust)
- **Concurrency**: Functional-style async concurrency using `tokio` and `FuturesUnordered`.
- **Error Handling**: RFC 9457 (Problem Details) compliant. All handlers return `anyhow::Result`.
- **State**: Single source of truth in `state.rs` (using `DashMap` for high-concurrency safety).
- **Redaction**: Regex-based secret redaction enforced at the log-stream boundary.

### Frontend (TypeScript/React)
- **Rendering**: React 19 (Server Components aware where applicable).
- **Styling**: Tailwind CSS v4 using CSS-first tokens.
- **State Mgmt**: Zustand stores with focused responsibilities (e.g., `header_store`, `skill_store`).
- **Safety**: Strict type safety; no usage of `any`.

---

## 🗺️ Detailed File Map (Reference)

### 🦀 Backend Hub: `server-rs/`

| Component | Path | Intelligence Signature |
| :--- | :--- | :--- |
| **Execution Core** | `src/agent/runner/mod.rs` | Mission lifecycle & budget gates. |
| **Modular Tools** | `src/agent/runner/*_tools.rs` | Refactored tool handlers (FS, Mission, External). |
| **Swarm Logic** | `src/agent/runner/swarm.rs` | Recursive task delegation and synthesis. |
| **Vector Memory** | `src/agent/memory.rs` | LanceDB RAG Scopes with Multi-Factor Scoring. |
| **Audit Hub** | `src/security/audit.rs` | Merkle Audit Trail (ED25519 Signed). |
| **Metering** | `src/security/metering.rs` | Persistent budget guards via SQLite. |
| **Discovery** | `src/services/discovery.rs` | mDNS node detection and broadcast. |
| **Telemetry** | `src/telemetry/pulse.rs` | 10Hz Binary MessagePack swarm pulse. |
| **Entry Point** | `src/main.rs` | Minimal Axum bootstrap. |

### ⚛️ Frontend Hub: `src/`

| Component | Path | Logic Layer |
| :--- | :--- | :--- |
| **Ops Dashboard** | `pages/Ops_Dashboard.tsx` | Real-time status grid & system logs. |
| **Swarm Pulse** | `components/Swarm_Visualizer.tsx` | Detachable Force-Directed Graph. |
| **API Bridge** | `services/tadpoleos_service.ts` | Unified proxy for backend communication. |
| **Reactive Stores** | `stores/*_store.ts` | Global state for agents, settings, and skills. |
| **Oversight UI** | `components/Swarm_Oversight_Node.tsx` | Floating floating decision modal. |

---

## 🚀 Key Entry Points (Reference)

- **Backend Startup**: `npm run engine` (Default Port: `8000`)
- **Frontend Startup**: `npm run dev` (Default Port: `5173`)
- **Primary Execution Hook**: `server-rs/src/agent/runner/mod.rs:execute()`
- **Identity Root**: `server-rs/data/context/IDENTITY.md`

---

## 🤖 AI Navigation Protocols (Agent-Only)

When navigating this codebase as an AI assistant, prioritize the following:
1.  **State Audit**: Always check `server-rs/src/state.rs` for live data structures.
2.  **Safety Boundary**: Never bypass `adapter/filesystem.rs` for I/O operations.
3.  **Traceability**: Use the `x-request-id` header to correlate frontend requests with backend telemetry.
4.  **Sovereignty Check**: Verify `PRIVACY_MODE` settings before suggesting cloud-based optimizations.

---

> [!TIP]
> **Dynamic Discovery**: For real-time route verification, run `python execution/verify_all.py`. This script cross-references the documentation map with the compiled Axum router.
