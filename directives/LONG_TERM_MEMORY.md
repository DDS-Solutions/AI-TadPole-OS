> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[LONG_TERM_MEMORY]` in audit logs.
>
> ### AI Assist Note
> 🧠 Tadpole Engine: Persistent Ledger (Long-Term Memory)
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🧠 Tadpole Engine: Persistent Ledger (Long-Term Memory)
**Intelligence Level**: High (ECC Optimized)
**Source of Truth**: `server-rs/src/memory.rs`, `directives/LONG_TERM_MEMORY.md`
**Last Hardened**: 2026-07-13
**IDENTITY.md Sync**: v1.2.1
**Standard Compliance**: ECC-MEM (Enhanced Contextual Clarity - Memory Standards)


---

## 🧠 Memory Lifecycle & Retrieval

```mermaid
graph LR
    Event["New Insight (Event)"]
    Embed["Vectorize (Embedding)"]
    Store["LanceDB (Storage)"]
    Search["Cosine Search (Retrieval)"]
    Rerank["Heuristic Rerank (Context)"]

    Event --> Embed
    Embed --> Store
    Store -- "Query" --> Search
    Search --> Rerank
```

---

# Long-Term Memory (Persistent Ledger)

Last Updated: 2026.07.13

## Key Learnings
- **Doc Path Resolution**: Documentation files like `docs/wiki/Glossary.md` must be referenced using workspace-relative paths (e.g. `docs/wiki/Glossary.md`) rather than absolute OS paths to avoid Tauri sandbox security violations. (Verification: Direct file read successes after relative path correction in cl-command).
- **API Key Fallback Protocol**: When external API key errors (`invalid_api_key`) prevent subagent swarming, agents must pivot to centralized manual synthesis using local architecture models to maintain progress. (Verification: Successful SBOM extraction from context maps in cl-command).
- **Glossary Compliance Alignment**: Discrepancies between `docs/wiki/Glossary.md` (SSOT) and codebase structures must be documented in a structured `Glossary_Drift_Report.md` and routed to the `Alpha` node. (Verification: Final audit delegation in cl-command).
- **Sub-Agent Model Override Isolation**: Clearing parent model details in `TaskPayload` via `derive_subtask_payload` ensures spawned sub-agents resolve their own model configurations instead of inheriting parent settings. (Verification: Correct model resolution in recruitment routing tests).
- **Specialist Recruitment Dispatching**: Correctly resolving and propagating the `target_id` inside `ensure_sub_agent_exists` prevents dispatcher failures and guarantees that swarms execute specialist roles accurately. (Verification: Full cargo test suite verification passes).

- **Safe Multi-byte String Pruning**: Emojis and multi-byte UTF-8 sequences cause Rust's `String::truncate` to panic if sliced at arbitrary byte indices. Always align the index using `floor_char_boundary(limit)` before truncating. (Verification: Resolves crash in `server-rs::utils::serialization`).
- **Engine Crash Reconciliation**: Startup crash reconciliation logs panic details to SQLite and marks crashed missions as failed, ensuring post-mortem analysis availability. (Verification: Resolves stuck active missions).
- **Sovereign Execution Capabilities**: The agent is now fully authorized and capable of executing builds, running audit scripts, and performing architectural hardening directly within the Tadpole OS environment (Verification: Successful execution of `parity_guard.py`, `awaken.py`, and `sovereign_audit.py`).
- **High-Fidelity Context Retention**: Through the AI Awakening protocol, all core files now provide explicit architectural metadata to maintain consistent agent reasoning over long-term missions.
- **Master Release Gate**: The `sovereign_audit.py` script is now the source of truth for repository integrity.

- **Bunker Deployment**: Filesystem access is preferred over external DBs for air-gapped stability.
- **Agent Handoffs**: CEO-to-Alpha handoffs require explicit reasoning protocol injection.
- **Hook Registry**: Pre-tool hooks are mandatory for security auditing.

## Session Markers
- Initializing high-security agentOS enhancements.

[//]: # (Metadata: [LONG_TERM_MEMORY])
