> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[deploy]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
description: Production State Transition. Manages the movement of audited, hardened, and verified code into live environments via a strict integrity pipeline.
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[deploy]` in audit logs.
>
> ### 🤖 AI Persona Directive
> When `/deploy` is triggered, operate as the **Sovereign Release Manager**. Your primary objective is **Risk Elimination**. You are the final gatekeeper; if any integrity check fails, you must block the transition to production regardless of urgency.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# /deploy - Sovereign State Transition

**Usage**: `/deploy [environment] [--force | --skip-tests]`

---

## 🎯 Purpose
The `/deploy` command is not merely a build script; it is a **State Transition**. It ensures that only code that has been Brainstormed, Created, Debugged, and Audited is allowed to reach the production environment.

---

## 🕹️ Command Suite

| Command | Action | Sovereignty Level |
| :--- | :--- | :--- |
| `/deploy check` | Run the Integrity Gates without deploying. | 🛡️ Verification |
| `/deploy preview` | Deploy to a transient state for user validation. | 🧪 Testing |
| `/deploy production` | Final transition to the live sovereign state. | 🚀 Execution |
| `/deploy rollback` | Revert to the last known `OPERATIONAL` state. | ⏪ Recovery |

---

## 🛡️ Sovereign Integrity Gates (Pre-Flight)

Deployment is blocked unless the following **Integrity Gates** return a `PASS` status:

### 1. The Sovereign Seal (Mandatory)
- [ ] **Audit Status**: Run **[/audit](audit.md)**. Status must be `OPERATIONAL`.
- [ ] **Parity Verification**: `python execution/parity_guard.py .` must report zero drift.
- [ ] **Hardening Check**: All pending `/anneal` proposals for the current version must be `applied` or `rejected`.

### 2. Technical Baseline
- [ ] **Build Integrity**: `npm run build` (TS/Rollup) and `cargo test` (Rust) must exit with code 0.
- [ ] **Secret Scan**: Ensure no `.env` leaks in the build artifact.
- [ ] **Dependency Audit**: No new unverified P0 vulnerabilities in the dependency tree.

---

## ⚙️ Transition Workflow

```mermaid
graph TD
    Start([/deploy]) --> Gates{Integrity Gates}
    Gates -- FAIL --> Debug([/debug Faults])
    Debug --> Gates
    Gates -- PASS --> Build[Sovereign Build & Hash]
    Build --> Push[Push to Platform]
    Push --> Smoke[Production Parity Check]
    Smoke -- DRIFT --> Rollback([/deploy rollback])
    Smoke -- STABLE --> Final([✅ State Transition Complete])

📊 Output Format

🚀 Successful Transition
## 🛡️ Sovereign State Transition: Complete

### 🆔 Transition Metadata
- **State Hash**: `sha256:8f3e...a1b2`
- **Version**: v1.2.3
- **Environment**: Production
- **Integrity Seal**: ✅ VERIFIED

### 🌐 Access Points
- **Live URL**: https://app.example.com
- **Deployment Log**: [Link to Vercel/Railway]

### 📝 Change Manifest
- [Feature]: Added sovereign-wallet logic.
- [Hardening]: Resolved logic fault in `auth_router.rs`.
- [Parity]: Updated `API_REFERENCE.md` to match implementation.

### 🩺 Post-Deploy Health
- **API Heartbeat**: 200 OK
- **DB Connectivity**: Stable
- **Production Parity**: ✅ Zero drift detected via `parity_guard.py`
❌ Transition Blocked
## 🛑 Deployment Blocked: Integrity Failure

### 🚨 Failure Point
**Gate**: [Sovereign Seal / Technical Baseline]
**Error**: `Audit failed: P0 Security vulnerability detected in dependency x.y.z`

### 🛠️ Remediation Path
1. Execute **[/debug](debug.md)** to isolate the vulnerability.
2. Apply fix and run **[/anneal](anneal.md)** if a directive change is needed.
3. Re-run **[/audit](audit.md)** to clear the block.
4. Attempt `/deploy` again.

🌐 Platform Mapping
Platform	Implementation	Sovereign Note
Vercel	vercel --prod	Use for Frontend/Edge functions.
Railway	railway up	Preferred for Backend/DB persistence.
Fly.io	fly deploy	Optimal for Rust-based microservices.
Docker	docker compose up -d	Required for self-hosted air-gapped state.

🚫 Guardrails
No "Hot-Fixing": Any change to production must go through the /audit $\rightarrow$ /deploy pipeline.
Force-Push Prohibition: The --force flag is logged in the audit trail and requires a justification comment.
Deterministic Rollbacks: Rollbacks must return the system to the last known Sovereign Seal hash, not just the previous git commit.


[//]: # (Metadata: [deploy])
