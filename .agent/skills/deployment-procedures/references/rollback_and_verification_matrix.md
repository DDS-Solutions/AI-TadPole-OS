> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / deployment-procedures
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Unverified production migration, failed rollback sequence, unbacked-up data loss.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[DEPLOYMENT_PROCEDURES]`)

# Production Deployment & Rollback Runbook (L3)

> Comprehensive reference runbooks, rollback matrices, database snapshot protocols, and zero-downtime deployment strategies.

---

## 🚀 1. Platform Deployment & Rollback Runbooks

| Platform Topology | Production Release Command | Immediate Rollback Command | Health Verification Gate |
|---|---|---|---|
| **Tadpole OS Core Daemon** | `cargo build --release --manifest-path server-rs/Cargo.toml` | Revert to previous binary backup (`target/release/tadpole-daemon.prev`) | `curl -f http://127.0.0.1:8080/health` |
| **Tauri Desktop Client** | `npm run tauri:build` | Distribute previously signed binary release | UI boot and IPC handshake validation |
| **Docker / Docker Compose** | `docker compose pull && docker compose up -d --remove-orphans` | `docker compose down && docker compose up -d <previous-tag>` | `docker compose ps` & container exit status check |
| **Kubernetes (K8s)** | `kubectl set image deployment/tadpole-core app=tadpole:v1.2.3` | `kubectl rollout undo deployment/tadpole-core` | `kubectl rollout status deployment/tadpole-core` |
| **VPS + PM2** | `pm2 reload ecosystem.config.js --update-env` | `pm2 reload ecosystem.config.prev.js` | `pm2 show tadpole-core` & zero restart counter |
| **Vercel / Cloudflare** | `vercel --prod` / `wrangler deploy` | Instant instant rollback via dashboard or CLI deploy promotion | Edge HTTP 200 response within 5 seconds |

---

## 💾 2. Pre-Deployment Database Snapshot Protocols

Never execute a production migration without an immediate snapshot:

### SQLite (Tadpole OS Native Engine)
Before running schema migrations in [server-rs/migrations/](file:///D:/TadpoleOS-Dev/server-rs/migrations):
1. **Force WAL Checkpoint**:
   ```sql
   PRAGMA wal_checkpoint(TRUNCATE);
   ```
2. **Create Online Backup**:
   ```powershell
   # Copy SQLite database atomically using sqlite3 .backup
   sqlite3 data/tadpole.db ".backup 'data/backups/tadpole_backup_$(Get-Date -Format 'yyyyMMdd_HHmmss').db'"
   ```
3. **Verify Backup Integrity**:
   ```sql
   PRAGMA integrity_check;
   ```

### PostgreSQL (Cloud / Remote Clusters)
```bash
# Atomic custom-format snapshot
pg_dump -Fc -v -d "$DATABASE_URL" -f "backup_$(date +%Y%m%d_%H%M%S).dump"
```

---

## 🔄 3. Expand-Contract Zero-Downtime Migration SOP

When altering active schemas, avoid table lockups and breaking downstream services:

```
Release N (EXPAND):
├── 1. Add new column as NULLABLE (e.g. ALTER TABLE agents ADD COLUMN new_status TEXT;)
├── 2. Deploy application code that writes to BOTH old and new columns, reading from old.
│
Release N+1 (MIGRATE):
├── 3. Execute background batch script to backfill historical records.
├── 4. Verify data parity between old and new columns.
│
Release N+2 (CONTRACT):
├── 5. Deploy application code reading and writing solely to new column.
├── 6. In a subsequent migration, apply NOT NULL constraints or drop legacy column.
```

---

## 🛑 4. Automated Rollback Triggers & Fast Revert Sequence

### Fast Revert Triggers:
1. **Health Check Timeout**: `/health` fails to return `HTTP 200 OK` within 15 seconds of process launch.
2. **Panic / Crash Loop**: Process exits non-zero or generates stack trace panics.
3. **Elevated Error Budget**: HTTP 5xx error rate exceeds 0.1% over a 3-minute sliding window.
4. **Database Lock Contention**: SQLite queries return `SQLITE_BUSY` or deadlock exceptions.

### Fast Revert Execution Protocol:
```
1. HALT      ➔ Stop the active deployment pipeline immediately.
2. REVERT    ➔ Execute platform-specific rollback command (binary swap, container rollback, or git revert).
3. RESTORE   ➔ If database migration failed or corrupted state, restore pre-migration snapshot.
4. RE-PROBE  ➔ Validate /health endpoint returns 200 OK.
5. POST-MORTEM➔ Trigger /debug and /anneal workflows to isolate the root cause.
```