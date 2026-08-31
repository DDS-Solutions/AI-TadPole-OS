/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / error-bus.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi } from 'vitest';
import { ErrorBus } from './error-bus';
import type { ApiError } from '../errors/api-error';

describe('ErrorBus', () => {
    const create_mock_error = (message: string): ApiError => ({
        message,
        name: 'ApiError',
        status: 500
    } as unknown as ApiError);

    it('initializes empty and with initial listeners', () => {
        const empty_bus = new ErrorBus();
        expect(empty_bus.size).toBe(0);

        const listener_1 = vi.fn();
        const listener_2 = vi.fn();
        const seeded_bus = new ErrorBus([listener_1, listener_2]);
        expect(seeded_bus.size).toBe(2);
    });

    it('subscribes and unsubscribes listeners correctly', () => {
        const bus = new ErrorBus();
        const listener = vi.fn();

        const unsubscribe = bus.subscribe(listener);
        expect(bus.size).toBe(1);

        const err = create_mock_error('Network failure');
        bus.emit(err);
        expect(listener).toHaveBeenCalledWith(err);

        unsubscribe();
        expect(bus.size).toBe(0);

        bus.emit(create_mock_error('Another error'));
        expect(listener).toHaveBeenCalledTimes(1);
    });

    it('handles exceptions in listeners safely without halting emission', () => {
        const bus = new ErrorBus();
        const faulty_listener = vi.fn().mockImplementation(() => {
            throw new Error('Listener crashed');
        });
        const healthy_listener = vi.fn();

        bus.subscribe(faulty_listener);
        bus.subscribe(healthy_listener);

        const err = create_mock_error('Fatal error');
        expect(() => bus.emit(err)).not.toThrow();

        expect(faulty_listener).toHaveBeenCalledWith(err);
        expect(healthy_listener).toHaveBeenCalledWith(err);
    });

    it('supports Set-like methods: add, delete, clear, forEach, and iterator', () => {
        const bus = new ErrorBus();
        const l1 = vi.fn();
        const l2 = vi.fn();

        // add
        bus.add(l1);
        bus.add(l2);
        expect(bus.size).toBe(2);

        // forEach
        const collected: Array<unknown> = [];
        bus.forEach((l) => collected.push(l));
        expect(collected).toEqual([l1, l2]);

        // iterator
        const from_iter = Array.from(bus);
        expect(from_iter).toEqual([l1, l2]);

        // delete
        const deleted = bus.delete(l1);
        expect(deleted).toBe(true);
        expect(bus.size).toBe(1);

        // clear
        bus.clear();
        expect(bus.size).toBe(0);
    });
});
