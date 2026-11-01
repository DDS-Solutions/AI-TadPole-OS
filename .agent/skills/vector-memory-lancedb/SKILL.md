---
name: vector-memory-lancedb
description: "Dense vector memory, LanceDB table schemas, hybrid RAG retrieval, cosine similarity deduplication, and table compaction."
when_to_use: "When working with LanceDB vector storage, generating embeddings, configuring semantic memory search, tuning deduplication thresholds, or compacting vector fragments."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.2.0"
priority: critical
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / vector-memory-lancedb
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Embedding dimension mismatch, vector index corruption, high drift queries, fragment file bloat.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# LanceDB Vector Memory & Semantic Retrieval Protocol

> **Purpose**: Manage embedded vector storage, enforce deduplication thresholds, execute hybrid dense+FTS retrieval, and power fast semantic RAG memory in Tadpole OS.  
> **Skill Synergy & Boundaries**:
> - **Relational Data & SQLite / SQLx**: Delegates to [`database-design`](../database-design/SKILL.md).
> - **Local LLM Models (Gemma 4)**: Delegates to [`gemma4-local`](../gemma4-local/SKILL.md).
> - **Workflow Bindings**: Integrated into [`/refactor`](../../workflows/refactor.md) and [`/debug`](../../workflows/debug.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core standards below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide / Tool | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/lancedb_vector_rag_recipes.md`](./references/lancedb_vector_rag_recipes.md) | PyArrow schemas, hybrid search (Dense + BM25) with RRF, fragment compaction, Arrow pushdown filtering | Setting up vector tables or tuning semantic query recall |
| [`scripts/vector_memory_validator.py`](./scripts/vector_memory_validator.py) | Deterministic vector similarity validator and deduplication boundary checker | Verifying embedding calculations or self-test validation |

---

## 🏛️ Core Architectural Invariants

1. **Strict Dimensionality Matching**: All vectors ingested into a table must match the table's declared fixed dimension (e.g. 768 for Gemma/Nomic).
2. **Deduplication Threshold (`LANCEDB_DEDUPE_THRESHOLD`)**: Retain default `0.92`. If an incoming document vector has cosine similarity $\ge 0.92$ to an existing record, update the existing entry rather than creating duplicates.
3. **Drift Detection (`LANCEDB_DRIFT_THRESHOLD`)**: If the nearest vector distance exceeds `0.35`, flag the retrieval as low-confidence and trigger fallback hybrid search.
4. **Hybrid Search Over Pure Vector**: When querying for code identifiers, error codes (e.g. `SEC-03`), or specific symbols, combine dense cosine search with full-text search (BM25) via Reciprocal Rank Fusion (RRF).
5. **Scheduled Table Compaction**: Ingesting many small batches creates fragmented parquet files. Trigger `table.compact_files()` and `table.cleanup_old_versions()` after batch ingestions to maintain $< 10\text{ ms}$ search latency.

---

## 🛡️ Socratic Gate: Vector Ingestion & Query Tuning

Before executing vector operations:
1. **Model & Dimension Consistency**: Is the embedding model consistent across writes and reads?
2. **Batch Processing**: Are bulk document ingestions batched rather than committed per-record?
3. **Payload Sanitization**: Are text chunks stripped of raw secrets or sensitive tokens before embedding?
4. **Exact Symbol Lookup**: If the query contains an exact code identifier, is full-text search enabled alongside dense vector search?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| Ingesting mismatched vector lengths | Crashes LanceDB table writes | Enforce fixed length schema verification before insert |
| Ingesting duplicates without dedupe check | Table bloat and degraded semantic recall | Check `LANCEDB_DEDUPE_THRESHOLD` before write |
| Pure vector search on exact symbol names | Misses exact identifier matches | Use hybrid search (Dense + BM25) with RRF |
| Uncompacted fragmented parquet files | Latency degradation over time | Schedule `compact_files()` and `cleanup_old_versions()` |
| In-memory post-filtering on large tables | High memory allocation and slow response | Pushdown predicate filter: `tbl.search().where(...)` |

---

## 🛠️ Verification & Execution Commands

```powershell
# Run self-test suite
python .agent/skills/vector-memory-lancedb/scripts/vector_memory_validator.py --test
```
