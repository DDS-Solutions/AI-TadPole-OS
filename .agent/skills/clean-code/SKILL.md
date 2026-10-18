---
name: clean-code
description: "Pragmatic coding standards: concise, direct, zero over-engineering, guard clauses, cognitive load minimization, and language idioms (Rust, TS, Python)."
when_to_use: "Always active for all code writing, refactoring, and code reviews. Enforces concise, direct implementation, testing pyramid, and cognitive load minimization."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.3.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / clean-code
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Over-engineering, God-objects, deep nesting, swallowed exceptions, unverified speculative code.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Clean Code — Pragmatic Engineering Standards

> **Philosophy**: Be concise, direct, and solution-focused. Code is written for humans to read and machines to execute. Minimize cognitive load at all times.

---

## 📚 Progressive Disclosure & L3 Reference Index

Consult deep **L3 Reference Guides** and verification tools on demand:
- **Refactoring & Git Standards**: [refactoring_and_git_standards.md](file:///D:/TadpoleOS-Dev/.agent/skills/clean-code/references/refactoring_and_git_standards.md)
- **Deterministic Clean Code Linter**: [clean_code_linter.py](file:///D:/TadpoleOS-Dev/.agent/skills/clean-code/scripts/clean_code_linter.py)

---

## 📐 1. Core Engineering Principles

| Principle | Meaning & Practical Implementation |
|---|---|
| **Single Responsibility (SRP)** | Each function or module does ONE thing well. Functions $\le 50$ lines, files $\le 500$ lines. |
| **Guard Clauses & Early Returns** | Invert `if` conditions to handle errors and boundary cases first. Eliminate `else` ladders and keep nesting depth $\le 3$. |
| **Cognitive Load Minimization** | The "3 AM Outage Test": If a sleepy on-call engineer cannot understand the logic in 30 seconds, refactor it. |
| **Zero Over-Engineering** | Do not build speculative interfaces, dynamic registries, or generic wrappers for single-use logic (YAGNI & KISS). |
| **Colocation & Domain Boundaries** | Place tests, types, and domain helpers physically close to where they are consumed. Avoid grab-bag `utils/` dumps. |
| **Self-Documenting Naming** | Reveal intent through precise identifiers (`is_active`, `get_session_by_token`). Avoid redundant inline comments. |

---

## ⚡ 2. Language-Specific Clean Code Idioms

### Rust Idioms
- **Error Propagation**: Use `?` operator with typed errors (`thiserror`, `anyhow`). Never call `.unwrap()` or `.expect()` in production routes or library modules.
- **Combinators over Match Nesting**: Prefer `.map()`, `.and_then()`, `.unwrap_or_default()` over deep pattern matching trees.
- **Borrowing Hygiene**: Pass borrowed slices (`&str`, `&[T]`) instead of owned references (`&String`, `&Vec<T>`). Avoid unnecessary `.clone()` calls.

### TypeScript / React Idioms
- **Strict Typing**: Zero `any`. Use `unknown` with type guards or discriminated unions for dynamic data.
- **Decoupled Architecture**: Decouple reactive Zustand/Context stores from presentational components. Component functions must remain pure.
- **Small Composable UI**: Break complex views into atomic components ($\le 150$ lines per UI component).

### Python Idioms
- **Explicit Typing**: Add type hints to all public function signatures and return types.
- **Resource Management**: Always use context managers (`with`) for files, network sessions, and database connections.
- **Immutable Defaults**: Never use mutable default arguments (`def func(items=[])` $\rightarrow$ `items=None`).

---

## 🚫 3. Critical Anti-Patterns & Violations

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **God Objects & Mega-Files** | Files $> 500$ lines or functions $> 50$ lines doing multiple unrelated tasks. | Decompose into focused domain modules. |
| **The Arrow Anti-Pattern** | Deeply nested conditionals $> 3$ levels deep. | Refactor using guard clauses and early exits. |
| **Swallowed Exceptions** | Empty `catch (e) {}` or `except: pass` silently dropping errors. | Wrap with domain context or return typed Result. |
| **Any-Type Abuse** | Bypassing TypeScript checks with `as any`. | Use unknown with explicit type narrowing. |
| **Magic Numbers & Strings** | Hardcoded literals (`86400`, `"admin"`) in algorithmic logic. | Define named symbolic constants at module headers. |
| **Zombie Code** | Dead, commented-out blocks left in the codebase. | Delete immediately; version control preserves history. |

---

## 🛠️ 4. Verification Gate

Audit source files for clean-code smells using the deterministic validator:

```powershell
# Run built-in self tests
python .agent/skills/clean-code/scripts/clean_code_linter.py --test

# Scan a source file for clean code smells
python .agent/skills/clean-code/scripts/clean_code_linter.py --file src/routes/system.rs

# Full repository integrity and parity audit
npm run adg:verify
python execution/parity_guard.py .
```

[//]: # (Metadata: [SKILL])