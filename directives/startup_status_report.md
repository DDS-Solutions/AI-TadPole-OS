> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[startup_status_report]` in audit logs.
>
> ### AI Assist Note
> Directive: Startup Status Report Generation
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# Directive: Startup Status Report Generation

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the Tadpole OS 3-Layer Architecture.
> - **@docs ARCHITECTURE:Infrastructure**
> - **Layer 1**: Directive (SOP for post-reboot system status report)
> - **Layer 2**: Orchestration (Reboot trigger via `monitor.ps1` / `awaken.py` / `npm run startup:report`)
> - **Layer 3**: Execution (`execution/generate_startup_report.py`)

## Goal
Generate a comprehensive, highly detailed post-reboot status report written to `startup_status_report.md` at the workspace root, as well as archived in `reports/startup_status/`.

## Post-Reboot Verification SOP

1. **System & Environment Assessment**:
   - Host platform, OS version, Python version, Node.js version, Rust compiler toolchain.
   - Uptime, memory footprint, CPU load, process table integrity.

2. **Core Engine & Service Health**:
   - Query engine endpoint `http://localhost:8000/v1/engine/health`.
   - Verify Rust `server-rs` sidecar status, Tokio async runtime, and axum webserver.
   - Check status of background monitors and listeners.

3. **Database & Storage Parity**:
   - Inspect SQLite state (`tadpole.db`), WAL journal size, table integrity, and migration version.
   - Confirm disk space availability and temp directory clean state.

4. **Symbol Graph & Codebase Intelligence**:
   - Verify graph index file (`reports/intelligence/symbol_graph.json` or `audit_context.json`).
   - Check blast radius readiness and AST parse status.

5. **AI Swarm Mesh & Agent Memory**:
   - Inspect active agent pool (`/v1/agents`), pending oversight tasks (`/v1/oversight/pending`).
   - Confirm `LONG_TERM_MEMORY.md` and session memory status.

6. **Security & Parity Gate**:
   - Validate security policies (`/v1/oversight/security/policies`).
   - Verify AI context alignment headers (`verify_ai_context.py` / `parity_guard.py`).

7. **Report Formatting**:
   - Format results into clean Markdown with alerts, metric tables, status badges, and actionable next steps.
   - Save primary copy to workspace root `startup_status_report.md`.
   - Save timestamped archive copy to `reports/startup_status/startup_status_report_<timestamp>.md`.

## Execution Tool
- `python execution/generate_startup_report.py`

[//]: # (Metadata: [startup_status_report])
