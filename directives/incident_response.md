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
---

## 🚦 Mitigation Protocols
- **Credential Leak**: Immediately rotate all keys in the `Neural Vault`.
- **Sandbox Escape**: Shut down the Engine sidecar and audit `filesystem.rs` canonicalization logic.
- **Token Exhaustion**: Analyze the `working_memory` of the agent that caused the burn to identify "Chain of Thought" loops.

## 📝 Reporting Protocol
Documentation in `incidents/REPORT_[TIMESTAMP].md` is mandatory within 1 hour of containment. Focus on "Lessons Learned" and "Automated Prevention" steps.