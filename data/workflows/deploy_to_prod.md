> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / deploy_to_prod
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[deploy_to_prod]`)

# deploy_to_prod
---
description: Deployment command for production releases.
---

1. Run pre-flight checks.
2. Build application.
3. Deploy to platform.
4. Verify deployment.