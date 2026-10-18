---
name: tdd-workflow
description: "Test-Driven Development workflow principles. RED-GREEN-REFACTOR cycle, writing failing tests first, and enforcing behavioral verification."
when_to_use: "When practicing Test-Driven Development, following the RED-GREEN-REFACTOR cycle, or implementing features where behavioral tests must precede code."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / tdd-workflow
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Writing implementation code before writing failing tests, untested regression paths, or unverified green transitions.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Test-Driven Development (TDD) Protocol

> **Philosophy**: Write tests first, code second. Prove failure before implementing solutions. The test is the deterministic contract of truth.  
> **Workflow Bindings**: Integrated directly into [`/test`](file:///D:/TadpoleOS-Dev/.agent/workflows/test.md), [`/create`](file:///D:/TadpoleOS-Dev/.agent/workflows/create.md), and [`/debug`](file:///D:/TadpoleOS-Dev/.agent/workflows/debug.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/tdd_patterns_and_cycles.md`](file:///D:/TadpoleOS-Dev/.agent/skills/tdd-workflow/references/tdd_patterns_and_cycles.md) | Concrete code cycles, Three Laws of TDD, test smell refactoring | Writing unit tests before implementation |
| [`scripts/tdd_cycle_guard.py`](file:///D:/TadpoleOS-Dev/.agent/skills/tdd-workflow/scripts/tdd_cycle_guard.py) | Automated TDD state-machine tracking Red-Green-Refactor phase transitions | Enforcing and recording TDD cycles |

---

## 🔄 1. The RED-GREEN-REFACTOR Cycle

```
[1. 🔴 RED] ─────────────> [2. 🟢 GREEN] ─────────────> [3. 🔵 REFACTOR]
 Write a minimal failing     Write the minimum code        Clean up code, remove
 test asserting behavior     required to pass the test     duplication, stay green
```

1. **🔴 RED**: Write a minimal, expressive test asserting the desired behavior. Run it and verify that it fails (exit code $\ne 0$). Record via `tdd_cycle_guard.py --record-phase RED`.
2. **🟢 GREEN**: Write the simplest implementation code that satisfies the test. Run the test and verify it passes (exit code $0$). Record via `tdd_cycle_guard.py --record-phase GREEN`.
3. **🔵 REFACTOR**: Clean up syntax, remove duplication, and optimize algorithms while ensuring tests remain green.

---

## 📐 2. The Three Sovereign Laws of TDD

1. **Law 1**: You are not allowed to write any production code unless it is to make a failing unit test pass.
2. **Law 2**: You are not allowed to write any more of a unit test than is sufficient to fail; and not compiling or failing an assertion is failing.
3. **Law 3**: You are not allowed to write any more production code than is sufficient to pass the one failing unit test.

---

## 🧠 3. Socratic Gate for TDD Execution

Before writing production logic, verify:
- **Gate 1 (Red Witness)**: Has the test been executed and proven to fail with a clear, specific failure message?
- **Gate 2 (Minimalism)**: Is the implementation code strictly confined to what the test demands without premature features?
- **Gate 3 (Test Clarity)**: Does the test name explicitly describe the behavioral expectation (`it('returns 404 when user id is absent')`)?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Writing Tests After Code** | Tests are biased to match the implementation rather than the specification. | Write the test assertion first; prove it fails. |
| **Passing Test in Red Phase** | A "red" test that passes immediately tests nothing. | Ensure the test asserts new, un-implemented behavior. |
| **Over-Engineering in Green** | Writing 500 lines of architecture when 10 lines satisfy the test. | Implement the simplest passing solution; refactor later. |
| **Skipping the Refactor Phase** | Codebase accumulates technical debt and duplicate logic. | Dedicate a clean-up turn immediately after reaching green. |

---

## 🛠️ 5. Deterministic Verification Gate

Track and enforce TDD cycles:

```powershell
# 1. Run built-in self-tests
python .agent/skills/tdd-workflow/scripts/tdd_cycle_guard.py --test

# 2. Record RED phase transition (test fails)
python .agent/skills/tdd-workflow/scripts/tdd_cycle_guard.py --record-phase RED --cmd "npm test" --code 1

# 3. Record GREEN phase transition (test passes)
python .agent/skills/tdd-workflow/scripts/tdd_cycle_guard.py --record-phase GREEN --cmd "npm test" --code 0
```