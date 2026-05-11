> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Indexing Principles
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# Indexing Principles

> When and how to create indexes effectively.

## When to Create Indexes

```
Index these:
├── Columns in WHERE clauses
├── Columns in JOIN conditions
├── Columns in ORDER BY
├── Foreign key columns
└── Unique constraints

Don't over-index:
├── Write-heavy tables (slower inserts)
├── Low-cardinality columns
├── Columns rarely queried
```

## Index Type Selection

| Type | Use For |
|------|---------|
| **B-tree** | General purpose, equality & range |
| **Hash** | Equality only, faster |
| **GIN** | JSONB, arrays, full-text |
| **GiST** | Geometric, range types |
| **HNSW/IVFFlat** | Vector similarity (pgvector) |

## Composite Index Principles

```
Order matters for composite indexes:
├── Equality columns first
├── Range columns last
├── Most selective first
└── Match query pattern
```

[//]: # (Metadata: [indexing])

[//]: # (Metadata: [indexing])
