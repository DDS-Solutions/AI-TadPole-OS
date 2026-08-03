> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[incident_response]` in audit logs.
>
> ### AI Assist Note
> Incident Response Protocol
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# Incident Response Protocol

1. Immediate sidecar process kill (Taskkill /F).
2. Wipe 'tadpole.db-shm' and '-wal' locks.
3. Check port 8000 availability.
4. Restart Engine in 'SAFE-MODE'.
5. Broadcast emergency status to 'Sovereign'.
6. Run POST-MORTEM analysis.

[//]: # (Metadata: [incident_response])
