> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[incident_response]` in audit logs.
>
> ### AI Assist Note
> 🚨 Directive: Incident Response (SOP-OPS-02)
>
> ### 🔍 Debugging & Observability
> Traceability via `execution/parity_guard.py`.

# 🚨 Directive: Incident Response (SOP-OPS-02)

## 🎯 Primary Objective
Ensure rapid containment and recovery during system-wide failures, security breaches, or unauthorized resource consumption. Time-to-Containment (TTC) is the primary KPI for this SOP.

---

## 🛠️ Phase 0: AI-Indexable Asset Check *(IDENTITY.md Directive #6 — Mandatory First Step)*
> [!IMPORTANT]
> Before ANY log inspection or code analysis, check the AI-Indexable assets. This prevents token waste and accelerates RCA.
- **Step 1 — Error Registry**: Search [`docs/ERROR_REGISTRY.json`](file:///docs/ERROR_REGISTRY.json) for the observed error code or symptom. It maps error codes directly to source files and known failure paths.
- **Step 2 — Telemetry Map**: If the error is not in the registry, search [`docs/TELEMETRY_MAP.json`](file:///docs/TELEMETRY_MAP.json) by tag (e.g., `[VaultStore]`, `[AgentService]`) to locate the log emitter.
- **Only if both assets yield no result**: Proceed to Phase 1.

---

## 🛠️ Phase 1: Detection & Triage
- **Indicator**: High frequency of 4xx/5xx errors in `Axum` logs, or `BudgetGuard` reporting unauthorized spend.
- **Verification Tool**: `python execution/verify_all.py .`
- **Initial Action**:
  - Identify the offending agent ID from the engine logs.
  - Review the `audit.rs` Merkle log for the last 5 minutes.

## 🛠️ Phase 2: Containment
- **Kill Switch**: If a single agent is looping, use the `reset_agent` route immediately.
- **Global Suspend**: If the swarm is compromised, set `PRIVACY_MODE=true` in `.env` to cut external LLM access.
- **Resource Flush**: Purge all `workspaces/` temp data if data exfiltration is suspected.

## 🛠️ Phase 3: Root Cause Analysis (RCA)
- **Log Synthesis**: Extract logs related to the incident.
- **Agent 99 Integration**: Use `execution/debrief_mission.py` to synthesize lessons from the failure logs.
- **Parity Check**: Run `parity_guard.py` to ensure no architectural backdoors were created.

---

## 🚦 Mitigation Protocols
- **Credential Leak**: Immediately rotate all keys in the `Neural Vault`.
- **Sandbox Escape**: Shut down the Engine sidecar and audit `filesystem.rs` canonicalization logic.
- **Token Exhaustion**: Analyze the `working_memory` of the agent that caused the burn to identify "Chain of Thought" loops.

## 📝 Reporting Protocol
Documentation in `incidents/REPORT_[TIMESTAMP].md` is mandatory within 1 hour of containment. Focus on "Lessons Learned" and "Automated Prevention" steps.
[//]: # (Metadata: [incident_response])
