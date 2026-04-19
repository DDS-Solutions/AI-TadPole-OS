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

# 🧠 Knowledge & Memory: The Hybrid "Split-Brain" Architecture

> **@docs ARCHITECTURE:Retrieval**

Tadpole OS implements a powerful **Split-Brain Memory** system that separates deterministic data from high-dimensional semantic search.

---

## 🏗️ Memory Strategy: Deterministic vs. Semantic

Intelligence requires both exact record-keeping and fuzzy contextual intuition.

| Layer | Technology | Data Type | Purpose |
|:--- |:--- |:--- |:--- |
| **Deterministic** | **SQLite (sqlx)** | Logs, Budgets, Registry, Metadata | Legal Audit & Operational Truth |
| **Semantic** | **LanceDB (Vector)**| Embeddings (768-dim), RAG snippets | Contextual Awareness & Discovery |

---

## 🛰️ The Data Ingestion Pipeline (4-Phase Model)

The engine converts raw documents into "Intelligence Assets" through a disciplined background pipeline.

### Phase 1: Hybrid Retrieval & Reranking
Tadpole OS doesn't just rely on vector similarity. It uses a **Hybrid Scoring** engine:
1.  **Vector Search**: Uses cosine similarity for semantic mapping.
2.  **Keyword Proximity**: BM25-based keyword scoring reduces hallucination on jargon.
3.  **Heuristic Reranker**: Combines both signals into a unified relevance score before the agent sees the data.

### Phase 2: Automated Connectors (`connectors.rs`)
Data sources are automatically synchronized via background workers:
- **`IngestionWorker`**: A Tokio daemon that crawls configured folders.
- **Incremental Sync**: Uses MD5-checksums to avoid re-embedding unchanged files.
- **Connectors**: Supports File-system watching with absolute path validation.

### Phase 3: Deterministic SOP Workflows (`workflows.rs`)
Beyond fuzzy RAG, the engine can execute **Structured SOPs**:
- **Markdown-to-State**: Parses `.md` files into executable steps.
- **Sequential Execution**: Ensures compliance-heavy tasks follow a strict order of operations.

### Phase 4: Layout-Aware Parsing (`parser.rs`)
Ingested data is intelligently chunked to preserve meaning:
- **Semantic Overlap**: 25% chunk overlap ensures continuity across vector searches.
- **Metadata Preservation**: Every chunk is tagged with its origin (Filename, Section, Row-index).

---

## 🧪 Vector Memory Management

### LanceDB + Apache Arrow
- **Zero-Copy Performance**: Uses Apache Arrow memory layout for ultra-low latency searches.
- **Orphan Sweeping**: A background daemon purges mission-specific temporary vector "scopes" once tasks are complete to prevent disk bloat.

### Local-First Embeddings
By default, the engine utilizes the **BGE-Small-EN-v1.5** model via ONNX for zero-latency local vectorization, ensuring no data ever leaves the bunker for memory processing.
