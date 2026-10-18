---
name: testing-patterns
description: "Testing patterns and principles. Unit, integration, mocking strategies, Arrange-Act-Assert (AAA), and multi-ecosystem test runners."
when_to_use: "When writing unit tests, integration tests, choosing testing frameworks (Vitest/Cargo/Pytest), or implementing mocking strategies."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / testing-patterns
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Flaky non-deterministic tests, coupled mock state, or assertions without behavioral verification.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Testing Patterns & Verification Principles

> **Standard**: Arrange-Act-Assert (AAA). Fast, deterministic, isolated unit tests.  
> **Workflow Bindings**: Integrated directly into [`/test`](file:///D:/TadpoleOS-Dev/.agent/workflows/test.md) and [`/adversary`](file:///D:/TadpoleOS-Dev/.agent/workflows/adversary.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign testing principles below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/mocking_and_fixture_patterns.md`](file:///D:/TadpoleOS-Dev/.agent/skills/testing-patterns/references/mocking_and_fixture_patterns.md) | Test double taxonomy (Mocks vs Spies vs Fakes), async Rust & Vitest fixtures | Writing unit and integration tests |
| [`scripts/test_runner.py`](file:///D:/TadpoleOS-Dev/.agent/skills/testing-patterns/scripts/test_runner.py) | Automated multi-ecosystem test executor (Vitest, Cargo, Pytest) with built-in self-tests | Running test suites across workspace boundaries |

---

## 📐 1. The AAA Pattern (Arrange-Act-Assert)

```typescript
it('calculates USD token burn correctly', () => {
  // 1. Arrange: Prepare mock fixtures and state
  const tokenTracker = new TokenTracker({ ratePerThousand: 0.002 });

  // 2. Act: Execute function under test
  const cost = tokenTracker.calculateCost(5000);

  // 3. Assert: Verify outcome matches specification
  expect(cost).toBe(0.01);
});
```

---

## ⚡ 2. Unit vs Integration Boundaries

| Test Layer | Target & Scope | Execution Speed Target |
|---|---|---|
| **Unit** | Pure functions, domain calculations, StateFlow reducers | Blazing ($<50$ms per test) |
| **Integration** | Axum router handlers, SQLite queries, WebSocket broadcasts | Fast (100–500ms) |
| **E2E / Swarm** | Multi-agent swarms & end-to-end browser flows | Bounded (1–5s) |

---

## 🧠 3. Socratic Gate for Test Design

Before writing or approving test suites, verify:
- **Gate 1 (Isolation)**: Does the test depend on shared mutable state, network access, or order of execution?
- **Gate 2 (Behavioral Assertions)**: Does the assertion test actual outcomes and edge cases rather than trivial tautologies (`expect(true).toBe(true)`)?
- **Gate 3 (Execution Speed)**: Does the unit test suite run in $<5$ seconds locally?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Over-Mocking** | Mocking everything so that the test passes even when real contracts break. | Test against real domain models; mock only external network boundaries. |
| **Flaky Timing** | Using arbitrary `setTimeout(100)` sleeps instead of explicit event promises. | Await deterministic event conditions or completion signals. |
| **Multiple Unrelated Asserts** | One test asserting 10 different unrelated behaviors obscures failure causes. | Split into discrete, focused test cases with clear intent. |

---

## 🛠️ 5. Deterministic Verification Gate

Execute tests across Node.js, Rust, or Python:

```powershell
# 1. Run built-in self-tests
python .agent/skills/testing-patterns/scripts/test_runner.py --test

# 2. Run automated test detection and suite execution
python .agent/skills/testing-patterns/scripts/test_runner.py .

# 3. Run with coverage
python .agent/skills/testing-patterns/scripts/test_runner.py . --coverage
```