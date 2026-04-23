> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[database_migration]` in audit logs.
>
> ### AI Assist Note
> 🗄️ Directive: Database Migration (SOP-DEV-02)
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🗄️ Directive: Database Migration (SOP-DEV-02)

## 🎯 Primary Objective
Execute schema migrations on `tadpole.db` (SQLite) or `memory.lance` (Vector DB) with zero data loss and minimal engine downtime.

---

## 🛫 Pre-Flight Checklist
- [ ] **Backup**: Create a filesystem snapshot of `data/tadpole.db` and the `data/workspaces/` directory.
- [ ] **Lock**: Ensure no background `IngestionWorker` or `ContinuityScheduler` jobs are active.
- [ ] **Schema Check**: Validate the migration SQL against `docs/SYSTEM_SCHEMA.json`.

---

## 🚀 Migration SOP

### 1. Incremental Update
- **Tool**: `sqlx-cli` or internal `server-rs/migrations` engine.
- **Action**: Apply migrations sequentially. Never skip a version.

### 2. Integrity Verification
- **SQLite**: Check for foreign key consistency and index health.
- **Vector DB**: Perform a `count()` check on the `memory.lance` table to ensure record counts match pre-migration stats.

### 3. Rollback Procedure
- **Trigger**: If `server-rs` fails to initialize post-migration, or if `cargo test` fails on DB modules.
- **Action**: Restore the filesystem snapshot and log the migration failure in `logs/migration_errors.log`.

---

## 📊 Post-Migration Audit
Run `python execution/verify_all.py .` (Data Layer category) to confirm the new schema is correctly utilized by both the backend hubs and the frontend stores.
[//]: # (Metadata: [database_migration])
