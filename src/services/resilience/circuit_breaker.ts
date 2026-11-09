/**
 * @docs ARCHITECTURE:UI-Services:Resilience
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / Resilience / Circuit Breaker
 * - **Primary Entrypoints**: `CircuitBreaker`, `CircuitBreakerOpenError`, `TimeoutError`, `withTimeout`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Circuit breaker manages failure thresholds, cooldowns, and half-open probe transitions.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: `CircuitBreakerOpenError`, `TimeoutError`
 * - **Telemetry Targets**: none declared
 */

import { event_bus } from '../event_bus';

const get_env_number = (key: string, default_val: number): number => {
    if (typeof import.meta !== 'undefined' && import.meta.env?.[key]) {
        const val = String(import.meta.env[key]).trim();
        if (/^\d+$/.test(val)) {
            const parsed = parseInt(val, 10);
            if (!isNaN(parsed)) {
                return parsed;
            }
        }
    }
    return default_val;
};

export const BREAKER_FAILURE_THRESHOLD = get_env_number('VITE_SYSTEM_BREAKER_FAILURE_THRESHOLD', 5);
export const BREAKER_COOLDOWN_MS = get_env_number('VITE_SYSTEM_BREAKER_COOLDOWN_MS', 10000);
export const DEFAULT_TIMEOUT_MS = get_env_number('VITE_SYSTEM_DEFAULT_TIMEOUT_MS', 30000);

export type BreakerState = 'CLOSED' | 'OPEN' | 'HALF_OPEN';

export class CircuitBreakerOpenError extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'CircuitBreakerOpenError';
        Object.setPrototypeOf(this, CircuitBreakerOpenError.prototype);
    }
}

export class TimeoutError extends Error {
    constructor(message = 'Request timed out at resilience boundary') {
        super(message);
        this.name = 'TimeoutError';
        Object.setPrototypeOf(this, TimeoutError.prototype);
    }
}

export function withTimeout<T>(promise: Promise<T>, timeoutMs: number): Promise<T> {
    let timeoutId: ReturnType<typeof setTimeout> | undefined;
    const timeoutPromise = new Promise<never>((_, reject) => {
        timeoutId = setTimeout(() => {
            reject(new TimeoutError('Request timed out at resilience boundary'));
        }, timeoutMs);
    });
    return Promise.race([promise, timeoutPromise]).finally(() => {
        clearTimeout(timeoutId);
    });
}

export class CircuitBreaker {
    private state: BreakerState = 'CLOSED';
    private failures = 0;
    private successes = 0;
    private lastFailureTime = 0;
    private readonly failureThreshold = BREAKER_FAILURE_THRESHOLD;
    private readonly cooldownPeriod = BREAKER_COOLDOWN_MS;
    private readonly halfOpenSuccessThreshold = 2;
    private probeInFlight = false;
    private ns: string;

    constructor(ns: string) {
        this.ns = ns;
    }

    public get_failures(): number {
        return this.failures;
    }

    public get_last_failure_time(): number {
        return this.lastFailureTime;
    }

    public async execute<T>(fn: () => Promise<T>): Promise<T> {
        this.updateState();

        if (this.state === 'OPEN') {
            throw new CircuitBreakerOpenError(
                `Service namespace ${this.ns} temporarily offline due to repeated failures (Circuit Breaker OPEN)`
            );
        }

        if (this.state === 'HALF_OPEN') {
            if (this.probeInFlight) {
                throw new CircuitBreakerOpenError(
                    `Service namespace ${this.ns} is in probe state (HALF_OPEN); trial probe request already in flight`
                );
            }
            this.probeInFlight = true;
        }

        const isTrialProbe = this.state === 'HALF_OPEN';

        try {
            const result = await fn();
            if (this.state === 'CLOSED') {
                this.failures = 0; // Decay consecutive failure counter on successful execution
            } else if (this.state === 'HALF_OPEN') {
                this.successes++;
                if (this.successes >= this.halfOpenSuccessThreshold) {
                    this.reset();
                }
            }
            return result;
        } catch (error) {
            this.recordFailure();
            throw error;
        } finally {
            if (isTrialProbe) {
                this.probeInFlight = false;
            }
        }
    }

    private updateState() {
        if (this.state === 'OPEN' && Date.now() - this.lastFailureTime > this.cooldownPeriod) {
            this.state = 'HALF_OPEN';
            this.successes = 0;
            event_bus.emit_log({
                source: 'System',
                severity: 'warning',
                text: `📡 [Circuit Breaker] ${this.ns.toUpperCase()} entered HALF_OPEN probe state.`,
                metadata: { namespace: this.ns, state: 'HALF_OPEN' }
            });
        }
    }

    private recordFailure() {
        this.failures++;
        this.lastFailureTime = Date.now();
        if (this.state === 'CLOSED' && this.failures >= this.failureThreshold) {
            this.state = 'OPEN';
            console.warn(`[Circuit Breaker] Tripping to OPEN due to ${this.failures} consecutive failures.`);
            event_bus.emit_log({
                source: 'System',
                severity: 'error',
                text: `❌ [Circuit Breaker] ${this.ns.toUpperCase()} tripped to OPEN due to ${this.failures} consecutive failures.`,
                metadata: { namespace: this.ns, state: 'OPEN', failures: this.failures }
            });
        } else if (this.state === 'HALF_OPEN') {
            this.state = 'OPEN';
            console.warn('[Circuit Breaker] Tripping back to OPEN from HALF_OPEN due to trial failure.');
            event_bus.emit_log({
                source: 'System',
                severity: 'error',
                text: `❌ [Circuit Breaker] ${this.ns.toUpperCase()} tripped back to OPEN from HALF_OPEN due to trial failure.`,
                metadata: { namespace: this.ns, state: 'OPEN' }
            });
        }
    }

    private reset() {
        this.state = 'CLOSED';
        this.failures = 0;
        this.successes = 0;
        console.debug('[Circuit Breaker] Recovered to CLOSED state.');
        event_bus.emit_log({
            source: 'System',
            severity: 'success',
            text: `✅ [Circuit Breaker] ${this.ns.toUpperCase()} recovered to CLOSED state.`,
            metadata: { namespace: this.ns, state: 'CLOSED' }
        });
    }

    public get_state(): BreakerState {
        this.updateState();
        return this.state;
    }

    public force_open() {
        this.state = 'OPEN';
        this.failures = this.failureThreshold;
        this.lastFailureTime = Date.now();
    }

    public force_close() {
        this.reset();
    }
}
