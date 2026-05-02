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
name: code-review-checklist
description: Code review guidelines covering code quality, security, and best practices.
allowed-tools: Read, Glob, Grep

command: ""
---

# Code Review Checklist

## Quick Checks
- **Correctness**: Works? Edges covered? Errors handled?
- **Security**: Sanitized? No secrets? Auth checks?
- **Perf**: No N+1? Caching? Bundle size?
- **Quality**: Naming? DRY? Tests?

## AI & LLM Review Patterns (2025)
- [ ] **Logic**: Chain of thought verifiable?
- [ ] **Hallucinations**: Invented APIs or files?
- [ ] **Prompt Safety**: Inputs sanitized before LLM use?

## Anti-Patterns to Flag
- ❌ **Magic Numbers** -> Use Constants.
- ❌ **Deep Nesting** -> Use Guard Clauses.
- ❌ **Any Type** -> Define Interfaces.
- ❌ **Long Functions** -> Break it up.

## Comments Guide
- 🔴 **BLOCKING**: Critical bugs, Security.
- 🟡 **SUGGESTION**: Improvements.
- 🟢 **NIT**: Style/Rename.
- ❓ **QUESTION**: Clarification.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
