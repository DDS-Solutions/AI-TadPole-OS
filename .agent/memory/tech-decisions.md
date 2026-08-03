> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[tech_decisions]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
type: project
created: 2026-07-26
updated: 2026-07-26
---

# Technical Decisions & Compatibility Notes

## TypeScript Version Compatibility

### TypeScript 7.0 Upgrade Status: BLOCKED
- **Tested Date**: 2026-07-26
- **Tested Version**: `typescript@7.0.2`
- **Result**:
  - `npx tsc --noEmit`: 0 errors (Pass)
  - `vitest` / Vite: Pass
  - `npx eslint .`: **Failed** with compatibility error:
    ```text
    Error: typescript-eslint does not support TS 7.0.
    Please see https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/#running-side-by-side-with-typescript-6.0 to run typescript-eslint using the TS 6 API.
    See also https://github.com/typescript-eslint/typescript-eslint/issues/10940 for tracking typescript-eslint's support for TS >=7.1
    ```
- **Decision**: Keep project on **TypeScript `5.9.3`** until `@typescript-eslint` releases AST parser support for TS >=7.1 (tracked in typescript-eslint issue #10940).

[//]: # (Metadata: [tech_decisions])
