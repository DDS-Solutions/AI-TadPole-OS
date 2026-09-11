/**
 * @docs ARCHITECTURE:Quality:Verification
 *
 * ### AI Context Alignment
 * - **Subsystem**: Test Verification Suite / telemetryAndMetrics.spec
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[E2E]`
 * - **Witness Tests**: none declared
 */

import { test, expect } from '@playwright/test';

test.describe('Telemetry & Metrics E2E Suite', () => {

    test('should expose public Prometheus metrics on standard and versioned endpoints', async ({ request }) => {
        // Query the Rust backend directly on port 8000
        const backendUrl = 'http://localhost:8000';
        
        console.log('[E2E] Requesting standard /metrics...');
        const responseMain = await request.get(`${backendUrl}/metrics`);
        expect(responseMain.status()).toBe(200);
        expect(responseMain.headers()['content-type']).toContain('text/plain');
        
        const bodyMain = await responseMain.text();
        expect(bodyMain).toMatch(/(# HELP|process_cpu_seconds_total|tool_latency_p50)/);

        console.log('[E2E] Requesting versioned /v1/engine/metrics...');
        const responseVersioned = await request.get(`${backendUrl}/v1/engine/metrics`);
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
