> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: server-management
description: Server management principles. Process, monitoring, scaling.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

# Server Management

**Boring is good.**

## Principles
1.  **Process**: PM2/Docker/Systemd. Auto-restart.
2.  **Monitoring**: Availability, Perf, Errors, Resources.
3.  **Logs**: Rotate, Structure (JSON), Levels.
4.  **Scaling**: Vertical (RAM) -> Horizontal (Instances) -> Auto.
5.  **Health**: HTTP 200 + Deps check.

## Security
- **Access**: SSH Keys only. No root login.
- **Firewall**: Min ports open.
- **Secrets**: Env vars.
- **Updates**: Unattended security patches.

## Troubleshooting Order
1.  **Status**: Running?
2.  **Logs**: Errors?
3.  **Resources**: Disk/RAM?
4.  **Network**: Ports/DNS?
5.  **Deps**: DB reachable?

---

## 🧠 Aletheia Reasoning Protocol (Reliability)

### 1. Generator (Failure Mode)
*   **Disk**: "Logs full?".
*   **Memory**: "Leak?".
*   **Network**: "DNS fail?".

### 2. Verifier (Capacity Planning)
*   **Load**: "Handle 2x traffic?".
*   **Backup**: "Restore tested?".
*   **Security**: "Port 22 exposed?".

### 3. Reviser (Automation)
*   **IaC**: "Update playbook, don't manual edit.".
*   **Alerts**: "Page on symptoms (Latency), not noise.".

---

## 🛡️ Security & Safety Protocol (Ops)

1.  **Access**: SSH Keys, MFA.
2.  **Secrets**: No `.env` in git.
3.  **Updates**: Auto-patch.
4.  **Privilege**: Run as `nobody`/`app`, never `root`.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
