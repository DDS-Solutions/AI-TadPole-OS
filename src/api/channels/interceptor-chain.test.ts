/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / interceptor-chain.test
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
import { InterceptorChain } from './interceptor-chain';

describe('InterceptorChain', () => {
    it('initializes empty and with initial interceptors', () => {
        const empty_chain = new InterceptorChain();
        expect(empty_chain.size).toBe(0);

        const i1 = vi.fn().mockReturnValue(null);
        const i2 = vi.fn().mockReturnValue(null);
        const seeded_chain = new InterceptorChain([i1, i2]);
        expect(seeded_chain.size).toBe(2);
    });

    it('registers and unregisters interceptors correctly', async () => {
        const chain = new InterceptorChain();
        const interceptor = vi.fn().mockReturnValue(null);

        const unregister = chain.register(interceptor);
        expect(chain.size).toBe(1);

        const res = await chain.run('/v1/test', { method: 'GET' });
        expect(res).toBeNull();
        expect(interceptor).toHaveBeenCalledWith('/v1/test', { method: 'GET' });

        unregister();
        expect(chain.size).toBe(0);
    });

    it('returns intercepted result on first matching non-null return', async () => {
        const chain = new InterceptorChain();
        const mock_data = { intercepted: true };

        const pass_through = vi.fn().mockReturnValue(null);
        const intercepting = vi.fn().mockReturnValue(Promise.resolve(mock_data));
        const unreachable = vi.fn().mockReturnValue(Promise.resolve({ unreachable: true }));

        chain.register(pass_through);
        chain.register(intercepting);
        chain.register(unreachable);

        const result = await chain.run('/v1/intercept');
        expect(result).toEqual(mock_data);
        expect(pass_through).toHaveBeenCalled();
        expect(intercepting).toHaveBeenCalled();
        expect(unreachable).not.toHaveBeenCalled();
    });

    it('supports Set-like methods: add, delete, clear, forEach, and iterator', () => {
        const chain = new InterceptorChain();
        const i1 = vi.fn().mockReturnValue(null);
        const i2 = vi.fn().mockReturnValue(null);

        chain.add(i1);
        chain.add(i2);
        expect(chain.size).toBe(2);

        const collected: Array<unknown> = [];
        chain.forEach((i) => collected.push(i));
        expect(collected).toEqual([i1, i2]);

        const from_iter = Array.from(chain);
        expect(from_iter).toEqual([i1, i2]);

        expect(chain.delete(i1)).toBe(true);
        expect(chain.size).toBe(1);

        chain.clear();
        expect(chain.size).toBe(0);
    });
});
