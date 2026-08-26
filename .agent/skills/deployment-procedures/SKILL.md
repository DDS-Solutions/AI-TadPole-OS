---
name: deployment-procedures
description: Production deployment principles and decision-making. Safe deployment workflows, rollback strategies, and verification. Teaches thinking, not scripts.
when_to_use: "When deploying to production, planning rollback strategies, or setting up CI/CD pipelines. Use with /deploy workflow."
allowed-tools: Read, Glob, Grep, Bash
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / deployment-procedures
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Production Deployment & State Transition Protocol

> **Philosophy**: Never deploy unverified code. Always have an immediate, tested rollback path.
> **Workflow Binding**: Used directly during the [`/deploy`](../../workflows/deploy.md) workflow.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** deployment gates below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/rollback_and_verification_matrix.md`](./references/rollback_and_verification_matrix.md) | Platform runbooks (Docker/PM2/K8s), migration expand-and-contract | Staging-to-production transitions & rollbacks |

---

## 🚀 1. The 5-Phase Release Pipeline

```
1. PREPARE  ➔ Run `parity_guard.py` & unit tests; verify production build.
2. SNAPSHOT ➔ Backup persistent SQLite / database state.
3. RELEASE  ➔ Execute binary update / container rollout with live log tailing.
4. VERIFY   ➔ Query `/health` endpoint and inspect initial traffic telemetry.
5. CONFIRM  ➔ If error rate > 0.1% or panic detected ➔ ROLLBACK IMMEDIATELY.
```

---

## 🛑 2. Rollback Triggers (Non-Negotiable)

Initiate an automated or manual rollback if:
1. **Health Check Failure**: Service fails to return 200 OK within 15 seconds of boot.
2. **Panic / Crash Loop**: Engine emits panic logs or exits with non-zero exit code.
3. **Database Lock Contention**: SQLite WAL mode encounters database locked exceptions.

---

## 🛠️ 3. Verification Commands

```powershell
# 1. Verify build and link integrity
python execution/parity_guard.py .

# 2. Verify backend compiles cleanly
cargo check --manifest-path server-rs/Cargo.toml
```