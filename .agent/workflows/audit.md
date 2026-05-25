---
description: 
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[audit]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
description: Sovereign Security & Compliance Audit. Executes deep system scans and verifies documentation parity.
---

# /audit - Sovereign Compliance Audit

$ARGUMENTS

---

## 🎯 Primary Objective
Perform a formal, deterministic audit of the Tadpole OS infrastructure to ensure 100% adherence to Security (P0) and Parity (P1) benchmarks.

## 🏗️ Audit Gates

### 1. Security Analysis (P0)
- **Engine Audit**: `.agent/skills/vulnerability-scanner/scripts/security_scan.py`
- **Dependency Audit**: `.agent/skills/vulnerability-scanner/scripts/dependency_analyzer.py`

### 2. Instructional Parity (P1)
- **Documentation Audit**: `python execution/parity_guard.py .`
- **Route Integrity**: Verify zero drift between `router.rs` and `API_REFERENCE.md`.

### 3. Verification Suite
- **Master Suite**: `python execution/verify_all.py . --url http://localhost:8000`

---

## 🛠️ Execution Protocol

1. **Initialize Audit**: Lock the repository against non-critical commits.
2. **Run Analysis**: Execute the full suite of P0 and P1 gates.
3. **Generate Signed Trail**: Save the output to `reports/AUDIT-{timestamp}.md`.
4. **Self-Anneal**: If failures are found, trigger the `/debug` workflow for immediate remediation.

---

## Output Format

```markdown
## 🛡️ Sovereign Audit Report: [Timestamp]

### 📊 Summary
- **Security (P0)**: PASSED / FAILED
- **Parity (P1)**: PASSED / FAILED
- **Baseline**: Verified against `Benchmark_Spec.md`

### 🚨 Critical Findings
- [Issue 1]: [Remediation]

### ✨ Verdict
**STATUS**: [OPERATIONAL | BREACHED | DRIFT_DETECTED]
```
Perform as a expert code developer in Rust, React, Tailwind CSS, and TypeScript.

## Graph Intelligence

Before non-trivial edits, reviews, or audits that touch Rust routes, parser logic,
React pages, services, stores, or shared contracts, use the scriptable symbol graph
to inspect local context and blast radius.

Useful commands:

```powershell
npm run graph:lookup -- --name SymbolName
npm run graph:file -- --path src/pages/Neural_Map.tsx
npm run graph:blast -- --path server-rs/src/routes/intelligence.rs --name get_blast_radius
npm run graph:export
```

Audit commands generate graph context at:

```text
reports/intelligence/audit_context.json
```

Use that artifact to debug audit findings by checking related symbols, callers,
callees, and likely blast radius before changing code.
[//]: # (Metadata: [audit])
