/**
 * @docs ARCHITECTURE:Quality:Verification
 *
 * ### AI Assist Note
 * - **Subsystem**: Test Verification Suite / telemetryAndMetrics.spec
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[E2E]`
 */

import { test, expect } from '@playwright/test';
import { E2E_BACKEND_URL, E2E_ENGINE_TOKEN } from './backendFixture';

test.describe('Telemetry & Metrics E2E Suite', () => {

    test('should require engine authentication for versioned Prometheus metrics', async ({ request }) => {
        const endpoint = `${E2E_BACKEND_URL}/v1/engine/metrics`;
        const unauthenticated = await request.get(endpoint);
        expect(unauthenticated.status()).toBe(401);

        const unauthorized = await request.get(endpoint, {
            headers: { Authorization: 'Bearer test-only-invalid-token' },
        });
        expect(unauthorized.status()).toBe(401);

        const responseVersioned = await request.get(endpoint, {
            headers: { Authorization: `Bearer ${E2E_ENGINE_TOKEN}` },
        });
        expect(responseVersioned.status()).toBe(200);
        expect(responseVersioned.headers()['content-type']).toContain('text/plain');
        
        const bodyVersioned = await responseVersioned.text();
        expect(bodyVersioned).toMatch(/(# HELP|process_cpu_seconds_total|tool_latency_p50)/);
    });

    test('should mount the telemetry and metrics views in the dashboard UI', async ({ page }) => {
        console.log('[E2E] Navigating directly to /engine...');
        await page.setViewportSize({ width: 1920, height: 1080 });
        await page.goto('/engine', { waitUntil: 'networkidle' });
        
        // Ensure clean storage state
        await page.evaluate(() => localStorage.clear());
        await page.reload({ waitUntil: 'networkidle' });
        await page.waitForTimeout(3000);

        // Verify that the Telemetry components load
        console.log('[E2E] Verifying telemetry visualizers and telemetry data cards...');
        await expect(page.getByText(/Neural Engine Telemetry/i).first()).toBeVisible({ timeout: 15000 });
        
        // Verify key telemetry fields/charts are present
        const telemetryStats = page.locator('div:has-text("CPU Usage"), div:has-text("Memory"), div:has-text("Inference Latency"), div:has-text("Hardware Load")').first();
        await expect(telemetryStats).toBeVisible({ timeout: 10000 });

        console.log('[E2E] Telemetry Dashboard rendering validated.');
    });
});
