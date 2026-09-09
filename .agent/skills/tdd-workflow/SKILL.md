---
name: tdd-workflow
description: Test-Driven Development workflow principles. RED-GREEN-REFACTOR cycle.
when_to_use: "When practicing Test-Driven Development, following RED-GREEN-REFACTOR cycle, or writing tests before implementation."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / tdd-workflow
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Test-Driven Development (TDD) Protocol

> **Philosophy**: Write tests first, code second. Prove failure before implementing solutions.
> **Workflow Binding**: Used directly during [`/test`](../../workflows/test.md), [`/create`](../../workflows/create.md), and [`/debug`](../../workflows/debug.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** TDD rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/tdd_patterns_and_cycles.md`](./references/tdd_patterns_and_cycles.md) | Concrete code cycles, Three Laws of TDD, test smell refactoring | Writing unit tests before implementation |

---

## 🔄 1. The RED-GREEN-REFACTOR Cycle

```
1. 🔴 RED      ➔ Write a minimal failing test that defines the expected behavior.
2. 🟢 GREEN    ➔ Write the minimum production code required to make the test pass.
3. 🔵 REFACTOR ➔ Clean up code, remove duplication, and optimize while staying green.
```

---

## 📐 2. Core TDD Directives

1. **Test Names Describe Intent**: Use clear behavioral names (`it('rejects expired capability tokens')`).
2. **YAGNI (You Aren't Gonna Need It)**: Do not write speculative code beyond what the failing test demands.
3. **One Logical Assertion**: Keep unit tests focused on a single logical invariant.

---

## 🛠️ 3. Execution Commands

```powershell
# 1. Run watch mode for fast feedback
npm run test -- --watch

# 2. Run backend test suite
cargo test --manifest-path server-rs/Cargo.toml
```