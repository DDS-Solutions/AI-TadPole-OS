> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Directives / emergency_shutdown
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[emergency_shutdown]`)

# 🛑 Directive: Emergency Shutdown (SOP-SYS-01)

## 🎯 Primary Objective
Halt all Tadpole OS activities immediately during a total system compromise, runaway resource consumption, or physical security breach. The goal is to protect core data integrity and financial assets.

---

## 🔀 STASIS vs. Emergency Shutdown — Decision Tree *(IDENTITY.md Directive #7)*

> [!IMPORTANT]
> These are TWO DISTINCT protocols. Using the wrong one causes either data loss (SIGKILL during a recoverable budget breach) or inaction (STASIS during a total compromise).

| Trigger | Correct Protocol | Who Resumes |
| :--- | :--- | :--- |
| `BudgetGuard` threshold breach | **STASIS mode** — suspend nodes, snapshot, alert Entity 0 | Entity 0 issues `RESUME` or `TERMINATE` |
| Total system compromise, sandbox escape, physical breach | **Emergency Shutdown (this SOP)** — `SIGKILL` | Root cause must be resolved before restart |
| Runaway agent loop (single agent) | `reset_agent` route — see `incident_response.md` Phase 2 | Automatic on reset confirmation |

> **Rule**: If in doubt, use `STASIS` first (non-destructive). Escalate to Emergency Shutdown only after Entity 0 confirms total compromise.

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