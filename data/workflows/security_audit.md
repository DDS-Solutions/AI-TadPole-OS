> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / security_audit
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[security_audit]`)

# Security Audit Protocol

1. Perform static analysis of new scripts.
2. Scan for hardcoded credentials (.env leakage).
3. Validate sanitization of all tool inputs.
4. Check for unauthorized outbound network attempts.
5. Quarantine rogue agent processes.