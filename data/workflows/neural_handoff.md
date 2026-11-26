> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / neural_handoff
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[neural_handoff]`)

# neural_handoff
---
description: Handoff a mission to another node via neural link.
---

1. Select optimal recipient node.
2. Package mission state.
3. Initiate cross-node transfer.
4. Verify recipient acknowledgement.