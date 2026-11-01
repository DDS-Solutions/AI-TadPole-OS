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

## 🔀 Budget Exhaustion vs. Emergency Shutdown — Decision Tree *(IDENTITY.md Directive #7)*

> [!IMPORTANT]
> The runtime has per-agent and per-mission budget quotas, but no global STASIS mode. Budget exhaustion rejects the affected provider call; any wider pause is a manual operator action.

| Trigger | Correct Protocol | Who Resumes |
| :--- | :--- | :--- |
| `BudgetGuard` rejects a provider-call reservation | Review the affected run and quota; use the engine emergency-stop control if a broader pause is needed. No global suspension, budget snapshot, or STASIS lockout is created automatically. | Operator reviews the quota and explicitly resumes or retries the affected work. |
| Total system compromise, sandbox escape, physical breach | Use the engine emergency-stop control to cancel registered work and persist dispatch pause. Terminate the server process at the OS level only if the control plane is compromised or unresponsive. | Root cause must be resolved; an authorized operator releases the persisted pause. |
| Runaway agent loop (single agent) | Stop or suspend the affected agent and follow `incident_response.md`; `reset_agent` is an explicit operator action, not an automatic recovery. | Operator confirms reset after review. |

> **Rule**: Budget exhaustion is not a system-wide shutdown signal. Choose a scoped stop or the engine emergency-stop control based on the observed incident.

---

## 🛠️ Execution SOP

### 1. Immediate Halt (The Kill Switch)
- **Engine Control**: Use the authenticated engine emergency-stop route first. It cancels registered agents, schedules, and workflows, persists `dispatch_paused`, and leaves the server available for explicit recovery.
- **Process Termination**: Use `SIGKILL` or `Stop-Process` only when the control plane is compromised or the process cannot complete the emergency stop.
- **Dashboard Control**: Close all active WebSocket connections to the Engine sidecar.

### 2. Resource Lockdown
- **Budget**: Review the affected agent and mission quotas. Do not rely on setting `DEFAULT_AGENT_BUDGET_USD=0` as a global stop; it controls the default quota for newly encountered agents, not existing quota rows or active calls.
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
