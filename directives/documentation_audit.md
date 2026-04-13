# 📖 Directive: Documentation Audit (SOP-DEV-09)

## 🎯 Primary Objective
Deliver 100% parity between the written "Operations Manual" and the actual "Sovereign Reality" of the codebase. This audit ensures that no agent or human ever operates in an information vacuum.

---

## 🏗️ Audit Checklist

### 1. Tool-Usage Parity (P0)
- **Check**: Compare `execution/` script arguments against the `OPERATIONS_MANUAL.md` tool references.
- **Action**: Flag any script that has been refactored without a documentation update.

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
