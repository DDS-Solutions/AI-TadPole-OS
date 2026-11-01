---
name: database-design
description: "Database design principles, schema modeling, migration strategies, indexing, and ORM selection. Covers SQLite/sqlx, PostgreSQL/Neon, Turso, LanceDB vector memory, Prisma, and Drizzle."
when_to_use: "When designing database schemas, choosing ORMs, planning migrations, or optimizing queries. When working with Prisma, Drizzle, SQL migrations, or SQLite/sqlx schemas."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.2.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / database-design
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Suboptimal schema design, unindexed foreign keys, migration checksum divergence, SQLx VersionMismatch panics, or SQLite table locks.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Database Design — Schema Engineering & Persistence Protocol

> **Learn to THINK: Select the right persistence engine, schema model, and indexing strategy for the specific operational topology.**

> **Skill Synergy & Boundaries**:
> - **Dense Vector Memory & LanceDB**: Delegates vector tables, RAG, and cosine deduplication to [`vector-memory-lancedb`](../vector-memory-lancedb/SKILL.md).
> - **Domain Terminology & Glossaries**: Synchronizes entity names with [`domain-modeling`](../domain-modeling/SKILL.md).
> - **Rust Engine & Axum State**: Coordinates connection pool injection with [`rust-pro`](../rust-pro/SKILL.md).

---

## 🏛️ Tadpole OS Native Persistence Architecture

Within this codebase, persistence is governed by strict sovereign invariants:
1. **Sovereign Primary Engine**: Local-first embedded **SQLite** managed via async **`sqlx`** in Rust ([server-rs/Cargo.toml](../../../server-rs/Cargo.toml)).
2. **Deterministic Migration Pipeline**: Forward-only, timestamped SQL migrations located in [server-rs/migrations/](../../../server-rs/migrations) verified on startup.
3. **Dense Vector Memory**: **LanceDB** (`vector-memory` feature flag) for dense embeddings and semantic nearest-neighbor retrieval.
4. **External & Full-Stack Apps**: **Prisma** or **Drizzle** when provisioning external client web applications, edge nodes (Turso), or cloud Postgres (Neon/Supabase).

---

## 📚 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core standards below; consult deep **L3 Topic Guides** on demand:

| Guide | Topic & Scope | Trigger / When to Read |
|---|---|---|
| [`references/database-selection.md`](./references/database-selection.md) | SQLite vs PostgreSQL vs Neon vs Turso vs LanceDB | Selecting the persistence engine |
| [`references/orm-selection.md`](./references/orm-selection.md) | `sqlx` vs Drizzle vs Prisma vs SQLAlchemy 2.0 | Choosing query layer or ORM |
| [`references/schema-design.md`](./references/schema-design.md) | Normalization, ULID/UUID keys, relationships, soft deletes | Designing or restructuring entity schemas |
| [`references/indexing.md`](./references/indexing.md) | B-tree, Hash, GIN, HNSW vector, composite ordering | Performance tuning and query plan optimization |
| [`references/optimization.md`](./references/optimization.md) | N+1 mitigation, EXPLAIN QUERY PLAN, pagination | Query debugging and latency bottlenecks |
| [`references/migrations.md`](./references/migrations.md) | SQLx migration immutability, offline compilation cache (`sqlx-data.json`), WAL pragmas | Writing and deploying schema changes |
| [`scripts/schema_validator.py`](./scripts/schema_validator.py) | Deterministic automated schema validation script | Pre-commit schema & migration verification |

---

## ⚠️ Core Architectural Invariants

> **Domain Alignment Requirement**: Always consult and update `CONTEXT.md` via `@[skills/domain-modeling]` whenever introducing new entities, tables, or columns so database models match ubiquitous domain terminology.

1. **Explicit Foreign Key Indexing**: Every foreign key column (`xxx_id` / `xxxId`) MUST have a corresponding index to avoid table locks and sequential scans during JOINs.
2. **Standard Audit Timestamps**: Every persistent table must include UTC creation timestamps (`created_at` or `createdAt` using `TIMESTAMPTZ`, epoch milliseconds, or ISO-8601).
3. **Expand-Contract Migrations**: Breaking schema modifications must occur in non-breaking phases (Add nullable $\rightarrow$ Backfill $\rightarrow$ Deploy code $\rightarrow$ Enforce NOT NULL $\rightarrow$ Deprecate old).
4. **SQLx Forward-Only Migration Immutability**: In `server-rs/migrations/`, past migrations are cryptographically hashed by SQLx. **NEVER edit an applied migration file**. Always write a new monotonic timestamped migration (`YYYYMMDDHHMMSS_action.sql`) to avoid fatal `VersionMismatch` boot panics.
5. **SQLite WAL & Busy Timeout Invariants**: SQLite connection pools must execute `PRAGMA journal_mode = WAL;` and `PRAGMA busy_timeout = 5000;` on connection acquisition to prevent `database is locked` panics on concurrent Tokio writes.
6. **SQLx Offline Compilation (`sqlx-data.json`)**: Pre-generate query metadata with `cargo sqlx prepare` so CI and clean checkouts compile with `SQLX_OFFLINE=true` without requiring a live database.
7. **No Unbounded Queries**: Production queries must paginate at the database level with deterministic ordering (`ORDER BY ... LIMIT ... OFFSET` or keyset/cursor pagination). Avoid `SELECT *`.

---

## 📋 Pre-Design Checklist

Before committing schema definitions or migrations:
- [ ] Checked `CONTEXT.md` to ensure entity and column names match ubiquitous domain terms.
- [ ] Confirmed migrations in `server-rs/migrations/` are forward-only without modifying historical files.
- [ ] Verified that column names do not duplicate existing columns in target tables.
- [ ] Created explicit indices on all foreign key columns and frequently filtered columns.
- [ ] Enforced appropriate `ON DELETE` rules (`CASCADE`, `RESTRICT`, or `SET NULL`).
- [ ] Verified non-blocking migration strategy and SQLite WAL pragmas.
- [ ] Executed `schema_validator.py` on the modified schema files.

---

## 🚫 Critical Anti-Patterns

- ❌ **Modifying Applied Migrations**: Editing historical SQL files in `server-rs/migrations/` creates fatal SQLx `VersionMismatch` and checksum panics.
- ❌ **Unindexed Foreign Keys**: Creating `REFERENCES other_table(id)` without a companion index on the foreign key column.
- ❌ **Implicit N+1 Loops**: Iterating through parent records in application code and issuing a query for each child instead of eager JOIN or batching.
- ❌ **SELECT \* in Production**: Fetching unbounded rows and unneeded JSON blobs, wasting database I/O and network serialization.
- ❌ **Missing SQLite Busy Timeout**: Operating SQLite without `busy_timeout` leads to immediate runtime write aborts under light concurrency.
- ❌ **Destructive In-Place Alterations**: Dropping or renaming columns in a single migration without an expand-contract transition.

---

## 🛠️ Verification Commands

Run deterministic schema validation before submitting schema or migration changes:

```powershell
# 1. Run the internal validator self-test suite
python .agent/skills/database-design/scripts/schema_validator.py --test

# 2. Validate project schema and migrations
python .agent/skills/database-design/scripts/schema_validator.py server-rs/migrations

# 3. Prepare SQLx offline query metadata cache
cargo sqlx prepare --manifest-path server-rs/Cargo.toml -- --all-targets
```