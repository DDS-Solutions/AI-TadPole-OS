> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / database-design
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Production downtime, broken backward compatibility, locked tables during DDL, unmigrated schema drift.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[migrations]`)

# Migration Principles & Zero-Downtime Strategy

> Never deploy breaking database changes in a single step. All production schema evolutions must maintain backward compatibility.

---

## 🔄 The Expand-Contract (Parallel Run) Pattern

To evolve a live database schema with zero downtime, execute changes across three decoupled releases:

```
Step 1: EXPAND
├── Add new column as NULLABLE (or with harmless non-blocking default)
├── Deploy application code that reads from old column, writes to BOTH old and new (dual-writing)
│
Step 2: MIGRATE & BACKFILL
├── Backfill historical rows in small, rate-limited batches
├── Verify data integrity between old and new columns
│
Step 3: CONTRACT
├── Deploy code reading exclusively from the new column
├── Remove dual-writing logic
└── Drop old column or old table in a final non-blocking migration
```

---

## 🔒 Non-Blocking Schema Evolution Rules

| Operation | Dangerous Pattern | Safe Sovereign Alternative |
|---|---|---|
| **Adding a Column** | `ALTER TABLE tbl ADD col TEXT NOT NULL;` (Table rewrite & lock) | Add as `NULLABLE`, backfill, then add `NOT NULL` constraint. |
| **Creating an Index** | `CREATE INDEX idx ON tbl(col);` (Locks table against writes) | In PostgreSQL: `CREATE INDEX CONCURRENTLY idx ON tbl(col);` |
| **Renaming a Column** | `ALTER TABLE tbl RENAME COLUMN old TO new;` (Instant app crash) | Expand-Contract dual-write sequence. |
| **Dropping a Column** | `ALTER TABLE tbl DROP COLUMN col;` before deploying code | Stop using column in all code first, deploy, then drop column. |
| **Altering Column Types**| Direct type cast on multi-million row table | Create new column, backfill, cut over. |

---

## 🏛️ Tadpole OS Migration Standards (`sqlx` & Rust)

In this repository, database schema migrations live in [server-rs/migrations/](../../../../server-rs/migrations):

1. **Naming Standard**: Timestamped prefix strictly following `YYYYMMDDHHMMSS_descriptive_slug.sql` (e.g. `20261017000100_add_system_metrics_policy.sql`).
2. **Forward-Only Monotonic Evolution**: Migrations are strictly forward-only. Never modify an existing applied migration in git; author a new migration file to correct errors or alter schemas. Violating this triggers fatal SQLx `VersionMismatch` panics.
3. **No Duplicate Columns**: Always verify existing table columns via SQLite schema inspection before writing `ALTER TABLE ADD COLUMN` to prevent duplicate column execution panics.
4. **SQLite WAL & Connection Pragmas**:
   ```rust
   let pool = SqlitePoolOptions::new()
       .max_connections(10)
       .after_connect(|conn, _meta| Box::pin(async move {
           conn.execute("PRAGMA journal_mode = WAL;").await?;
           conn.execute("PRAGMA synchronous = NORMAL;").await?;
           conn.execute("PRAGMA busy_timeout = 5000;").await?;
           Ok(())
       }))
       .connect_with(options)
       .await?;
   ```
5. **SQLx Offline Compilation (`sqlx-data.json`)**:
   When developing offline or running CI without a live SQLite file:
   ```powershell
   # 1. Update offline cache when queries change:
   cargo sqlx prepare --manifest-path server-rs/Cargo.toml -- --all-targets

   # 2. Compile in offline mode:
   $env:SQLX_OFFLINE = "true"
   cargo check --manifest-path server-rs/Cargo.toml
   ```

---

## ☁️ Serverless & Cloud Database Considerations

### Neon (Serverless PostgreSQL)
- **Instant Branching**: Create isolated copy-on-write database branches for PR review and staging tests before applying migrations to production.
- **Connection Pooling**: Use Neon's pooled endpoint (`-pooler`) for serverless functions to prevent connection exhaustion.

### Turso (Edge libSQL)
- **Embedded Replicas**: Distribute read replicas globally to edge locations while channeling write migrations to the primary instance.
- **Durable Replication**: Ensure all edge nodes complete replication sync after DDL operations.

---

## 🚫 Migration Anti-Patterns

- ❌ **Editing Past Migrations**: Changing committed migration SQL files causes checksum mismatch errors across existing developer databases and live nodes.
- ❌ **Giant Monolithic Backfills**: Running `UPDATE big_table SET ...` across 1M+ rows in a single transaction blocks readers and bloats WAL logs.
- ❌ **Missing busy_timeout on SQLite**: SQLite locks immediately on concurrent writes unless a busy timeout of at least 5000ms is declared.
- ❌ **Tight Coupling of DDL and App Code**: Deploying code that immediately expects a column that has not yet finished migrating.