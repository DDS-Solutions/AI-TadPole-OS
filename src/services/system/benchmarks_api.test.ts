/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / benchmarks_api.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { benchmarks_api } from './benchmarks_api';
import { api_request } from '../base_api_service';

vi.mock('../base_api_service', () => ({
    api_request: vi.fn()
}));

describe('benchmarks_api', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('get_benchmarks', () => {
        it('calls GET /v1/benchmarks', async () => {
            const mock_records = [{ id: 'b-1', name: 'speed_test', test_id: 't1', category: 'speed', mean_ms: 120, status: 'success', created_at: '2026-06-07T12:00:00Z' }];
            vi.mocked(api_request).mockResolvedValueOnce(mock_records);

            const result = await benchmarks_api.get_benchmarks();
            expect(result).toEqual(mock_records);
            expect(api_request).toHaveBeenCalledWith('/v1/benchmarks', expect.objectContaining({
                method: 'GET'
            }));
        });

        it('propagates abort signal and timeout option', async () => {
            const controller = new AbortController();
            vi.mocked(api_request).mockResolvedValueOnce([]);

            await benchmarks_api.get_benchmarks({ signal: controller.signal, timeout: 2500 });
            expect(api_request).toHaveBeenCalledWith('/v1/benchmarks', expect.objectContaining({
                signal: controller.signal,
                timeout: 2500
            }));
        });
    });

    describe('run_benchmark', () => {
        it('calls POST /v1/benchmarks/run/{test_id} on safe identifier', async () => {
            const mock_record = { id: 'b-2', name: 'bench2', test_id: 'test-2.v1', category: 'memory', mean_ms: 45, status: 'success', created_at: '2026-06-07T12:00:00Z' };
            vi.mocked(api_request).mockResolvedValueOnce(mock_record);

            const result = await benchmarks_api.run_benchmark('test-2.v1');
            expect(result).toEqual(mock_record);
            expect(api_request).toHaveBeenCalledWith('/v1/benchmarks/run/test-2.v1', expect.objectContaining({
                method: 'POST'
            }));
        });

        it('blocks invalid characters and traversal attempts', async () => {
            await expect(benchmarks_api.run_benchmark('subdir/../../run'))
                .rejects.toThrow('Invalid identifier: subdir/../../run');
            await expect(benchmarks_api.run_benchmark('test-1 2'))
                .rejects.toThrow('Invalid identifier: test-1 2');
            await expect(benchmarks_api.run_benchmark('test-1\u200b'))
                .rejects.toThrow('Invalid identifier: test-1\u200b');
            await expect(benchmarks_api.run_benchmark('a'.repeat(65)))
                .rejects.toThrow('Invalid identifier:');
            expect(api_request).not.toHaveBeenCalled();
        });

        it('propagates abort signal and timeout option', async () => {
            const controller = new AbortController();
            vi.mocked(api_request).mockResolvedValueOnce({});

            await benchmarks_api.run_benchmark('t1', { signal: controller.signal, timeout: 3000 });
            expect(api_request).toHaveBeenCalledWith('/v1/benchmarks/run/t1', expect.objectContaining({
                signal: controller.signal,
                timeout: 3000
            }));
        });
    });
});
