> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / webapp-testing
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Flaky selector timeouts or browser harness crashes.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[WEBAPP_TESTING]`)

# Web App Testing & Playwright Recipes (L3)

---

## 1. Page Object Model (POM) Template

```typescript
import { Page, Locator, expect } from '@playwright/test';

export class DashboardPage {
  readonly page: Page;
  readonly statusIndicator: Locator;
  readonly sendTaskButton: Locator;

  constructor(page: Page) {
    this.page = page;
    this.statusIndicator = page.getByTestId('system-status-indicator');
    this.sendTaskButton = page.getByRole('button', { name: /dispatch/i });
  }

  async goto() {
    await this.page.goto('http://localhost:5173/ops');
  }

  async verifyOnline() {
    await expect(this.statusIndicator).toHaveText('ONLINE');
  }
}
```

---

## 2. Automated Accessibility Audit Script (`scripts/playwright_runner.py`)

```powershell
# Run basic automated browser check with screenshot
python .agent/skills/webapp-testing/scripts/playwright_runner.py http://localhost:5173 --screenshot --a11y
```