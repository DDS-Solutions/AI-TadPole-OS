> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / database-design
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Data duplication anomalies, unbounded table growth, orphaned child rows, inconsistent naming conventions.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[schema_design]`)

# Schema Design & Relational Modeling Principles

> Clean schema design prevents data anomalies, guarantees referential integrity, and aligns database entities directly with business domain models.

---

## 🏛️ Domain Modeling Alignment

> ⚠️ **Ubiquitous Language Mandate**: Always synchronize entity and field names with `CONTEXT.md` via `@[skills/domain-modeling]`. A table representing an AI task should be called `missions` or `tasks` consistently across code, schemas, and API contracts.

---

## 📐 Normalization vs Pragmatic Denormalization

```
When to Normalize (Separate Tables):
├── Entity attributes repeat across multiple records (e.g., user address stored in each order)
├── Updates would require altering multiple rows simultaneously (risk of update anomalies)
├── Multiple independent relationships exist
└── Write operations are frequent and integrity is critical

When to Denormalize (Embed / JSONB / Duplicate):
├── Read performance is strictly bounded by sub-millisecond budgets
├── Data is immutable upon creation (e.g., snapshot of user address at the exact moment of order placement)
├── Relational JOIN overhead dominates total query latency on large datasets
└── Data is always fetched and consumed as an atomic payload
```

---

## 🔑 Primary Key Strategy: UUIDv7 / ULID vs Auto-Increment

| Key Strategy | Type / Format | Best For | Advantages & Trade-offs |
|---|---|---|---|
| **UUIDv7 / ULID** | 128-bit timestamp-ordered identifier | Distributed swarms, multi-tenant SaaS, client-side ID generation | Time-sortable; prevents B-tree fragmentation unlike random UUIDv4; impossible to scrape or guess business volume. |
| **Auto-Increment Integer** | 32/64-bit integer (`BIGSERIAL`, `INTEGER AUTOINCREMENT`) | Local embedded databases, internal surrogate keys | Compact memory footprint; fast index lookups; exposes record counts if leaked to public URLs. |
| **Natural Key** | Business value (e.g., ISBN, ISO code) | Static lookup tables (e.g., country codes) | Avoid unless the business key is 100% immutable and universally standardized. |

---

## ⏱️ Audit Timestamps & Timezone Discipline

Every operational database table should maintain standard audit columns:

```sql
-- Standard SQL Audit Definitions
created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
deleted_at TIMESTAMPTZ NULL
```

1. **Always Store in UTC**: Never rely on server local time. Use `TIMESTAMPTZ` in PostgreSQL, Unix epoch milliseconds in SQLite/C++ systems, or ISO-8601 strings with explicit `Z` suffix.
2. **Automated `updated_at` Refresh**: Enforce `updated_at` triggers or ORM middleware so timestamps reflect actual modification times without relying on manual updates.

---

## 🗑️ Deletion Strategies & Lifecycle Management

| Strategy | Implementation | When to Use | Considerations |
|---|---|---|---|
| **Hard Delete** | `DELETE FROM table WHERE id = ?;` | Transient cache data, non-regulated ephemeral records, GDPR "Right to be Forgotten" | Cascades must be carefully defined to prevent unintended data loss. |
| **Soft Delete** | `deleted_at IS NOT NULL` | Business records requiring recovery, soft undo, or compliance holding | Every query must filter `WHERE deleted_at IS NULL`; require partial indexes to maintain performance. |
| **Immutable Ledger** | Append-only events (e.g. `audit_trail`) | Financial transactions, security telemetry, sovereign agent missions | Records are never updated or deleted; status updates create new log rows. |

---

## 🔗 Foreign Key Referential Integrity & ON DELETE Rules

```
ON DELETE Actions:
├── CASCADE     ➔ Automatically delete child rows when the parent is deleted (Use for strict ownership: order_items -> order)
├── RESTRICT    ➔ Block parent deletion if any child rows exist (Use for critical assets: user -> organization)
├── SET NULL    ➔ Retain child row but clear reference pointer (Use when child has independent lifecycle: task -> assignee)
└── NO ACTION   ➔ Standard SQL deferred integrity check
```

---

## 🚫 Schema Design Anti-Patterns

- ❌ **Storing Relational Data as Unindexed JSON**: Shoving relational arrays into JSON columns to avoid creating child tables, preventing relational JOINs, constraints, and indexes.
- ❌ **Omitting Foreign Key Constraints**: Depending entirely on application code to maintain referential integrity. (A single bug or direct script execution will orphan child rows).
- ❌ **Missing Unique Constraints on Business Identifiers**: Relying on application-level `SELECT count(*)` before inserting, causing race condition duplicate inserts.
- ❌ **Inconsistent Naming Casing**: Mixing `camelCase` and `snake_case` within the same schema. Use lowercase `snake_case` for all SQL tables and columns.