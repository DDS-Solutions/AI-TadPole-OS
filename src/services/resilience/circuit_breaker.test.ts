/**
 * @docs ARCHITECTURE:UI-Services:Resilience
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / Resilience / circuit_breaker.test
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
    CircuitBreaker,
    CircuitBreakerOpenError,
    TimeoutError,
    withTimeout,
    BREAKER_FAILURE_THRESHOLD,
    BREAKER_COOLDOWN_MS,
} from './circuit_breaker';

describe('CircuitBreaker', () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it('trips to OPEN after consecutive failures, gates concurrent HALF_OPEN probes, and recovers to CLOSED', async () => {
        const breaker = new CircuitBreaker('test-ns');
        expect(breaker.get_state()).toBe('CLOSED');

        for (let i = 0; i < BREAKER_FAILURE_THRESHOLD; i++) {
            await expect(breaker.execute(() => Promise.reject(new Error('boom')))).rejects.toThrow('boom');
        }
        expect(breaker.get_state()).toBe('OPEN');
        await expect(breaker.execute(() => Promise.resolve('ok'))).rejects.toBeInstanceOf(CircuitBreakerOpenError);

        // Advance past cooldown -> HALF_OPEN
        vi.advanceTimersByTime(BREAKER_COOLDOWN_MS + 10);
        expect(breaker.get_state()).toBe('HALF_OPEN');

        // While first probe is in flight, a concurrent probe must be rejected
        let resolveFirst!: (val: string) => void;
        const firstProbe = breaker.execute(() => new Promise<string>((r) => { resolveFirst = r; }));
        await expect(breaker.execute(() => Promise.resolve('concurrent'))).rejects.toBeInstanceOf(CircuitBreakerOpenError);
        resolveFirst('probe-1');
        expect(await firstProbe).toBe('probe-1');

        // Second successful probe recovers to CLOSED
        expect(await breaker.execute(() => Promise.resolve('probe-2'))).toBe('probe-2');
        expect(breaker.get_state()).toBe('CLOSED');
        expect(breaker.get_failures()).toBe(0);
    });

    it('re-opens immediately if a HALF_OPEN trial probe fails', async () => {
        const breaker = new CircuitBreaker('retry-ns');
        breaker.force_open();
        vi.advanceTimersByTime(BREAKER_COOLDOWN_MS + 10);
        expect(breaker.get_state()).toBe('HALF_OPEN');

        await expect(breaker.execute(() => Promise.reject(new Error('still down')))).rejects.toThrow('still down');
        expect(breaker.get_state()).toBe('OPEN');
    });
});

describe('withTimeout', () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it('rejects with TimeoutError when promise exceeds budget and resolves normally when timely', async () => {
        const fast = withTimeout(Promise.resolve(42), 500);
        expect(await fast).toBe(42);

        const slow = withTimeout(new Promise<number>(() => { /* never settles */ }), 100);
        const assertion = expect(slow).rejects.toBeInstanceOf(TimeoutError);
        await vi.advanceTimersByTimeAsync(150);
        await assertion;
    });
});
