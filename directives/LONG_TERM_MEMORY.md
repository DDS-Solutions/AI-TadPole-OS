# 🧠 Tadpole Engine: Persistent Ledger (Long-Term Memory)
**Intelligence Level**: High (ECC Optimized)
**Source of Truth**: `server-rs/src/memory.rs`, `directives/LONG_TERM_MEMORY.md`
**Last Hardened**: 2026-04-01
**Standard Compliance**: ECC-MEM (Enhanced Contextual Clarity - Memory Standards)

> [!IMPORTANT]
> **AI Assist Note (Memory Logic)**:
> This document governs the "Split-Brain" architecture of Tadpole OS.
> - **Primary Core**: SQLite handles relational metadata and logs.
> - **Neural Core**: LanceDB handles vector embeddings (Semantic Recall).
> - **Sync Policy**: All writes are debounced (10s) via `memory.rs`.

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

Last Updated: 2026.02.27

## Key Learnings
- **Sovereign Execution Capabilities**: The agent is now fully authorized and capable of executing builds, running audit scripts, and performing architectural hardening directly within the Tadpole OS environment (Verification: Successful execution of `parity_guard.py`, `awaken.py`, and `sovereign_audit.py`).
- **High-Fidelity Context Retention**: Through the AI Awakening protocol, all core files now provide explicit architectural metadata to maintain consistent agent reasoning over long-term missions.
- **Master Release Gate**: The `sovereign_audit.py` script is now the source of truth for repository integrity.

* **API Integration**: Leverage existing weather APIs for efficient data retrieval (Verification: Successful mission completion using API data)
* **Geolocation Handling**: Validate and standardize zipcode inputs for accurate location-based queries (Verification: Correct weather report for zipcode 19702)
* **Data Parsing**: Implement robust parsing mechanisms to extract relevant weather data from API responses (Verification: Accurate extraction of current condition and temperature)
* **Error Handling**: Implement comprehensive error handling to ensure mission success despite potential API or data inconsistencies (Verification: No errors reported during mission execution)
* **Response Formatting**: Utilize standardized output formats for clear and concise mission summaries (Verification: Successful generation of **FINAL SUMMARY** section)

- **Bunker Deployment**: Filesystem access is preferred over external DBs for air-gapped stability.
- **Agent Handoffs**: CEO-to-Alpha handoffs require explicit reasoning protocol injection.
- **Hook Registry**: Pre-tool hooks are mandatory for security auditing.

## Session Markers
- Initializing high-security agentOS enhancements.
