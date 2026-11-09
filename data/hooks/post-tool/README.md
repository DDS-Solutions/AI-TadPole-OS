> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / README
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[README]`)

# Post-Tool Lifecycle Hooks

Place executable scripts (`.ps1`, `.bat`, `.exe` on Windows; `.sh` or others on Linux) in this directory.

The engine will execute them **after** any agent tool call.
Context is provided via environment variables:
- `AGENT_CONTEXT`: JSON containing `agent_id`, `mission_id`, and `skill`.
- `TOOL_PARAMS`: JSON containing the arguments passed to the tool.