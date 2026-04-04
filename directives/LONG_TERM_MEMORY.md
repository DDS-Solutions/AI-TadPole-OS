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
- **Bunker Deployment**: Filesystem access is preferred over external DBs for air-gapped stability.
- **Agent Handoffs**: CEO-to-Alpha handoffs require explicit reasoning protocol injection.
- **Hook Registry**: Pre-tool hooks are mandatory for security auditing.

## Session Markers
- Initializing high-security agentOS enhancements.
