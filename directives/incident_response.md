# 🚨 Directive: Incident Response (SOP-OPS-02)

## 🎯 Primary Objective
Ensure rapid containment and recovery during system-wide failures, security breaches, or unauthorized resource consumption. Time-to-Containment (TTC) is the primary KPI for this SOP.

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