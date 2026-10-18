---
name: vector-memory-lancedb
description: "Dense vector memory, LanceDB table schemas, embedding pipelines, cosine similarity deduplication, and semantic RAG retrieval."
when_to_use: "When working with LanceDB vector storage, generating embeddings, configuring semantic memory search, or tuning deduplication thresholds."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: critical
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / vector-memory-lancedb
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Embedding dimension mismatch, vector index corruption, high drift queries.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# LanceDB Vector Memory & Semantic Retrieval Protocol

> **Purpose**: Manage embedded vector storage, enforce deduplication thresholds, and power fast semantic RAG memory in Tadpole OS.
> **Workflow Binding**: Used during [`/refactor`](../../workflows/refactor.md) and [`/debug`](../../workflows/debug.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core standards below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide / Tool | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/lancedb_vector_rag_recipes.md`](./references/lancedb_vector_rag_recipes.md) | PyArrow schema definitions, cosine search parameters, indexing options (IVF-PQ) | Setting up vector tables or tuning semantic query recall |
| [`scripts/vector_memory_validator.py`](./scripts/vector_memory_validator.py) | Deterministic vector similarity validator and deduplication boundary checker | Verifying embedding calculations or self-test validation |

---

## 🏛️ Core Architectural Invariants

1. **Strict Dimensionality Matching**: All vectors ingested into a table must match the table's declared fixed dimension (e.g., 768 for Gemma/Nomic).
2. **Deduplication Threshold (`LANCEDB_DEDUPE_THRESHOLD`)**: Retain default `0.92`. If an incoming document vector has cosine similarity $\ge 0.92$ to an existing record, update the existing entry rather than creating duplicates.
3. **Drift Detection (`LANCEDB_DRIFT_THRESHOLD`)**: If the nearest vector distance exceeds `0.35`, flag the retrieval as low-confidence and trigger fallback search.

---

## 🛡️ Socratic Gate: Vector Ingestion & Query Tuning

Before executing vector operations:
1. **Model & Dimension Consistency**: Is the embedding model consistent across writes and reads?
2. **Batch Processing**: Are bulk document ingestions batched rather than committed per-record?
3. **Payload Sanitization**: Are text chunks stripped of raw secrets or sensitive tokens before embedding?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| Ingesting mismatched vector lengths | Crashes LanceDB table writes | Enforce fixed length schema verification before insert |
| Ingesting duplicates without dedupe check | Table bloat and degraded semantic recall | Check `LANCEDB_DEDUPE_THRESHOLD` before write |
| Unindexed linear scans on large tables | Latency spikes on semantic search | Create IVF-PQ or HNSW index when rows $> 10,000$ |

---

## 🛠️ Verification & Execution Commands

```powershell
# Run self-test suite
python .agent/skills/vector-memory-lancedb/scripts/vector_memory_validator.py --test
```
