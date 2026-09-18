> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / deployment-procedures
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Unverified production migration or failed rollback sequence.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[DEPLOYMENT_PROCEDURES]`)

# Production Deployment & Rollback Runbook (L3)

---

## 1. Platform Deployment Runbooks

| Platform | Production Deployment Command | Rollback Procedure |
|---|---|---|
| **Vercel / Cloudflare** | `vercel --prod` / `wrangler deploy` | Instant instant rollback via dashboard or CLI deploy promotion |
| **Docker / Compose** | `docker compose pull && docker compose up -d --remove-orphans` | `docker compose rollback` or re-tag previous image SHA |
| **VPS + PM2** | `pm2 reload ecosystem.config.js --update-env` | `pm2 reload ecosystem.config.prev.js` |
| **Kubernetes** | `kubectl set image deployment/app app=app:v2` | `kubectl rollout undo deployment/app` |

---

## 2. Database Migration Safety Rules

1. **Expand and Contract Pattern**: Never drop or rename active columns in a single deploy. Add the new column (Expand), migrate data, deploy code reading the new column, then delete the old column in a subsequent release (Contract).
2. **Pre-Migration Snapshot**: Always take an automated snapshot of SQLite / PostgreSQL before running DDL commands.