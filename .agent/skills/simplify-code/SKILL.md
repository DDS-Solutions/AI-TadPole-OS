---
name: simplify-code
description: "Reduce complexity of over-engineered code. Identify unnecessary abstractions, remove dead code, flatten deep nesting, and simplify logic while preserving behavior."
when_to_use: "When code is over-engineered, overly abstract, deeply nested, or more complex than needed. When asked to 'simplify', 'clean up', or 'reduce complexity'. NOT for adding new features."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / simplify-code
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Introducing behavioral regressions during refactoring, breaking existing invariant tests, or premature abstraction.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Simplify Code — Reduce Unnecessary Complexity

> **Philosophy**: Complexity is an ongoing tax. Every unused abstraction, redundant wrapper, and deeply nested block increases cognitive load and hides bugs. Simplify ruthlessly while proving 100% behavioral parity.  
> **Workflow Bindings**: Integrated into [`/refactor`](../../workflows/refactor.md) and [`/anneal`](../../workflows/anneal.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/complexity_metrics_and_refactoring.md`](./references/complexity_metrics_and_refactoring.md) | Cyclomatic complexity limits, nesting thresholds, and pass-through wrapper elimination | Refactoring complex functions or removing indirection |
| [`scripts/complexity_analyzer.py`](./scripts/complexity_analyzer.py) | Automated static analyzer detecting deep nesting ($\ge 4$ levels) and oversized functions | Auditing source files before and after refactoring |

---

## 🧭 1. The Simplification Rubric

### 1. Unnecessary Abstractions
- **Pass-through Wrappers**: If a class or wrapper function simply delegates to an inner module without transformation, remove it.
- **Single-Implementation Interfaces**: Merge interfaces with only one implementing class.
- **Premature Factories**: If a factory only instantiates one concrete class, use the constructor directly.

### 2. Flattening Deep Nesting
- **Guard Clauses**: Invert nested `if` statements into early returns (`if (!condition) return;`).
- **Linearize Async**: Flatten nested `.then()` callbacks into clean `async/await` sequences.

### 3. Dead Code Elimination
- Prune commented-out code blocks (git history preserves all changes).
- Remove unused variables, imports, and dead branches.

---

## 🧠 2. Socratic Gate for Simplification

Before committing a simplification refactor, verify:
- **Gate 1 (Zero Behavioral Change)**: Does this change preserve 100% of existing behavior and test coverage?
- **Gate 2 (Verification Proof)**: Did all unit and invariant tests pass before and after the change?
- **Gate 3 (Readability Gain)**: Does this modification objectively reduce cognitive load and nesting levels?

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Clever One-Liners** | Replacing a readable 4-line loop with an inscrutable regex or nested ternary. | Write straightforward, readable code over hyper-dense cleverness. |
| **Refactoring Without Tests** | Modifying logic without existing test coverage introduces regressions. | Write regression tests first before simplifying. |
| **Speculative Extensibility** | Adding interfaces and factory layers "just in case we need it later". | Implement strictly for today's concrete requirements (YAGNI). |

---

## 🛠️ 4. Deterministic Verification Gate

Measure indentation complexity and verify behavioral preservation:

```powershell
# 1. Run built-in self-tests
python .agent/skills/simplify-code/scripts/complexity_analyzer.py --test

# 2. Analyze complexity in a target source file
python .agent/skills/simplify-code/scripts/complexity_analyzer.py --file src/services/system.ts
```