/**
 * @docs ARCHITECTURE:Quality:Verification
 *
 * ### AI Assist Note
 * - **Subsystem**: E2E backend fixture / disposable local engine credentials
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared; this module exports fixture constants.
 */

export const E2E_BACKEND_URL = 'http://localhost:8000';
export const E2E_ENGINE_TOKEN = 'test-only-engine-access';
export const E2E_ADMIN_TOKEN = 'test-only-engine-admin';
