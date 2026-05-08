> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[documentation_audit]` in audit logs.
>
> ### AI Assist Note
> 📖 Directive: Documentation Audit (SOP-DEV-09)
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 📖 Directive: Documentation Audit (SOP-DEV-09)

## 🎯 Primary Objective
Deliver 100% parity between the written "Operations Manual" and the actual "Sovereign Reality" of the codebase. This audit ensures that no agent or human ever operates in an information vacuum.

---

## 🏗️ Audit Checklist

### 1. Tool-Usage & Density Parity (P0)
- **Check**: Compare `execution/` script arguments against the `OPERATIONS_MANUAL.md` tool references.
- **Density**: Run `python execution/comment_coverage.py` and verify coverage is >15% per module.
- **Action**: Flag any script or module with low density or missing documentation.

### 2. API Alignment
- **Check**: Verify all endpoints in `server-rs/src/routes/` are correctly documented in `API_REFERENCE.md`.
- **Constraint**: Check for correct `snake_case` usage in payload examples.

### 3. Architectural Integrity
- **Check**: Ensure `ARCHITECTURE.md` perfectly reflects the Current State of the Hub-Runner-Registry system.
- **Source of Truth**: The active source code is ALWAYS the source of truth if drift is detected.

---

## 🛠️ Audit SOP

### 1. The Drift Scanner
- **Execution**: Run `python execution/parity_guard.py .`.
- **Action**: Immediately correct any "Level 1" drift (typos/path changes).

### 2. Expert Review
- **Action**: Use a specialist agent to audit the "Instructional Clarity" of the SOPs. Can a new "Cluster CEO" follow the instructions without error?

---

## 📊 Reporting Protocol
Results must be archived in `reports/DOC_AUDIT_[TIMESTAMP].md`. Highlight "Broken SOPs" as CRITICAL findings that require immediate `documentation_audit.md` remediation.

[//]: # (Metadata: [documentation_audit])
