/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / reconnection_policy
 * - **Primary Entrypoints**: `ReconnectionPolicy`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

const DEFAULT_MAX_RETRIES = 10;
const DEFAULT_INITIAL_BACKOFF = 2000;
const DEFAULT_MAX_BACKOFF = 30000;

/**
 * ReconnectionPolicy
 * Implements exponential backoff retry delays.
 */
export class ReconnectionPolicy {
    private readonly initial_backoff: number;
    private readonly max_backoff: number;
    private readonly max_retries: number;

    constructor(
        initial_backoff = DEFAULT_INITIAL_BACKOFF,
        max_backoff = DEFAULT_MAX_BACKOFF,
        max_retries = DEFAULT_MAX_RETRIES
    ) {
        this.initial_backoff = initial_backoff;
        this.max_backoff = max_backoff;
        this.max_retries = max_retries;
    }

    get_delay(retry_count: number, with_jitter = false): number {
        const base = Math.min(this.initial_backoff * Math.pow(2, retry_count), this.max_backoff);
        if (with_jitter) {
            const jitter = Math.floor(Math.random() * 500);
            return base + jitter;
        }
        return base;
    }

    should_retry(retry_count: number): boolean {
        return retry_count < this.max_retries;
    }
}
