> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / incident_response
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[incident_response]`)

# Incident Response Protocol

1. Immediate sidecar process kill (Taskkill /F).
2. Wipe 'tadpole.db-shm' and '-wal' locks.
3. Check port 8000 availability.
4. Restart Engine in 'SAFE-MODE'.
5. Broadcast emergency status to 'Sovereign'.
6. Run POST-MORTEM analysis.