> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / database-design
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Table locks, sequential full-table scans, write throughput degradation, sub-optimal index cardinality.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[indexing]`)

# Indexing Principles & Strategies

> Effective indexing accelerates reads while maintaining acceptable write throughput. Measure with EXPLAIN before and after adding indexes.

---

## 🎯 When to Create Indexes

```
Must Index:
├── Foreign Key Columns (Avoids full-table lockups during parent deletions & JOINs)
├── Columns in WHERE Clauses with high selectivity (e.g., status, email, user_id)
├── Columns in ORDER BY / GROUP BY clauses for hot queries
└── Columns with UNIQUE business constraints

Do NOT Index:
├── Low-cardinality columns queried in isolation (e.g., a boolean `is_active` across millions of rows)
├── Append-only, write-heavy ingestion logs where reads are infrequent batch exports
└── Small tables (< 500 rows) where sequential memory scans are faster than B-tree traversals
```

---

## 🔍 Index Types & Use Cases

| Index Type | Supported Engines | Primary Use Case | Performance Characteristics |
|---|---|---|---|
| **B-Tree** | SQLite, PostgreSQL, MySQL | General-purpose equality (`=`), range (`<`, `>`, `BETWEEN`), sorting (`ORDER BY`) | $O(\log N)$ lookups; versatile and default in almost all relational engines. |
| **Hash** | PostgreSQL, MySQL | Pure equality comparisons (`=`) | $O(1)$ lookups; cannot be used for range queries or sorting. |
| **GIN** (Generalized Inverted) | PostgreSQL | JSONB fields, array containment (`@>`), full-text search (`tsvector`) | Slower writes, fast lookups inside semi-structured data structures. |
| **GiST** (Generalized Search Tree) | PostgreSQL | Geometric data (PostGIS), range types, nearest-neighbor | Flexible tree structure for complex multi-dimensional data. |
| **HNSW / IVFFlat** | LanceDB, `pgvector` | Dense vector embeddings, cosine/L2 semantic similarity | Approximate Nearest Neighbor (ANN) search with adjustable recall/latency trade-off. |

---

## 📐 Composite Index Rules & Column Ordering

When creating composite indexes across multiple columns `(col_a, col_b, col_c)`, column ordering is paramount:

```
Optimal Column Sequence:
1. Equality Columns First  ➔ Columns filtered with exact equality (`WHERE status = 'ACTIVE'`)
2. Range Columns Next      ➔ Columns filtered with inequalities (`AND created_at >= '2026-01-01'`)
3. Sort Columns Last       ➔ Columns used in sorting (`ORDER BY priority DESC`)
```

> ⚠️ **The Left-to-Right Matching Rule**: A composite index on `(org_id, department_id, created_at)` can serve queries on `(org_id)` and `(org_id, department_id)`, but **CANNOT** accelerate queries filtering only on `(department_id)` or `(created_at)`.

---

## ⚡ Covering & Partial Indexes

1. **Covering Indexes (`INCLUDE`)**:
   In PostgreSQL, use `CREATE INDEX idx_users_lookup ON users(email) INCLUDE (first_name, role);` to allow Index-Only Scans without reading the heap table.
2. **Partial Indexes**:
   Index only the active subset of records to reduce index footprint and write penalty:
   ```sql
   -- Only index unresolved tasks
   CREATE INDEX idx_tasks_pending ON tasks(assigned_agent_id) WHERE status = 'PENDING';
   ```

---

## 🚫 Indexing Anti-Patterns

- ❌ **Missing Foreign Key Indexes**: Defining a foreign key without a companion index forces full table scans on parent table updates and cascades.
- ❌ **Over-Indexing Write-Heavy Tables**: Adding 10+ indexes to high-velocity telemetry or audit tables, severely degrading insert throughput.
- ❌ **Indexing Expressions Without Expression Indexes**: Filtering on `LOWER(email)` when the index is on `email` prevents index usage. Use `CREATE INDEX idx_users_lower_email ON users(LOWER(email));`.
- ❌ **Duplicate / Redundant Indexes**: Having an index on `(user_id)` when another index already exists on `(user_id, status)`.