> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[fiscal_audit]` in audit logs.
>
> ### AI Assist Note
> Financial Integrity Workflow
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# Financial Integrity Workflow

1.  **Audit Trigger**: Finance Director initiates an end-of-month or per-mission audit.
2.  **Usage Scanning**: Expense Auditor pulls `usage_usd` and real-time budget data.
3.  **Anomaly Detection**: AuditBot flags any mission exceeding its 10-second debounced sync budget.
4.  **Policy Review**: Compliance Agent checks Working Memory logs for data leak risks.
5.  **Reconciliation**: Finance Director synthesizes a "Verified Fiscal State" report.

[//]: # (Metadata: [fiscal_audit])
