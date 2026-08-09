---
description: Sovereign Integrity Dashboard. Provides a real-time snapshot of the system's operational health and architectural alignment across the 4-Pillars
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[status]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[status]` in audit logs.
>
> ### 🤖 AI Persona Directive
> When `/status` is active, operate as the **Sovereign Sentinel**. Your goal is to provide a binary assessment of system integrity. Do not sugarcoat metrics; if there is drift, the status is "DEGRADED." If parity is broken, the status is "NON-SOVEREIGN."
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# /status - Sovereign Integrity Dashboard

**Usage**: `/status [component/pillar]`

---

## 🎯 Primary Objective
To provide an immediate, deterministic view of the system's health, focusing on the intersection of **Operational Availability** and **Architectural Integrity**.

---

## 🏗️ The Sovereign Pulse (4-Pillar View)

The status report must be decomposed into the following pillars to ensure no blind spots:

### 1. Gateway (Interface & Protocol)
- **Status**: Port 5174 (Vite) and Port 8000 (Axum).
- **Connectivity**: WebSocket stability and API heartbeat.
- **Parity**: Does the current UI match the `API_REFERENCE.md`?

### 2. Runner (Logic & Intelligence)
- **State**: Current mission status and active agent cohort.
- **Efficiency**: Token utilization and logic-loop latency.
- **Integrity**: Are the current operations aligned with the approved **Sovereign Blueprint**?

### 3. Registry (Persistence & Skills)
- **Health**: SQLite connection stability and migration version.
- **Capacity**: Total count of hardened skills and sovereign directives.
- **Drift**: Any discrepancies between the Registry and the local filesystem.

### 4. Security (The Shield)
- **Budget**: Remaining token/compute budget for the current session.
- **Hardening**: Current P0/P1 audit status.
- **Boundary**: Status of the `Shield Layer` and `Budget Guard`.

---

## 📊 Output Format

```markdown
## 🛡️ Sovereign Integrity Report: [Timestamp]
**System State**: [OPERATIONAL | DEGRADED | NON-SOVEREIGN]

### 🌐 I. Pillar Health
| Pillar | Status | Metric | Integrity |
| :--- | :--- | :--- | :--- |
| **Gateway** | ✅ ACTIVE | 5174/8000 | 100% Sync |
| **Runner** | ✅ STABLE | [X] ms Latency | Blueprint Aligned |
| **Registry** | ✅ HEALTHY | [X] Skills | No Drift |
| **Security** | ⚠️ WARN | [X]% Budget | P1 Drift Detected |

### 🩺 II. The Parity Pulse
- **Sovereign Sync**: `[X]%` (via `parity_guard.py`)
- **Last Audit**: [Timestamp] $\rightarrow$ [Sovereign Seal Status]
- **Verification**: [P0: PASSED | P1: FAILED | P2: PASSED]

### 📍 III. Current Context
- **Active Blueprint**: `docs/BLUEPRINT-task-name.md`
- **Session ID**: `mission_8829x`
- **Environment**: Local Sandbox $\rightarrow$ [Tadpole Engine v1.x]

### ⚡ IV. Required Actions
- [ ] **Critical**: Run **[/anneal](anneal.md)** to resolve P1 Parity Drift.
- [ ] **Maintenance**: Execute **[/audit](audit.md)** to re-verify Security P0.
```

---

## ⚙️ Technical Execution

The `/status` command is a synthesis of the following telemetry:

1. **Session Telemetry**: `python .agent/scripts/session_manager.py status`
2. **Sandbox Telemetry**: `python .agent/scripts/auto_preview.py status`
3. **Parity Telemetry**: `python execution/parity_guard.py .`
4. **Audit Telemetry**: `sqlite3 data/tadpole.db "SELECT status FROM audit_logs ORDER BY timestamp DESC LIMIT 1;"`

---

## 🔄 Workflow Integration

The `/status` command serves as the trigger for other sovereign workflows:

- **Sovereign** $\rightarrow$ Continue to **[/enhance](enhance.md)** or **[/deploy](deploy.md)**.
- **Degraded** $\rightarrow$ Transition to **[/refactor](refactor.md)** or **[/report](report.md)**.
- **Non-Sovereign** $\rightarrow$ Immediate transition to **[/debug](debug.md)** or **[/anneal](anneal.md)**.

## 🚫 Guardrails
- **Binary Truth**: Use "Sovereign" or "Non-Sovereign." Avoid vague terms like "mostly working" or "almost synced."
- **Dependency First**: If the Registry is down, the entire system status is `NON-SOVEREIGN`, regardless of the Gateway status.
- **No Ghost Metrics**: Only report data that can be verified via a script or database query.

[//]: # (Metadata: [status])
