import { test, expect } from '@playwright/test';

test.describe('Mission Lifecycle E2E', () => {

    test('should complete a full mission journey', async ({ page }) => {
        // 1. Navigation to Missions and Creation
        console.debug('[E2E] Navigating to /missions...');
        try {
            await page.setViewportSize({ width: 1920, height: 1080 });
            await page.goto('/missions', { waitUntil: 'networkidle' });

            // Clear storage and reload to ensure clean state
            await page.evaluate(() => localStorage.clear());
            await page.reload({ waitUntil: 'networkidle' });
            await page.waitForTimeout(5000);

            console.debug('[E2E] On Missions page. Looking for NEW MISSION button...');
            // Fuzzy match button containing "MISSION"
            const newMissionBtn = page.locator('button').filter({ hasText: /MISSION/i }).first();
            await expect(newMissionBtn).toBeVisible({ timeout: 20000 });

            console.debug('[E2E] Clicking NEW MISSION button...');
            await newMissionBtn.click({ force: true, delay: 1000 });

            console.debug('[E2E] Waiting for creation form...');
            await page.waitForTimeout(2000);
            const nameInput = page.getByLabel(/Mission Name/i).first();
            await nameInput.waitFor({ state: 'visible', timeout: 15000 });
            await expect(nameInput).toBeVisible();
            await page.waitForTimeout(1000);

            console.debug('[E2E] Filling mission form...');
            await nameInput.fill('E2E Spec Ops');

            // Select Executive department
            await page.locator('select').selectOption('Executive');

            // Assign first available agent
            const assignBtn = page.locator('button:has-text("ASSIGN")').first();
            if (await assignBtn.isVisible()) {
                await assignBtn.click({ force: true });
            }

            console.debug('[E2E] Clicking CREATE button...');
            const createBtn = page.locator('button:has-text("CREATE")').first();
            await createBtn.click({ force: true });

            // Verify cluster exists
            console.debug('[E2E] Verification: cluster list...');
            await expect(page.getByText('E2E Spec Ops')).toBeVisible({ timeout: 20000 });

            // Trigger a Swarm Proposal
            console.debug('[E2E] Triggering swarm proposal...');
            await page.getByText('E2E Spec Ops').first().click();
            await page.waitForTimeout(2000);

            const objectiveTextarea = page.locator('textarea').first();
            await objectiveTextarea.fill('Optimize mission parameters for E2E verification.');
            await page.keyboard.press('Tab');

            console.debug('[E2E] Waiting for swarm proposal UI...');
            // Avoid strict mode violation by using a more unique header
            await expect(page.getByText(/Neural Swarm Optimization Proffered/i)).toBeVisible({ timeout: 20000 });

            // 2. Check Engine
            console.debug('[E2E] Navigating to Engine Dashboard...');
            await page.getByTitle(/Engine Dashboard/i).click();
            await expect(page.getByText(/Neural Engine Telemetry/i)).toBeVisible({ timeout: 20000 });

            // 3. Oversight
            console.debug('[E2E] Navigating to Oversight Dashboard...');
            await page.getByTitle(/Oversight/i).click();
            await page.waitForTimeout(5000);

            console.debug('[E2E] Verifying Oversight page content...');
            // Use specific header text found in OversightDashboard.tsx
            await expect(page.getByText(/Swarm Intelligence Oversight/i)).toBeVisible({ timeout: 20000 });

            console.debug('[E2E] Mission Lifecycle Verified.');
        } catch (error) {
            console.error('[E2E] Error occurred:', error);
            await page.screenshot({ path: 'tests/e2e/failure-final.png', fullPage: true });
            throw error;
        }
    });

});

