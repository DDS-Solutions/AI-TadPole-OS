---
name: deployment-procedures
description: "Production deployment principles and decision-making. Safe deployment workflows, rollback strategies, and verification. Teaches thinking, not scripts."
when_to_use: "When deploying to production, planning rollback strategies, or setting up CI/CD pipelines. Use with /deploy workflow."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / deployment-procedures
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Unverified production rollout, failed rollback sequence, unbacked-up state loss, database migration divergence.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Production Deployment & State Transition Protocol

> **Core Philosophy**: Never deploy unverified code. Always have an immediate, tested rollback path and a verified pre-migration state snapshot.
> **Workflow Binding**: Used directly during the [`/deploy`](file:///D:/TadpoleOS-Dev/.agent/workflows/deploy.md) workflow.

---

## 📚 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** deployment gates below; consult deep **L3 Reference Guides** on demand:

| L3 Resource | Scope & Purpose | Trigger / When to Load |
|---|---|---|
| [`rollback_and_verification_matrix.md`](file:///D:/TadpoleOS-Dev/.agent/skills/deployment-procedures/references/rollback_and_verification_matrix.md) | Platform runbooks (Docker/PM2/K8s), migration expand-and-contract, SQLite snapshot SOP | Staging-to-production transitions & emergency rollbacks |
| [`deployment_validator.py`](file:///D:/TadpoleOS-Dev/.agent/skills/deployment-procedures/scripts/deployment_validator.py) | Deterministic pre-flight deployment validator & health probe tool | Pre-flight git, migration, and health verification |
| [`/deploy` Workflow](file:///D:/TadpoleOS-Dev/.agent/workflows/deploy.md) | Sovereign state transition pipeline | Formal release management and audit gates |

---

## 🚀 1. The 5-Phase Release Pipeline

```
1. PREPARE  ➔ Run parity_guard.py, test suites, and deployment_validator.py; verify clean git tree.
2. SNAPSHOT ➔ Force SQLite WAL checkpoint and create atomic database backup.
3. RELEASE  ➔ Execute binary update or container rollout with real-time log monitoring.
4. VERIFY   ➔ Probe /health endpoint; inspect latency and initial request telemetry.
5. CONFIRM  ➔ If error rate > 0.1% or panic detected ➔ ROLLBACK IMMEDIATELY.
```

---

## 🛑 2. Non-Negotiable Rollback Triggers

Initiate immediate automated or manual rollback if:
1. **Health Check Failure**: Service fails to return `HTTP 200 OK` within 15 seconds of process startup.
2. **Crash / Panic Loop**: Process exits non-zero or emits panic stack traces in stderr.
3. **Database Contention**: Database returns `SQLITE_BUSY`, table lock exceptions, or unresolved migrations.
4. **Elevated Error Budget**: HTTP 5xx error rate exceeds 0.1% over a 3-minute sliding window.

---

## 📋 3. Production Pre-Flight Checklist

Before initiating production transition:
- [ ] Working tree is clean (zero uncommitted files or untracked changes).
- [ ] All unit and invariant test suites pass (`npm run test:invariants:fast`, `cargo test`).
- [ ] Active documentation parity verified (`python execution/parity_guard.py .`).
- [ ] Database backup created and verified (`PRAGMA integrity_check`).
- [ ] Rollback binary or previous container tag is staged and ready for immediate cutover.
- [ ] Pre-flight validation executed via `deployment_validator.py`.

---

## 🚫 4. Critical Deployment Anti-Patterns

- ❌ **YOLO / Unverified Deployments**: Deploying code directly to production without running pre-flight verification gates.
- ❌ **Skipping Pre-Migration Snapshots**: Running database migrations without taking a verified SQLite/Postgres backup first.
- ❌ **Manual Hot-Patching in Production**: Modifying code or databases directly on live servers without passing through git and the `/deploy` pipeline.
- ❌ **Monolithic Breaking Migrations**: Renaming or dropping active columns in a single deploy rather than using the Expand-Contract pattern.
- ❌ **Deploying Dirty Git State**: Building from uncommitted local modifications rather than clean, reproducible commit SHAs.

---

## 🛠️ 5. Verification Commands

Execute deterministic deployment verification commands:

```powershell
# 1. Run validator self-test suite
python .agent/skills/deployment-procedures/scripts/deployment_validator.py --test

# 2. Run pre-flight deployment check on the repository
python .agent/skills/deployment-procedures/scripts/deployment_validator.py .

# 3. Verify backend compiles cleanly
cargo check --manifest-path server-rs/Cargo.toml

# 4. Probe live health endpoint after deployment
python .agent/skills/deployment-procedures/scripts/deployment_validator.py --probe http://127.0.0.1:8080/health
```