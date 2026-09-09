---
name: webapp-testing
description: Web application testing principles. E2E, Playwright, deep audit strategies.
when_to_use: "When writing E2E tests with Playwright, performing deep web app audits, or testing user flows. Use with /test workflow."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / webapp-testing
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Web Application Testing Protocol

> **Purpose**: Systematic discovery and end-to-end verification of all web routes, user interactions, and UI components.
> **Workflow Binding**: Used directly during the [`/test`](../../workflows/test.md) workflow.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** testing rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/playwright_e2e_recipes.md`](./references/playwright_e2e_recipes.md) | Page Object Models, accessibility checks, screenshot diff fixtures | Writing Playwright test suites & UI flow automation |

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

---

## 🛠️ 3. Execution Commands

```powershell
# Run headless browser suite
npx playwright test

# Execute runtime python browser audit
python .agent/skills/webapp-testing/scripts/playwright_runner.py http://localhost:5173 --a11y
```