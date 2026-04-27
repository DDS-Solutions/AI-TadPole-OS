> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Infrastructure:Execution**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: testing-patterns
description: Testing principles. Unit, Integration, Mocking.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

# Testing Patterns

**Reliable suites.**

## Pyramid
- **E2E** (Top, Few): Critical flows. Slow.
- **Integration** (Mid, Some): API/DB. Medium.
- **Unit** (Base, Many): Functions/Classes. Fast.

## Unit Principles
- **Fast**: < 100ms.
- **Isolated**: No external deps.
- **Scope**: Test Logic, Error Handling. NOT Frameworks/Getters.

## Integration Principles
- **Target**: API endpoints, DB queries.
- **Lifecycle**: Connect -> Reset -> Test -> Clean -> Disconnect.

## Mocking
- **When**: External APIs, DB (unit), Time.
- **Don't**: Code under test, Simple deps.
- **Types**: Stub (Fixed), Spy (Track), Mock (Expect), Fake (Simple impl).

---

## 🧠 Aletheia Reasoning Protocol (QA)

### 1. Generator (Strategy)
*   **Unit**: "Complex logic?".
*   **Integration**: "Hits DB?".
*   **E2E**: "Checkout flow?".

### 2. Verifier (Audit)
*   **Flakiness**: "Fix or delete.".
*   **Speed**: "Too slow? Parallelize/Mock.".
*   **Reality**: "Mocking too much?".

### 3. Reviser (Maintenance)
*   **Dedupe**: "Merge tests.".
*   **Clean**: "Use `beforeEach`.".

---

## 🛡️ Security & Safety Protocol (Testing)

1.  **Data**: Never run against Prod DB.
2.  **Fuzzing**: Fuzz input parsers.
3.  **Auth**: Test permissions (A vs B).
4.  **Deps**: Audit test deps.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
