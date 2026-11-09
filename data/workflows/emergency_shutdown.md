> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / emergency_shutdown
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[emergency_shutdown]`)

# emergency_shutdown
---
description: Immediate graceful shutdown of all non-critical systems.
---

1. Signal all nodes to pause.
2. Flush persistence buffers.
3. Terminate non-essential processes.
4. Enter low-power stabilization mode.