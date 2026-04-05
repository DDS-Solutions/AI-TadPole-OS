# Financial Integrity Workflow

1.  **Audit Trigger**: Finance Director initiates an end-of-month or per-mission audit.
2.  **Usage Scanning**: Expense Auditor pulls `usage_usd` and real-time budget data.
3.  **Anomaly Detection**: AuditBot flags any mission exceeding its 10-second debounced sync budget.
4.  **Policy Review**: Compliance Agent checks Working Memory logs for data leak risks.
5.  **Reconciliation**: Finance Director synthesizes a "Verified Fiscal State" report.
