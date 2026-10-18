---
name: webapp-testing
description: Web application testing principles. E2E, Playwright, deep audit strategies.
when_to_use: "When writing E2E tests with Playwright, performing deep web app audits, or testing user flows. Use with /test workflow."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
version: "2.1.0"
priority: critical
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / webapp-testing
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Web Application Testing Protocol

> **Purpose**: Systematic discovery, accessibility audit, and end-to-end verification of all web routes, user interactions, and UI components.
> **Workflow Binding**: Used directly during the [`/test`](../../workflows/test.md) workflow.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** testing rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide / Tool | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/playwright_e2e_recipes.md`](./references/playwright_e2e_recipes.md) | Page Object Models, accessibility checks, screenshot diff fixtures | Writing Playwright test suites & UI flow automation |
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

- **Stable Selectors**: Always use `data-testid` or accessible ARIA roles (`getByRole`) rather than fragile CSS classes.
- **Auto-Waiting**: Rely on Playwright built-in web-first assertions (`expect(locator).toBeVisible()`).
- **State Isolation**: Ensure each test starts with a clean or authenticated session state.
- **Console Log Capture**: Capture and fail on uncaught browser `window.onerror` or unhandled promise rejections.

---

## 🛡️ Socratic Gate: E2E Test Strategy & Blast Radius

Before generating or refactoring E2E test suites, confirm:
1. **Mock vs Real Backend**: Are tests running against mock Axum services or live test databases?
2. **Selector Durability**: Do target elements contain deterministic `data-testid` attributes?
3. **Flake Tolerance**: Is network idling properly asserted using `waitForLoadState('networkidle')` or explicit API response interceptors?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| Hardcoded `page.waitForTimeout(5000)` | Non-deterministic flakiness & CI slowdowns | Use locator-driven auto-waiting (`toBeVisible()`) |
| Fragile CSS/XPath hierarchy (`div > nth-child(2)`) | Breaks on minor style refactoring | Standardize on semantic ARIA roles or `data-testid` |
| Ignoring uncaught console errors | Silent visual breakage | Hook `page.on('console', msg => ...)` and fail on errors |
| Shared mutable state across tests | Cross-test contamination | Generate unique fixture IDs per test session |

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