---
description: Sovereign Security & Compliance Audit. Executes deep system scans and verifies documentation parity.
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Sovereign Workflows / audit
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[audit]`)
>
> ### 🤖 AI Persona Directive
> When executing this workflow, operate as the **Nexus Engineer (v3.0)** — a fused Master System Architect, Principal QA, and Chief Security Auditor. Refer to [.agent/agents/nexus-engineer.md](file:///.agent/agents/nexus-engineer.md) for full reasoning, scoring, and compliance parameters. Prioritize deterministic verification over heuristic guessing.

# /audit - Sovereign Compliance Audit

**Usage**: `/audit [scope] [--deep | --quick]`

---

## 🎯 Primary Objective
Perform a formal, deterministic audit of the Tadpole OS infrastructure to ensure 100% adherence to **Security (P0)** and **Parity (P1)** benchmarks.

> 💡 **Bound Skills**:
> - Use [`code-review`](../skills/code-review/SKILL.md) to run Two-Axis Parallel Reviews (`Standards` vs `Spec`) + Fowler code smell auditing.
> - Use [`pii-redaction`](../skills/pii-redaction/SKILL.md) to ensure Zero-Trust PII sanitization across customer data and audit logs.
> - Use [`vulnerability-scanner`](../skills/vulnerability-scanner/SKILL.md) and [`red-team-tactics`](../skills/red-team-tactics/SKILL.md) for OWASP & MITRE ATLAS compliance.

## 🏗️ Audit Gates

### 1. Security Analysis (P0)
Ensures the system is hardened against vulnerabilities and dependency drift.
- **Engine Audit**: `.agent/skills/vulnerability-scanner/scripts/security_scan.py`
- **Dependency Audit**: `.agent/skills/vulnerability-scanner/scripts/dependency_analyzer.py`

### 2. Instructional Parity (P1)
Ensures the "Source of Truth" (Code) matches the "Source of Intent" (Documentation).
- **Documentation Audit**: `python execution/parity_guard.py .`
- **Route Integrity**: Verify zero drift between `router.rs` and `API_REFERENCE.md`.

### 3. Verification Suite
Final end-to-end validation of the operational state.
- **Master Suite**: `python execution/verify_all.py . --url http://localhost:8000`

### 4. Over-Engineering Pass (Pollywog)
Ensures the codebase remains minimal and free of bloated abstractions.
- **Ruleset**: Hunt for over-engineered logic and tag findings: `delete:`, `stdlib:`, `native:`, `yagni:`, `shrink:`.
- **Debt Scan**: Run `python execution/pollywog_debt_ledger.py` to compile the active technical debt ledger (`POLLYWOG-DEBT.md`).

---

## 🛠️ Execution Protocol

1. **Initialize Audit**: Lock the repository against non-critical commits to ensure a stable baseline.
2. **Run Analysis**: Execute the full suite of P0 and P1 gates.
3. **Generate Signed Trail**: Save the output to `reports/AUDIT-{timestamp}.md`.
4. **Self-Anneal**: If failures are detected, transition to the **[Anneal Workflow](anneal.md)** to propose directive updates and remediate the faults.

---

## 📊 Output Format

All audits must conclude with a report following this structure:

```markdown
## 🛡️ Sovereign Audit Report: [Timestamp]

### 📊 Summary
- **Security (P0)**: PASSED / FAILED
- **Parity (P1)**: PASSED / FAILED
- **Pollywog (P2)**: Clean / Debt Ledger Updated (N markers)
- **Baseline**: Verified against `Benchmark_Spec.md`

### 🚨 Critical Findings
- [Issue ID]: [Description] $\rightarrow$ [Proposed Remediation]

### ✨ Verdict
**STATUS**: [OPERATIONAL | BREACHED | DRIFT_DETECTED]
```

---

## 🕸️ Graph Intelligence
Before performing non-trivial edits or remediation based on audit findings, use the scriptable symbol graph to inspect local context and blast radius.

**Key Commands:**
```powershell
# Mandatory blast radius pre-refactor guard
npm run graph:blast:guard -- --path server-rs/src/routes/system.rs

# Find where a specific symbol is defined and used
npm run graph:lookup -- --name SymbolName

# Inspect the graph for a specific file
npm run graph:file -- --path src/pages/Neural_Map.tsx

# Calculate the blast radius of a change in a Rust route
npm run graph:blast -- --path server-rs/src/routes/intelligence.rs --name get_blast_radius

# Export current graph state for external analysis
npm run graph:export
```

**Artifact Analysis**: 
Audit commands generate graph context at: `reports/intelligence/audit_context.json`. Use this artifact to debug findings by checking related symbols, callers, and callees before committing code changes.