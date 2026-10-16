---
name: testing-patterns
description: Testing patterns and principles. Unit, integration, mocking strategies.
when_to_use: "When writing unit tests, integration tests, choosing testing frameworks, or implementing mocking strategies."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / testing-patterns
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Testing Patterns & Verification Principles

> **Standard**: Arrange-Act-Assert (AAA). Fast, deterministic, isolated unit tests.
> **Workflow Binding**: Used directly during the [`/test`](../../workflows/test.md) workflow.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** testing principles below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/mocking_and_fixture_patterns.md`](./references/mocking_and_fixture_patterns.md) | Test double taxonomy (Mocks vs Spies vs Fakes), async Rust & Vitest fixtures | Writing unit and integration tests |

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

| Test Layer | Target & Scope | Execution Speed |
|---|---|---|
| **Unit** | Pure functions, domain calculations, StateFlow reducers | Blazing (<50ms per test) |
| **Integration** | Axum router handlers, SQLite queries, WebSocket broadcasts | Medium (100–500ms) |
| **E2E** | Multi-agent swarms & end-to-end browser flows | Heavy (1–5s) |

---

## 🛠️ 3. Execution Commands

```powershell
# 1. Run frontend unit tests
npm run test

# 2. Run backend Rust unit & integration tests
cargo test --manifest-path server-rs/Cargo.toml
```