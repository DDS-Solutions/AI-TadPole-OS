> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / compliance_check
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[compliance_check]`)

# Compliance & Governance Check

1. Scan mission logs for neural token leakage.
2. Verify all API calls use approved providers.
3. Audit budget consumption against limits.
4. Report 'Out of Bound' agent behavior.
5. Post pass/fail status to the Hub.