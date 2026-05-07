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

# 🤖 Agent Runner: The Intelligence Lifecycle

> **Intelligence Level**: High (Sovereign Context)  
> **Status**: Verified Production-Ready  
> **Version**: 1.2.1  
> **Last Hardened**: 2026-05-02 (Unified Tool Registry & Manifest-Driven Discovery)  

The **Agent Runner** is the stateful heart of Tadpole OS. It transforms a high-level intent into a chain of tactical successes through a disciplined "Intelligence Loop."

---

## 🛰️ The Intelligence Loop (Goal → Synthesis)

Every agent mission follows a deterministic 5-phase lifecycle to ensure repeatability and safety.

### Phase 1: Context Resolution
Before "Thinking" begins, the runner aggregates:
- **Unified Registry**: Dynamically discovers available capabilities from the `RegistryHub`.
- **Institutional Memory**: Global directives from `directives/`.
- **RAG Context**: Vector-retrieved findings from LanceDB.

### Phase 2: Reasoning & Discovery
The agent (LLM) evaluates the goal against its available tools, which are dynamically injected from the tool manifest.
- **Tool Mapping**: Resolving internal skills vs. external MCP tools via the dynamic registry.
- **Safety Pre-Check**: The runner validates the tool choice against the **Security Scanner**.

### Phase 3: Zero-Trust Execution Pipeline (SEC-04)
The tool is executed through a hardened pipeline:
1.  **WAL (Write-Ahead Log)**: Intent is persisted to the audit trail.
2.  **CBS (Capability-Based Security)**: A unique `CapabilityToken` is minted and verified.
3.  **Isolation**: The tool is executed within an isolated `ToolContext`.
4.  **Audit**: Success or failure is recorded in the final audit trail.

### Phase 4: Recursive Swarm Orchestration
Tadpole OS enables agents to recruit specialists to handle sub-tasks.
- **Recruitment**: Agent calls `recruit_specialist`.
- **Parallel Swarming**: Several sub-agents can be spawned and synchronized simultaneously.

### Phase 5: Synthesis & Learning
Final results are aggregated and distilled.
- **Mission Synthesis**: Merging sub-task outputs into a final deliverable.
- **Self-Annealing**: If a tool fails with a transient error, the engine utilizes structured `RecoveryActions` to attempt autonomous repair.

---

## 🏛️ Swarm Protocols

Tadpole OS employs standardized organizational patterns to reduce cognitive load and improve performance.

### 1. CEO (Agent of Nine)
- **Role**: Global strategy & Intent refinement.
- **Sovereignty**: The only node authorized to issue `alpha_directives`.

### 2. Alpha Node (Tadpole Alpha)
- **Role**: Tactical coordinator.
- **Mechanics**: Manages multiple sub-agents and synthesizes their results.

### 3. Specialist Node
- **Role**: High-fidelity tool execution (e.g., Engineer, Researcher).
- **Mechanics**: Minimal context, maximal efficiency on singular domains.

---

## 🛠️ Concurrency & Asynchronous Design

The runner is built on the **Tokio** runtime for non-blocking mission management.
- **Parallel Execution**: Multiple tool calls or recruitments are handled in parallel.
- **Self-Healing Retries**: Automated recovery from transient failures and malformed JSON payloads via the `ToolExecutionError` system.

---

## 🏛️ System Prompt Architecture

Tadpole OS implements a "Server-Side Synthesis" model. Agents do not provide their own system prompts; instead, the engine builds them dynamically based on the swarm state.

**Data Assembly Order:**
1.  **Agent Identity**: Name, Role, Department, and Description.
2.  **Hierarchy Level**: Determined by `swarm_depth`.
3.  **Skills & Workflows**: Standardized tool definitions scoped to the active "Neural Slot".
4.  **Working Context**: Persistent scratchpad reasoning (Working Memory).
5.  **Sovereign Directives**: Global identity and institutional memory files.

[//]: # (Metadata: [Agent_Runner_Workflow])
