# LanceDB Vector Memory & Semantic Retrieval Architecture (L3 Reference)

> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / vector-memory-lancedb / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Embedding dimension mismatch, drift threshold collapse, unindexed vector scans, table fragmentation.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document details the configuration, embedding pipeline, hybrid search, and maintenance routines for Tadpole OS's local LanceDB vector memory.

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

## 3. Hybrid Semantic & Full-Text Search (RRF)

When querying for exact symbols, function signatures, or error codes alongside semantic queries:

```python
# Hybrid search: Combines dense vector distance with BM25 full-text rank
results = tbl.search(query_text, query_type="hybrid") \
    .vector(query_vector) \
    .where("metadata LIKE '%core%'", prefilter=True) \
    .limit(5) \
    .to_arrow()
```

---

## 4. Maintenance & Compaction Routines

Ingesting numerous small batches causes file fragmentation and degrades query performance. Run compaction periodically:

```python
# 1. Compact small fragments into optimized parquet chunks
tbl.compact_files()

# 2. Prune old transaction logs and deleted rows (older than 7 days)
tbl.cleanup_old_versions(older_than=timedelta(days=7))
```

---

## 5. Deduplication Gate Before Write

```python
def upsert_with_dedupe(tbl, doc_id: str, text: str, vector: list[float], metadata: dict):
    # Check nearest neighbor similarity
    nearest = tbl.search(vector).limit(1).to_arrow()
    if len(nearest) > 0:
        dist = nearest["_distance"][0].as_py()
        similarity = 1.0 - dist
        if similarity >= 0.92:
            # Update existing entry rather than duplicating
            existing_id = nearest["id"][0].as_py()
            tbl.update(where=f"id = '{existing_id}'", values={"text": text, "metadata": str(metadata)})
            return existing_id

    tbl.add([{"id": doc_id, "vector": vector, "text": text, "metadata": str(metadata), "timestamp": "..."}])
    return doc_id
```
