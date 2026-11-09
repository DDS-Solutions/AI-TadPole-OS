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

## 2. Deterministic Clock Mocking Recipe

When testing debounced search bars, polling intervals, or token refresh timers, never use real-time delays:

```typescript
test('fast-forwards debounced search input', async ({ page }) => {
  // Install mock clock before navigation
  await page.clock.install();
  await page.goto('/neural-map');

  const searchInput = page.getByPlaceholder(/search symbols/i);
  await searchInput.fill('get_blast_radius');

  // Fast-forward 500ms debounce without real-time sleep
  await page.clock.fastForward(500);

  const results = page.getByTestId('symbol-result-item');
  await expect(results).toHaveCount(1);
});
```

---

## 3. Strict Route Interception & Network Mocking

```typescript
test('handles Axum backend network failure gracefully', async ({ page }) => {
  // Intercept backend API endpoint to simulate 503 Service Unavailable
  await page.route('**/api/v1/system/status', async (route) => {
    await route.fulfill({
      status: 503,
      contentType: 'application/json',
      body: JSON.stringify({ error: 'Daemon offline' }),
    });
  });

  await page.goto('/ops');
  await expect(page.getByText(/daemon offline/i)).toBeVisible();
});
```

---

## 4. Automated Accessibility Audit Script (`scripts/playwright_runner.py`)

```powershell
# Run basic automated browser check with screenshot and a11y audit
python .agent/skills/webapp-testing/scripts/playwright_runner.py http://localhost:5173 --screenshot --a11y
```