> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[emergency_shutdown]` in audit logs.
>
> ### AI Assist Note
> 🛑 Directive: Emergency Shutdown (SOP-SYS-01)
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🛑 Directive: Emergency Shutdown (SOP-SYS-01)

## 🎯 Primary Objective
Halt all Tadpole OS activities immediately during a total system compromise, runaway resource consumption, or physical security breach. The goal is to protect core data integrity and financial assets.

---

## 🛠️ Execution SOP

### 1. Immediate Halt (The Kill Switch)
- **Engine Control**: Send a `SIGKILL` or use the `Stop-Process` command on the `server-rs` engine process.
- **Dashboard Control**: Close all active WebSocket connections to the Engine sidecar.

### 2. Resource Lockdown
- **Budget**: Set `DEFAULT_AGENT_BUDGET_USD=0` in `.env` and restart the engine briefly (if possible) to commit the state.
- **Privacy**: Enable `PRIVACY_MODE=true` to cut all external connectivity.

### 3. Data Protection
- **Vault**: Manually lock the `Neural Vault` in the dashboard or purge the `sessionStorage` in the browser.
- **Sandbox**: Purge all temporary mission files in `data/workspaces/`.

---

## 🛡️ Recovery Phases

### 1. Triage
- **Action**: Follow the `incident_response.md` protocol to identify the trigger for the shutdown.

### 2. Post-Mortem
- **Tool**: Extract the final 100 entries from the Merkle Audit Trail for forensic analysis.
- **Requirement**: The system may only be restarted after the root cause has been identified and neutralized.

---

## 🚦 Authorization
The Emergency Shutdown can be triggered by any authorized human overseer or automatically by the `Health Watchdog` if `failure_count` thresholds are exceeded globally.
[//]: # (Metadata: [emergency_shutdown])
