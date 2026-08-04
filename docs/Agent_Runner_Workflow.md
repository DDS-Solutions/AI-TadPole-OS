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
> **Version**: 1.3.0  
> **Last Hardened**: 2026-07-11 (A2A Economics Protocol, runner/tools sub-module, CapabilityToken CBS)

The **Agent Runner** is the stateful heart of Tadpole OS. It transforms a high-level intent into a chain of tactical successes through a disciplined "Intelligence Loop."

---

## 🛰️ The Intelligence Loop (Goal → Synthesis)

Every agent mission follows a deterministic 5-phase lifecycle to ensure repeatability and safety.

### Phase 1: Context Resolution
Before "Thinking" begins, the runner aggregates:
- **Unified Tool Registry**: Dynamically discovers available capabilities from the `RegistryHub` (`state/hubs/reg.rs`). Tools are resolved via `runner/tools/registry.rs` and `runner/tools/manifest.rs` — distinguishing internal skills from external MCP tools.
- **Institutional Memory**: Global directives from `directives/`.
- **RAG Context**: Vector-retrieved findings from LanceDB (`agent/knowledge_store/`).
- **Context Compaction**: Dialogue history is optimized dynamically in $O(N)$ linear time to strip large logs and keep the last 4 reasoning turns raw, protecting the context window from overflow.

### Phase 2: Reasoning & Discovery
The agent (LLM) evaluates the goal against its available tools, which are dynamically injected from the tool manifest.
- **Tool Mapping**: Resolving internal skills vs. external MCP tools via the dynamic registry.
- **Safety Pre-Check**: The runner validates the tool choice against the **Security Scanner**.

### Phase 3: Zero-Trust Execution Pipeline (SEC-04)
The tool is executed through a hardened pipeline implemented in `runner/tools/`:
1.  **WAL (Write-Ahead Log)**: Intent is persisted to the audit trail (`security/audit.rs`).
2.  **CBS (Capability-Based Security)**: A unique `CapabilityToken` is minted and verified (`runner/tools/capability.rs`).
3.  **Isolation**: The tool is executed within an isolated `ToolContext` (`runner/tools/mod.rs`).
4.  **Security Pre-Check**: The tool choice is validated against the **Security Scanner** (`runner/tools/security.rs`) before execution.
5.  **Audit**: Success or failure is recorded in the final audit trail.

### Phase 4: Recursive Swarm Orchestration
Tadpole OS enables agents to recruit specialists to handle sub-tasks.
- **Recruitment**: Agent calls `recruit_specialist`.
- **Parallel Swarming**: Several sub-agents can be spawned and synchronized simultaneously across up to `MAX_CLUSTERS = 10` active mission clusters running parallel swarm loops concurrently via independent `is_active` toggling.
- **Mode-Switched Utility Cluster Studio**: Team presets (`Team_Cluster_Preset`) are instantiated via the configuration studio with $O(1)$ agent lookup maps (`agent_map`), collaborator pre-fill options, and team badges (`🏷️ [TEAM: BADGE]`).
- **Completions Gateway Route**: Synchronous client requests sent to `/v1/agents/chat/completions` bypass UI coordination and are executed directly by the swarm runner, allowing external clients to query the swarm.

### Phase 5: Synthesis & Learning
Final results are aggregated and distilled.
- **Mission Synthesis**: Merging sub-task outputs into a final deliverable (`runner/finalize.rs`).
- **Dual-Trace Telemetry Swarm Analytics**: Interactive telemetry trace inspector (`Dual_Trace_Playback`) comparing baseline execution steps against 100% certified World Model state machines.
- **Self-Annealing & Model Fallback**: If a tool fails with a transient error, the engine uses structured `RecoveryAction` (`runner/tools/error.rs`) for autonomous repair. Persistent failures trigger a **model fallback** via `PRIVACY_FALLBACK_MODEL` and `model_routing.rs`, routing to a secondary/tertiary provider configuration to analyze, resolve, and verify the failure before resuming.
- **Mission Analysis (Agent 99)**: Post-mission debriefing is handled by `runner/analysis.rs`, which evaluates success criteria and generates prescriptive improvement insights.

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

## 🔧 runner/tools Sub-Module

The tool execution pipeline is implemented as a dedicated sub-module at `server-rs/src/agent/runner/tools/`. This was introduced after v1.2.1 to decouple security, dispatch, and capability logic from the main runner loop.

| File | Responsibility |
|---|---|
| `tools/mod.rs` | `ToolContext`, `ToolResult` — primary execution context types |
| `tools/registry.rs` | `recruit_specialist`, `alpha_directive` — swarm tool dispatch |
| `tools/dispatcher.rs` | Routes tool calls to skill, MCP, or internal handler |
| `tools/capability.rs` | `CapabilityToken` minting & CBS verification |
| `tools/security.rs` | Pre-execution security scanner & sandbox validation |
| `tools/manifest.rs` | Manifest-driven tool discovery & schema injection |
| `tools/error.rs` | `RecoveryAction` — structured self-healing error types |
| `tools/cache.rs` | Tool response caching layer |
| `tools/plugin.rs` | Plugin extension points |
| `tools/trait_tool.rs` | `ToolTrait` — unified interface for all tool types |
| `tools/macros.rs` | Tool definition macros |
| `tools/lease.rs` | `ToolLeaseGuard` — RAII lease wrapper for `ConflictManager` path locks |
| `tools/policy.rs` | `resolve_required_permission` — maps tool name + args → required `Permission` |
| `tools/validation.rs` | `extract_path_from_args` — path extraction and `validate_path` enforcement |
| `tools/mcp_wrapper.rs` | `McpToolWrapper` — adapts external MCP tools to the internal `ToolTrait` interface |
| `tools/system_tools.rs` | Built-in system tool implementations (filesystem, search, shell) |

---

## 💸 A2A Economics Protocol

Added post v1.2.1. Agents can issue economic transfers to each other using the Agent-to-Agent (A2A) payment protocol, supporting both local ledger and Web3 settlement.

| File | Responsibility |
|---|---|
| `runner/a2a_types.rs` | Core types: `Address` (Local/Web3), `Amount`, `EconomicZone` |
| `runner/a2a_ledger.rs` | `A2ATransactionCoordinator` — SQLite-backed ledger |
| `runner/a2a_router.rs` | `A2APaymentAdapter` trait — local + Web3 adapters |
| `runner/a2a_mailbox.rs` | Async message delivery between agents |

**Key properties:**
- ED25519-signed transfers (same key infrastructure as `security/audit.rs`)
- `Address::Local` for intra-swarm transfers, `Address::Web3` for blockchain settlement
- All transactions are checked against daily spend limit caps (including pending locks) via the `PaymentRouter`
- Envelopes sent via `A2AMailbox` preserve model reasoning traces and list of file artifacts. Mailbox delivery is network-capable: target agent IDs matching remote HTTP/HTTPS addresses are automatically signed and dispatched via reqwest.
- All incoming remote A2A HTTP envelopes are cryptographically validated by checking their signature (HMAC-SHA256) using the sender's key stored in the local `KEYRING` keyring.
- All transactions are persisted to the `transaction_ledger` and `transaction_locks` SQLite tables for audit trail

---

## 🛠️ Concurrency & Asynchronous Design

The runner is built on the **Tokio** runtime for non-blocking mission management.
- **Parallel Execution**: Multiple tool calls or recruitments are handled in parallel.
- **Self-Healing Retries**: Automated recovery from transient failures and malformed JSON payloads via the `ToolExecutionError` system.

---

## 🏛️ System Prompt Architecture

Tadpole OS implements a "Server-Side Synthesis" model. Agents do not provide their own system prompts; instead, the engine builds them dynamically in `runner/prompt_renderer.rs` based on the swarm state.

**Data Assembly Order:**
1.  **Agent Identity**: Name, Role, Department, and Description.
2.  **Hierarchy Level**: Determined by `swarm_depth` (enforced max: `MAX_SWARM_DEPTH` env var, default 5).
3.  **Skills & Workflows**: Standardized tool definitions scoped to the active model configuration (`ModelConfig`), resolved by `runner/tools/manifest.rs`.
4.  **Working Context**: Persistent scratchpad reasoning (Working Memory) from `agent/context_manager.rs`.
5.  **Sovereign Directives**: Global identity and institutional memory files from `directives/`. *(Primary authority: [`directives/IDENTITY.md v1.2.1`](../directives/IDENTITY.md) — read before any complex task.)*

---

## 📁 Implementation Map (v1.3.0)

```
server-rs/src/agent/runner/
├── mod.rs              — AgentRunner, IntelligenceOutput, run()
├── intelligence.rs     — LLM call orchestration
├── context.rs          — Agent context assembly
├── conductor.rs        — Multi-agent orchestration
├── swarm.rs            — FuturesUnordered parallel dispatch, swarm_depth
├── lifecycle.rs        — Agent state transitions
├── prompt_renderer.rs  — Server-side system prompt synthesis
├── provider.rs         — Provider selection & failover
├── analysis.rs         — Mission Analysis (Agent 99 / post-mission debrief)
├── refinement.rs       — Output refinement & quality gates
├── finalize.rs         — Mission synthesis & deliverable assembly
├── oversight.rs        — Governance gate integration
├── fs_tools.rs         — Filesystem tool handlers
├── external_tools.rs   — External API tool handlers
├── evolution_tools.rs  — Adaptive tool evolution
├── a2a_types.rs        — A2A Economics: Address, Amount, EconomicZone
├── a2a_ledger.rs       — A2A Economics: SQLite transaction coordinator
├── a2a_router.rs       — A2A Economics: payment adapters (local + Web3)
├── a2a_mailbox.rs      — A2A Economics: async agent message delivery
└── tools/
    ├── mod.rs          — ToolContext, ToolResult
    ├── registry.rs     — recruit_specialist, alpha_directive dispatch
    ├── dispatcher.rs   — Skill / MCP / internal routing
    ├── capability.rs   — CapabilityToken CBS
    ├── security.rs     — Security scanner pre-check
    ├── manifest.rs     — Manifest-driven tool discovery
    ├── error.rs        — RecoveryAction self-healing
    ├── cache.rs        — Tool response cache
    ├── plugin.rs       — Plugin extension points
    ├── trait_tool.rs   — ToolTrait unified interface
    ├── macros.rs       — Tool definition macros
    ├── lease.rs        — ToolLeaseGuard (RAII ConflictManager wrapper)
    ├── policy.rs       — resolve_required_permission (tool→Permission mapping)
    ├── validation.rs   — extract_path_from_args (SafePath enforcement)
    ├── mcp_wrapper.rs  — McpToolWrapper (external MCP → ToolTrait adapter)
    └── system_tools.rs — Built-in system tool implementations
```

[//]: # (Metadata: [Agent_Runner_Workflow])
