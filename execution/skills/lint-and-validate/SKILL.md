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
name: lint-and-validate
description: Automatic quality control. Run after EVERY code change.
allowed-tools: Read, Glob, Grep, Bash
---

# Lint and Validate

**MANDATORY: Error-free code before task completion.**

## Procedures
- **Node/TS**: `npm run lint` + `npx tsc --noEmit`.
- **Python**: `ruff check . --fix` + `mypy .`.
- **Security**: `npm audit` / `bandit`.

## The Quality Loop
1.  **Edit**.
2.  **Audit** (Lint + Type Check).
3.  **Fix** (Must be clean).
4.  **Submit**.

## Error Handling
- **Lint Fail**: Fix grammar/style.
- **Type Fail**: Fix logic/schema.
- **No Config**: Suggest creating `.eslintrc`/`tsconfig.json`.

**Rule:** No "done" with red linter output.

## Scripts
- `scripts/lint_runner.py`: Unified runner.
- `scripts/type_coverage.py`: Coverage check.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
