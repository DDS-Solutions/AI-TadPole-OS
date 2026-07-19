/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **ReconnectionPolicy**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[reconnection_policy]` in observability traces.
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

    get_delay(retry_count: number): number {
        return Math.min(this.initial_backoff * Math.pow(2, retry_count), this.max_backoff);
    }

    should_retry(retry_count: number): boolean {
        return retry_count < this.max_retries;
    }
}

// Metadata: [reconnection_policy]
