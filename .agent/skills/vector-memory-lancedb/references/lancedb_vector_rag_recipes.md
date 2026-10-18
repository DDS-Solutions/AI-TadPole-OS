# LanceDB Vector Memory & Semantic Retrieval Architecture (L3 Reference)

> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / vector-memory-lancedb / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Embedding dimension mismatch, drift threshold collapse, unindexed vector scans.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document details the configuration, embedding pipeline, and search parameters for Tadpole OS's local LanceDB vector memory.

---

## 1. System Invariants & Thresholds

Tadpole OS enforces specific environment thresholds for vector storage:
- **`LANCEDB_DEDUPE_THRESHOLD`** (default `0.92`): Documents with cosine similarity $\ge 0.92$ are merged or updated rather than duplicated.
- **`LANCEDB_DRIFT_THRESHOLD`** (default `0.35`): Queries with best distance $> 0.35$ trigger a low-confidence fallback flag.
- **Embedding Dimensions**: Must remain strictly uniform across a table (e.g. 768 for nomic-embed-text / Gemma embeddings, 1536 for text-embedding-3-small).

---

## 2. Table Creation & Ingestion Pattern

```python
import lancedb
import pyarrow as pa

schema = pa.schema([
    pa.field("id", pa.string()),
    pa.field("vector", pa.list_(pa.float32(), 768)),
    pa.field("text", pa.string()),
    pa.field("metadata", pa.string()),
    pa.field("timestamp", pa.string())
])

db = lancedb.connect("data/lancedb")
tbl = db.create_table("agent_memory", schema=schema, exist_ok=True)
```

---

## 3. Nearest-Neighbor Semantic Search

```python
results = tbl.search(query_vector) \
    .limit(5) \
    .metric("cosine") \
    .to_arrow()
```
