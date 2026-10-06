---
name: clean-code
description: Pragmatic coding standards - concise, direct, no over-engineering, no unnecessary comments.
when_to_use: "Always active for ALL code writing. Enforces concise, direct coding standards, the testing pyramid, and performance best practices."
allowed-tools: Read, Write, Edit
version: 2.2
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / clean-code
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Clean Code — Pragmatic Engineering Standards

> **Philosophy**: Be concise, direct, and solution-focused. Code is written for humans to read and machines to execute.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core coding standards below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/refactoring_and_git_standards.md`](./references/refactoring_and_git_standards.md) | Testing pyramid (70/20/10), Git commit standards, refactoring checklists | Pull requests & unit test coverage planning |

---

## 📐 1. Core Engineering Principles

| Principle | Meaning & Practical Rule |
|---|---|
| **Single Responsibility (SRP)** | Each function or struct does ONE thing well. |
| **Guard Clauses** | Return early for edge cases; avoid deep `if / else` nesting. |
| **Colocation** | Keep related logic, types, and unit tests physically close to their usage. |
| **Self-Documenting Code** | Reveal intent through precise names (`is_active`, `get_user_by_id`). Avoid redundant inline comments. |

---

## 🚫 2. Critical Anti-Patterns to Avoid

- **No God Objects**: Never let a single file or function grow beyond 500 lines. Decompose into focused modules.
- **No `utils.ts` Dumps**: Put functions in their respective domain namespaces instead of generic grab-bag files.
- **No Magic Numbers / Strings**: Define named constants (`MAX_RETRIES`, `DEFAULT_TIMEOUT_SECS`) at file headers.
- **No Swallowed Errors**: Always handle or re-wrap errors with contextual domain traces.

---

## 🛠️ 3. Verification Gate

```powershell
# Verify syntax, formatting, and doc parity
python execution/parity_guard.py .
```