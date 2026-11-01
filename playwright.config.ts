/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Assist Note
 * - **Subsystem**: System Core / playwright.config
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { defineConfig, devices } from '@playwright/test';
import { mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { E2E_ADMIN_TOKEN, E2E_BACKEND_URL, E2E_ENGINE_TOKEN } from './tests/e2e/backendFixture';

const backendWorkspace = mkdtempSync(join(tmpdir(), 'tadpole-e2e-'));

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
    testDir: './tests/e2e',
    /* Maximum time one test can run for. */
    timeout: 60 * 1000,
    expect: {
        /**
         * Maximum time expect() should wait for the condition to be met.
         * For example in `await expect(locator).toHaveText();`
         */
        timeout: 10 * 1000
    },
    /* Run tests in files in parallel */
    fullyParallel: false,
    /* Fail the build on CI if you accidentally left test.only in the source code. */
    forbidOnly: !!process.env.CI,
    /* Retry on CI only */
    retries: process.env.CI ? 2 : 0,
    /* Opt out of parallel tests on CI. */
    workers: process.env.CI ? 1 : undefined,
    /* Reporter to use. See https://playwright.dev/docs/test-reporters */
    reporter: [['list'], ['html', { open: 'never' }]],
    /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
    use: {
        /* Base URL to use in actions like `await page.goto('/')`. */
        baseURL: 'http://localhost:5173',

        /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
        trace: 'retain-on-failure',
        screenshot: 'only-on-failure',
        video: 'retain-on-failure',

        // Smooth interaction
        viewport: { width: 1280, height: 720 },
    },

    /* Configure projects for major browsers */
    projects: [
        {
            name: 'chromium',
            use: { ...devices['Desktop Chrome'] },
        },
        {
            name: 'firefox',
            use: { ...devices['Desktop Firefox'] },
        },
        {
            name: 'webkit',
            use: { ...devices['Desktop Safari'] },
        },
    ],

    /* Run your local dev server and Rust backend before starting the tests */
    webServer: [
        {
            command: 'npm run dev',
            url: 'http://localhost:5173',
            reuseExistingServer: !process.env.CI,
            timeout: 180 * 1000,
        },
        {
            command: 'cargo run --manifest-path server-rs/Cargo.toml',
            url: `${E2E_BACKEND_URL}/v1/engine/health`,
            reuseExistingServer: false,
            env: {
                TADPOLE_ENV: 'test',
                NEURAL_TOKEN: E2E_ENGINE_TOKEN,
                NEURAL_ENGINE_ACCESS_TOKEN: E2E_ENGINE_TOKEN,
                NEURAL_ADMIN_TOKEN: E2E_ADMIN_TOKEN,
                ADMIN_TOKEN: E2E_ADMIN_TOKEN,
                WORKSPACE_ROOT: backendWorkspace,
                DATABASE_URL: `sqlite:${join(backendWorkspace, 'tadpole.db').replaceAll('\\', '/')}`,
                SKIP_DB_SEED: 'true',
                TADPOLE_NULL_PROVIDERS: 'true',
                DISABLE_TELEMETRY: 'true',
                TADPOLE_TRUST_INTERPRETERS: '0',
                TADPOLE_TRUST_HOOK_SHELLS: '0',
            },
            timeout: 180 * 1000,
        },
    ],
});
