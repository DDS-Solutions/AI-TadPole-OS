/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / constants
 * - **Primary Entrypoints**: `DEFAULT_TIMEOUT`, `DEPLOY_TIMEOUT`, `MAX_RETRIES`, `INITIAL_RETRY_DELAY`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export const DEFAULT_TIMEOUT = 30000; // 30 seconds default
export const DEPLOY_TIMEOUT = 7200000; // 2 hours for deployment
export const MAX_RETRIES = 3;
export const INITIAL_RETRY_DELAY = 1000;
