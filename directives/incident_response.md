> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Directives / incident_response
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[incident_response]`)

# 🚨 Directive: Incident Response (SOP-OPS-02)

## 🎯 Primary Objective
Ensure rapid containment and recovery during system-wide failures, security breaches, or unauthorized resource consumption. Time-to-Containment (TTC) is the primary KPI for this SOP.

---

## 🛠️ Phase 0: AI-Indexable Asset Check *(IDENTITY.md Directive #6 — Mandatory First Step)*
Before inspecting source files or raw logs, consult the AI-indexable assets in strict sequence:
1. **Error Registry**: Query [`docs/ERROR_REGISTRY.json`](../docs/ERROR_REGISTRY.json) for runtime error codes such as `BUDGET_EXHAUSTED` and `RECURSION_BLOCKED`. `STASIS`, `LOGIC_BLOCKER`, and `COMPLIANCE_DRIFT` are manual governance labels, not runtime errors.
2. **Security Registry**: Query [`docs/SECURITY_REGISTRY.json`](../docs/SECURITY_REGISTRY.json) by security policy (`SEC-01` through `SEC-08`) to verify cryptographic and sandbox guards.
3. **Telemetry Map**: Query [`docs/TELEMETRY_MAP.json`](../docs/TELEMETRY_MAP.json) by emitter tag to trace the exact Rust or TypeScript call site.
4. **Dual-Plane Breach Triage (`FR-11`)**: Classify whether the incident involves **Plane 1 (`Engine-Maintenance`)** or planned **Plane 2 (`SMB-Digital-Twin`)**. The current runner rejects only identities whose stored `department` exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive); other labels do not activate it. Check possible data exposure paths directly and do not assume company-scoped isolation is enforced.
---

## 🚦 Mitigation Protocols
- **Credential Leak**: Immediately rotate all keys in the `Neural Vault`.
- **Sandbox Escape**: Shut down the Engine sidecar and audit `filesystem.rs` canonicalization logic.
- **Token Exhaustion**: Analyze the `working_memory` of the agent that caused the burn to identify "Chain of Thought" loops.

## 📝 Reporting Protocol
Documentation in `incidents/REPORT_[TIMESTAMP].md` is mandatory within 1 hour of containment. Focus on "Lessons Learned" and "Automated Prevention" steps.
