# Mission Debrief Protocols & Wisdom Schemas

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / mission-analyst / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

## 1. Architectural Wisdom Taxonomy

When analyzing mission logs, Agent 99 categorizes insights into four sovereign domains:

| Category | Definition | Example Format |
|---|---|---|
| **`[PROVIDER]`** | Multi-LLM provider parameter quirks, token limits, and payload schemas | `- **Provider Parity**: Gemini models require 'google' as the provider key in TaskPayload, not 'gemini'.` |
| **`[CONCURRENCY]`** | Database locking, Tokio async task scheduling, race hazards, and worker contention | `- **SQLite Contention**: Enable WAL mode when 10+ agents are swarming to prevent busy timeouts.` |
| **`[IPC_TAURI]`** | Tauri v2 IPC command serializations, Rust-to-TypeScript payload boundaries | `- **IPC Serialization**: Tauri command returns require serde rename_all camelCase for JS bridges.` |
| **`[REGISTRY]`** | Memory, SQLite, or LanceDB vector storage schemas and retention rules | `- **Vector Indexing**: LanceDB drift threshold must stay below 0.05 during continuous ingestion.` |

---

## 2. Ingestion & Filtering Matrix

Distinguish ephemeral transaction noise from enduring architectural learnings:

```
[RAW MISSION LOG STREAM]
          │
          ├── (File read, tool invocation ok, HTTP 200) ──> [DISCARD: Transactional Noise]
          │
          ├── (Transient timeout resolved by retry)     ──> [RECORD: Metric Counter]
          │
          └── (Logic flaw discovered & fixed pattern)   ──> [SYNTHESIZE: Architectural Wisdom]
```

### Wisdom Quality Rubric
1. **Actionable**: A future agent encountering this exact situation must immediately know how to resolve it without trial and error.
2. **Deterministic Witness**: Must cite a specific mission ID, test case, or invariant benchmark verifying the resolution.
3. **Succinct**: Exactly 1–3 dense sentences. Zero conversational boilerplate.

---

## 3. Storage & Persistence Schema (`LONG_TERM_MEMORY.md`)

```markdown
### [YYYY-MM-DD] Mission <ID> Debrief Summary
- **Category**: [PROVIDER | CONCURRENCY | IPC_TAURI | REGISTRY]
- **Insight**: <Actionable technical observation>
- **Resolution**: <Concrete code pattern or configuration fix>
- **Witness**: <Verification test, benchmark ID, or commit hash>
```
