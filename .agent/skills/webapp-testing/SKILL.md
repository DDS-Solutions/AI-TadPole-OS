> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: webapp-testing
description: Web application testing. E2E, Playwright, deep audit.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

# Web App Testing

**Discover. Map. Test.**

## Strategy
1.  **Discovery**: Scan routes, APIs, components.
2.  **Systematic**: Map -> Scan -> Test.

## Pyramid
- **E2E**: Critical flows (Login, Checkout).
- **Integration**: API/Data.
- **Component**: UI Units.

## E2E Principles
- **Priority**: Happy Path > Auth > Critical > Errors.
- **Practices**: `data-testid`, Auto-wait, Clean state.

## Playwright
- **Page Object Model**: Encapsulate logic.
- **Fixtures**: Reusable setup.
- **Config**: Retries on CI, Trace on first retry.

## Visual Testing
- Snapshot baseline -> Compare on change -> Review diffs.

---

## 🧠 Aletheia Reasoning Protocol (End-to-End)

### 1. Generator (Flows)
*   **Happy**: "Login -> Buy".
*   **Sad**: "Login -> Decline".
*   **Chaos**: "Back button?".

### 2. Verifier (Robustness)
*   **Wait**: "Spinner gone?".
*   **Selector**: "`data-testid`?".
*   **Env**: "Clean data?".

### 3. Reviser (Maintenance)
*   **POM**: "Encapsulate.".
*   **Debug**: "Video on failure?".

---

## 🛡️ Security & Safety Protocol (E2E)

1.  **Credentials**: Test accounts only.
2.  **Cleanup**: Delete created data.
3.  **Scope**: No load tests on Prod.
4.  **Secrets**: Env vars.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
