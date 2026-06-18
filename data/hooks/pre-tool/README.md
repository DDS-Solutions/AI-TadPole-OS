> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[README]` in audit logs.
>
> ### AI Assist Note
> Pre-Tool Lifecycle Hooks
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# Pre-Tool Lifecycle Hooks

Place executable scripts (`.ps1`, `.bat`, `.exe` on Windows; `.sh` or others on Linux) in this directory.

The engine will execute them **before** any agent tool call.
Context is provided via environment variables:
- `AGENT_CONTEXT`: JSON containing `agent_id`, `mission_id`, and `skill`.
- `TOOL_PARAMS`: JSON containing the arguments passed to the tool.

The script must exit with code 0 to allow the tool execution to proceed.

[//]: # (Metadata: [README])
