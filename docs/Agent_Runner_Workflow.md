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

> **@docs ARCHITECTURE:AgentExecutionRuntime**

The **Agent Runner** is the stateful heart of Tadpole OS. It transforms a high-level intent into a chain of tactical successes through a disciplined "Intelligence Loop."

---

## 🛰️ The Intelligence Loop (Goal → Synthesis)

Every agent mission follows a deterministic 4-phase lifecycle to ensure repeatability and safety.

### Phase 1: Context Resolution
Before "Thinking" begins, the runner aggregates:
- **Registry State**: The agent's identity and assigned skills.
- **Strategic Protocols**: The parent's current intent (if recruited).
- **Institutional Memory**: Global directives from `directives/`.
- **RAG Context**: Vector-retrieved findings from LanceDB.

### Phase 2: Reasoning & Dispatch
The agent (LLM) evaluates the goal against its available tools.
- **Tool Mapping**: Resolving internal skills vs. external MCP tools.
- **Safety Pre-Check**: The runner validates the tool choice against the **Oversight Gate** and **Security Scanner**.

### Phase 3: Recursive Swarm Orchestration
Tadpole OS enables agents to recruit specialists to handle sub-tasks.
- **Recruitment**: Agent calls `recruit_specialist`.
- **Intent Injection**: The parent's specific "Strategic Thought" is injected into the child's prompt.
- **Parallel Swarming**: Using `FuturesUnordered`, several sub-agents can be spawned and synchronized simultaneously.

### Phase 4: Synthesis & Learning
Final results are aggregated and distilled.
- **Mission Synthesis**: Merging sub-task outputs into a final deliverable.
- **Wisdom Write-Back (Agent 99)**: Key technical discoveries are summarized and appended to `LONG_TERM_MEMORY.md`.

---

## 🏛️ Swarm Protocols

Tadpole OS employs standardized organizational patterns to reduce cognitive load and improve performance.

### 1. CEO (Agent of Nine)
- **Role**: Global strategy & Intent refinement.
- **Sovereignty**: The only node authorized to issue `alpha_directives`.

### 2. Alpha Node (Tadpole Alpha)
- **Role**: Tactical coordinator.
- **Mechanics**: Manages multiple sub-agents and synthesizes their results into a final workspace report.

### 3. Specialist Node
- **Role**: High-fidelity tool execution (e.g., Engineer, Researcher).
- **Mechanics**: Minimal context, maximal efficiency on singular domains.

---

## 🛠️ Concurrency & Asynchronous Design

The runner is built on the **Tokio** runtime for non-blocking mission management.
- **Parallel Execution**: Multiple tool calls or recruitments are handled in parallel via `FuturesUnordered`.
- **Shared Connection Pool**: A single `reqwest::Client` is reused across all LLM providers and web-fetch tasks, eliminating TLS handshake overhead.
- **Self-Healing Retries**: Automated recovery from rate-limits and malformed JSON payloads.

---

## 🏛️ System Prompt Architecture

Tadpole OS implements a "Server-Side Synthesis" model. Agents do not provide their own system prompts; instead, the engine builds them dynamically based on the swarm state.

**Data Assembly Order:**
1.  **Agent Identity**: Name, Role, Department, and Description.
2.  **Hierarchy Level**: Determined by `swarm_depth` (OVERLORD, ALPHA NODE, etc.).
3.  **Recruitment Lineage**: The full recruitment chain (e.g., `A -> B -> C`).
4.  **Skills & Workflows**: Standardized tool definitions scoped to the active "Neural Slot".
5.  **Working Context**: Persistent scratchpad reasoning (Working Memory).
6.  **Sovereign Directives**: Global identity and institutional memory files.
