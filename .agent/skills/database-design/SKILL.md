---
name: database-design
description: "Database design principles, schema modeling, migration strategies, indexing, and ORM selection. Covers SQLite/sqlx, PostgreSQL/Neon, Turso, LanceDB vector memory, Prisma, and Drizzle."
when_to_use: "When designing database schemas, choosing ORMs, planning migrations, or optimizing queries. When working with Prisma, Drizzle, SQL migrations, or SQLite/sqlx schemas."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / database-design
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Suboptimal schema design, unindexed foreign keys, migration divergence, N+1 query loops.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Database Design — Schema Engineering & Persistence Protocol

> **Learn to THINK: Select the right persistence engine, schema model, and indexing strategy for the specific operational topology.**

---

## 🏛️ Tadpole OS Native Persistence Architecture

Within this codebase, persistence is governed by strict sovereign invariants:
1. **Sovereign Primary Engine**: Local-first embedded **SQLite** managed via async **`sqlx`** in Rust ([server-rs/Cargo.toml](file:///D:/TadpoleOS-Dev/server-rs/Cargo.toml)).
2. **Deterministic Migration Pipeline**: Forward-only, timestamped SQL migrations located in [server-rs/migrations/](file:///D:/TadpoleOS-Dev/server-rs/migrations) verified on startup.
3. **Dense Vector Memory**: **LanceDB** (`vector-memory` feature flag) for dense embeddings and semantic nearest-neighbor retrieval.
4. **External & Full-Stack Apps**: **Prisma** or **Drizzle** when provisioning external client web applications, edge nodes (Turso), or cloud Postgres (Neon/Supabase).

---

## 📚 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core standards below; consult deep **L3 Topic Guides** on demand:

| Guide | Topic & Scope | Trigger / When to Read |
|---|---|---|
| [`references/database-selection.md`](file:///D:/TadpoleOS-Dev/.agent/skills/database-design/references/database-selection.md) | SQLite vs PostgreSQL vs Neon vs Turso vs LanceDB | Selecting the persistence engine |
| [`references/orm-selection.md`](file:///D:/TadpoleOS-Dev/.agent/skills/database-design/references/orm-selection.md) | `sqlx` vs Drizzle vs Prisma vs SQLAlchemy 2.0 | Choosing query layer or ORM |
| [`references/schema-design.md`](file:///D:/TadpoleOS-Dev/.agent/skills/database-design/references/schema-design.md) | Normalization, ULID/UUID keys, relationships, soft deletes | Designing or restructuring entity schemas |
| [`references/indexing.md`](file:///D:/TadpoleOS-Dev/.agent/skills/database-design/references/indexing.md) | B-tree, Hash, GIN, HNSW vector, composite ordering | Performance tuning and query plan optimization |
| [`references/optimization.md`](file:///D:/TadpoleOS-Dev/.agent/skills/database-design/references/optimization.md) | N+1 mitigation, EXPLAIN QUERY PLAN, pagination | Query debugging and latency bottlenecks |
| [`references/migrations.md`](file:///D:/TadpoleOS-Dev/.agent/skills/database-design/references/migrations.md) | Expand-contract zero-downtime migrations, rollback safety | Writing and deploying schema changes |
| [`scripts/schema_validator.py`](file:///D:/TadpoleOS-Dev/.agent/skills/database-design/scripts/schema_validator.py) | Deterministic automated schema validation script | Pre-commit schema & migration verification |

---

## ⚠️ Core Architectural Invariants

> **Domain Alignment Requirement**: Always consult and update `CONTEXT.md` via `@[skills/domain-modeling]` whenever introducing new entities, tables, or columns so database models match ubiquitous domain terminology.

1. **Explicit Foreign Key Indexing**: Every foreign key column (`xxx_id` / `xxxId`) MUST have a corresponding index to avoid table locks and sequential scans during JOINs.
2. **Standard Audit Timestamps**: Every persistent table must include UTC creation timestamps (`created_at` or `createdAt` using `TIMESTAMPTZ`, epoch milliseconds, or ISO-8601).
3. **Expand-Contract Migrations**: Breaking schema modifications must occur in non-breaking phases (Add nullable $\rightarrow$ Backfill $\rightarrow$ Deploy code $\rightarrow$ Enforce NOT NULL $\rightarrow$ Deprecate old).
4. **No Unbounded Queries**: Production queries must paginate at the database level with deterministic ordering (`ORDER BY ... LIMIT ... OFFSET` or keyset/cursor pagination). Avoid `SELECT *`.

---

## 📋 Pre-Design Checklist

Before committing schema definitions or migrations:
- [ ] Checked `CONTEXT.md` to ensure entity and column names match ubiquitous domain terms.
- [ ] Selected the correct database engine based on deployment target (embedded local vs cloud serverless).
- [ ] Confirmed primary key strategy (ULID / UUIDv7 for distributed or client-generated; auto-increment integer for purely local sequence).
- [ ] Created explicit indices on all foreign key columns and frequently filtered columns.
- [ ] Enforced appropriate `ON DELETE` rules (`CASCADE`, `RESTRICT`, or `SET NULL`).
- [ ] Verified non-blocking migration strategy (transactional DDL or `CREATE INDEX CONCURRENTLY`).
- [ ] Executed `schema_validator.py` on the modified schema files.

---

## 🚫 Critical Anti-Patterns

- ❌ **Unindexed Foreign Keys**: Creating `REFERENCES other_table(id)` without a companion index on the foreign key column.
- ❌ **Implicit N+1 Loops**: Iterating through parent records in application code and issuing a query for each child instead of eager JOIN or batching.
- ❌ **SELECT \* in Production**: Fetching unbounded rows and unneeded JSON blobs, wasting database I/O and network serialization.
- ❌ **Destructive In-Place Alterations**: Dropping or renaming columns in a single migration without an expand-contract transition.
- ❌ **Over-Indexing Write-Heavy Ledgers**: Adding redundant indexes to high-throughput append-only audit tables.

---

## 🛠️ Verification Commands

Run deterministic schema validation before submitting schema or migration changes:

```powershell
# Run the internal validator self-test suite
python .agent/skills/database-design/scripts/schema_validator.py --test

# Validate project schema and migrations
python .agent/skills/database-design/scripts/schema_validator.py server-rs/migrations

# Output structured JSON report
python .agent/skills/database-design/scripts/schema_validator.py server-rs/migrations --json
```