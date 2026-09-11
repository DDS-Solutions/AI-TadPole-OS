/**
 * @docs ARCHITECTURE:Quality:Verification
 *
 * ### AI Context Alignment
 * - **Subsystem**: Test Verification Suite / chatDirect.spec
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[E2E]`
 * - **Witness Tests**: none declared
 */

import { test } from '@playwright/test';

test.describe('Chat Direct E2E', () => {
    test('should send message to Agent of Nine', async ({ page }) => {
        console.debug('[E2E] Navigating to dashboard...');
        
        // Listen to console and page error events to capture failures
        page.on('console', msg => {
            console.debug(`[BROWSER CONSOLE] [${msg.type()}]: ${msg.text()}`);
        });

        page.on('pageerror', err => {
            console.error(`[BROWSER PAGEERROR]: ${err.message}\n${err.stack}`);
        });

        // Monitor all network requests and responses
        page.on('request', request => {
            if (request.url().includes('/tasks') || request.url().includes('/chat')) {
                console.debug(`[NET REQ]: ${request.method()} ${request.url()}`);
                console.debug(`[NET REQ BODY]: ${request.postData()}`);
            }
        });

        page.on('response', async response => {
            if (response.url().includes('/tasks') || response.url().includes('/chat')) {
                console.debug(`[NET RES]: ${response.status()} ${response.url()}`);
                try {
                    const text = await response.text();
                    console.debug(`[NET RES BODY]: ${text}`);
                } catch (e) {
                    console.debug(`[NET RES BODY ERROR]: Failed to read body: ${e}`);
                }
            }
        });

        await page.goto('/dashboard', { waitUntil: 'networkidle' });

        // Seed settings store in localStorage to bypass token / url setup
        await page.evaluate(() => {
            localStorage.setItem('tadpole_settings', JSON.stringify({
                state: {
                    settings: {
                        tadpole_os_url: 'http://localhost:8000',
                        tadpole_os_api_key: 'tadpole-2026-dev',
                        theme: 'zinc',
                        density: 'compact',
                        default_model: 'gemma4:31b-cloud',
                        default_temperature: 0.7,
                        auto_approve_safe_skills: true,
                        max_agents: 50,
                        max_clusters: 10,
                        max_swarm_depth: 5,
                        max_task_length: 32768,
                        default_budget_usd: 1.0,
                        is_safe_mode: false,
                        privacy_mode: false
                    }
                },
                version: 0
            }));
            sessionStorage.setItem('tadpole-vault-master-key', 'my-vault-pass');
            localStorage.setItem('tadpole-vault-secrets', JSON.stringify({
                state: {
                    is_locked: false,
                    master_key: 'my-vault-pass',
                    encrypted_configs: {}
                },
                version: 0
            }));
        });

        // Reload to apply settings
        await page.reload({ waitUntil: 'networkidle' });
        await page.waitForTimeout(3000);

        // Click the minimized Chat button if it exists
        const chatButton = page.locator('button:has-text("CHAT")');
        if (await chatButton.isVisible()) {
            console.debug('[E2E] Maximizing Chat panel...');
            await chatButton.click();
            await page.waitForTimeout(1000);
        }

        // Fill in the message input field
        const inputField = page.locator('input[type="text"]');
        await inputField.waitFor({ state: 'visible', timeout: 15000 });
        console.debug('[E2E] Typing "hi" into Chat input...');
        await inputField.fill('hi');

        // Press enter to send
        console.debug('[E2E] Sending message...');
        await inputField.press('Enter');

        // Wait for system logs, telemetry, or responses
        await page.waitForTimeout(15000);
        console.debug('[E2E] Done waiting.');
    });
});
