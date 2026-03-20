import { test, expect } from '@playwright/test';

test.describe('Role Blueprint Promotion Flow', () => {
    test('should allow creating a new role from an agent configuration', async ({ page }) => {
        // 1. Navigate to dashboard
        await page.goto('/');

        // 2. Open Config Panel for a mock agent (e.g. Agent 1)
        // We assume the button has an ID or descriptive title from HierarchyNode
        await page.locator('button[title*="Configure Agent"]').first().click();

        // 3. Verify panel is open and expanded width is applied
        // Basic check for width - "max-w-xl" corresponds to approx 576px
        // await expect(page.locator('div:has-text("Neural Node Configuration")').locator('xpath=..')).toHaveCSS('max-width', '576px'); 

        // 4. Toggle a unique skill to differentiate the new role
        // Assuming skill buttons have titles or text
        await page.getByRole('button', { name: 'Deep Research' }).first().click();

        // 5. Open "Promote to Role" section
        await page.getByRole('button', { name: 'Promote to Role' }).click();

        // 6. Enter a new role name
        const newRoleName = `Test Role ${Date.now()}`;
        await page.getByPlaceholder('Enter new role name').fill(newRoleName);

        // 7. Click Promote
        await page.getByRole('button', { name: 'PROMOTE' }).click();

        // 8. Verify success notification in EventBus (Terminal/System Log)
        await expect(page.locator('div:has-text("saved to system library")')).toBeVisible();

        // 9. Verify the new role appears in the Role Select dropdown
        const roleSelect = page.locator('select').first();
        await roleSelect.selectOption(newRoleName);
        await expect(roleSelect).toHaveValue(newRoleName);
    });
});
