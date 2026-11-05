> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Directives / compliance_check
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[compliance_check]`)

# ⚖️ Directive: Compliance Check (SOP-FIN-04)

## 🎯 Primary Objective
Validate that the AI-Tadpole-OS cluster operates within all defined legal, ethical, and security boundaries. This directive is the final gate for "Sovereign Certification."

---

## 🛡️ Compliance Gates

### 1. Regulatory Alignment (GDPR/CCPA/Sovereign)
- **Check**: Audit `Privacy Mode` implementation in `privacy.rs`.
- **Constraint**: Verify data residency against the actual model endpoint, configured tools, integrations, and host/network policy. The runner's Privacy Mode does not enforce a system-wide egress boundary; report any unverified or external data path rather than claiming 100% residency or zero leakage.

### 2. Security Compliance (SOC2/ISO Alignment)
- **Check**: Run `vulnerability-scanner`.
- **Requirement**: Zero unmitigated HIGH or CRITICAL findings.
- **Action**: Verify `identity_anonymization` logic in engine logs.

### 3. Governance Continuity & Dual-Plane Firewall (`FR-11`)
- **Check**: Verify the `merkle-chain` in `tadpole.db` and audit capability boundaries between **Plane 1 (`Engine-Maintenance`)** and **Plane 2 (`SMB-Digital-Twin`)**.
- **Requirement**: No broken links in the audit history. Every tool call must be linked to an authorized mission and budget ID. Verify the actual runner guard: it rejects only identities whose stored `department` exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive); other labels do not activate it. Do not infer zero code-editing or shell privileges from a role name. Because company-scoped authorization and storage are pending, report `FR-11` and `NFR-1` as incomplete and do not use live company data through the generic runner. Check that no company PII or business records entered engine source control or `directives/LONG_TERM_MEMORY.md`.

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
