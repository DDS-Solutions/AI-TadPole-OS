> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Directives / compliance_check
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[compliance_check]`)

# ⚖️ Directive: Compliance Check (SOP-FIN-04)

## 🎯 Primary Objective
Validate that the Tadpole OS cluster operates within all defined legal, ethical, and security boundaries. This directive is the final gate for "Sovereign Certification."

---

## 🛡️ Compliance Gates

### 1. Regulatory Alignment (GDPR/CCPA/Sovereign)
- **Check**: Audit `Privacy Mode` implementation in `privacy.rs`.
- **Constraint**: Ensure 100% data residency and zero unauthorized PII leakage to external providers.

### 2. Security Compliance (SOC2/ISO Alignment)
- **Check**: Run `vulnerability-scanner`.
- **Requirement**: Zero unmitigated HIGH or CRITICAL findings.
- **Action**: Verify `identity_anonymization` logic in engine logs.

### 3. Governance Continuity
- **Check**: Verify the `merkle-chain` in `tadpole.db`.
- **Requirement**: No broken links in the audit history. Every tool call must be linked to an authorized mission and budget ID.

---

## 🛠️ Audit SOP

### 1. Automated Scan
- **Execution**: Run `python execution/verify_all.py .` (Compliance category).

### 2. Manual Sample
- **Action**: Randomly audit 5 mission logs for budget adherence and "Human-in-the-Loop" compliance.

---

## 📊 Reporting Protocol
Generate a `COMPLIANCE_STATUS_[DATE].md` report. Any "Compliance Fault" must trigger an immediate Level 2 `incident_response.md`.

## 🚦 Approval
"Compliant" status is only granted after a successful Master Audit run and human sign-off of the audit report.

> [!NOTE]
> **Compliance Heartbeat** *(IDENTITY.md Directive #8)*: This manual SOP supplements — but does not replace — the automated Compliance Heartbeat. Every 10th swarm operation must include a self-audit checksum verifying directive alignment and Merkle Trail availability. A `[COMPLIANCE:DRIFT]` result from the automated heartbeat triggers this SOP immediately, regardless of schedule.