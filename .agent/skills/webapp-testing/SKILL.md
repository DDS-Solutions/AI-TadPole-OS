---
name: webapp-testing
description: "Web application testing principles. E2E, Playwright, deterministic clock mocking, route interception, and deep audit strategies."
when_to_use: "When writing E2E tests with Playwright, performing deep web app audits, or testing user flows. Use with /test workflow."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.2.0"
priority: critical
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / webapp-testing
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Flaky selector timeouts, browser harness crashes, or unhandled console exceptions.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Web Application Testing Protocol

> **Purpose**: Systematic discovery, accessibility audit, and deterministic end-to-end verification of all web routes, user interactions, and UI components using Playwright.
> **Skill Synergy & Boundaries**:
> - **Unit & Invariant Testing (Vitest)**: Delegates to [`testing-patterns`](../testing-patterns/SKILL.md) and [`tdd-workflow`](../tdd-workflow/SKILL.md).
> - **UI Design Verification**: Delegates visual fidelity to [`frontend-design`](../frontend-design/SKILL.md).
> - **Workflow Binding**: Used directly during the [`/test`](../../workflows/test.md) workflow.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** testing rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide / Tool | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/playwright_e2e_recipes.md`](./references/playwright_e2e_recipes.md) | Page Object Models, deterministic clock mocking, network route interception, accessibility checks | Writing Playwright test suites & UI flow automation |
| [`scripts/playwright_runner.py`](./scripts/playwright_runner.py) | Deterministic headless Playwright route runner & accessibility scanner | Automated E2E verification or `--test` validation |

---

## 🔍 1. Discovery-First Audit Strategy

```
1. MAP ROUTES ➔ Inspect React Router (`src/App.tsx`, `src/pages/`) and backend Axum endpoints.
2. HEALTH SCAN➔ Verify all primary routes mount without unhandled JavaScript exceptions.
3. FLOW TESTS ➔ Assert mission dispatch, vault unlocks, and task lifecycle events.
```

---

## 📐 2. Test Selection & Stability Guidelines

- **Stable Selectors**: Always use `data-testid` or accessible ARIA roles (`getByRole`, `getByLabel`) rather than fragile CSS classes.
- **Deterministic Auto-Waiting**: Rely on Playwright built-in web-first assertions (`await expect(locator).toBeVisible()`). Never use hardcoded sleeps (`page.waitForTimeout`).
- **Clock Mocking**: When testing debounced inputs, countdowns, or polling intervals, use `page.clock.install()` and `page.clock.fastForward()` rather than waiting in real-time.
- **Route Interception**: Test backend error handling deterministically by mocking HTTP/WebSocket endpoints with `page.route()`.
- **Console Error Trapping**: Fail test runs immediately upon unhandled browser exceptions (`page.on('pageerror')`).

---

## 🛡️ Socratic Gate: E2E Test Strategy & Blast Radius

Before generating or refactoring E2E test suites, confirm:
1. **Mock vs Real Backend**: Are tests running against mocked network routes (`page.route()`) or the live local test daemon?
2. **Selector Durability**: Do target elements contain deterministic `data-testid` or ARIA role attributes?
3. **Flake Tolerance**: Is network idling properly asserted using auto-waiting locators rather than arbitrary sleep timers?
4. **Console Hygiene**: Is `pageerror` monitored to capture silent JavaScript failures?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| Hardcoded `page.waitForTimeout(5000)` | Non-deterministic flakiness & CI slowdowns | Use locator auto-waiting (`toBeVisible()`) or `page.clock` |
| Fragile CSS/XPath (`div > nth-child(2)`) | Breaks on minor style refactoring | Standardize on semantic ARIA roles or `data-testid` |
| Ignoring uncaught console errors | Silent visual breakage passes in CI | Hook `page.on('pageerror')` and fail on errors |
| Shared mutable state across tests | Cross-test contamination | Generate unique fixture IDs and isolated contexts |

---

## 🛠️ Verification & Execution Commands

```powershell
# Run headless browser suite via Playwright
npx playwright test

# Execute runtime python browser audit
python .agent/skills/webapp-testing/scripts/playwright_runner.py http://localhost:5173 --a11y

# Run self-test suite
python .agent/skills/webapp-testing/scripts/playwright_runner.py --test
```