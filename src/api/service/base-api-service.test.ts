/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / base-api-service.test
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
import { BaseApiService, normalize_headers } from './base-api-service';
import type { ApiServiceConfig } from '../types';

describe('BaseApiService & normalize_headers', () => {
    describe('normalize_headers helper', () => {
        it('returns an empty object when headers are undefined or empty', () => {
            expect(normalize_headers()).toEqual({});
            expect(normalize_headers(undefined)).toEqual({});
        });

        it('normalizes Headers instances into a key-value record', () => {
            const h = new Headers();
            h.set('x-foo', 'bar');
            h.set('authorization', 'Bearer secret-token');
            const normalized = normalize_headers(h);
            expect(normalized).toEqual({
                'x-foo': 'bar',
                'authorization': 'Bearer secret-token'
            });
        });

        it('normalizes array of header tuples', () => {
            const tuples: [string, string][] = [
                ['x-request-id', 'req-123'],
                ['accept', 'application/json']
            ];
            const normalized = normalize_headers(tuples);
            expect(normalized).toEqual({
                'x-request-id': 'req-123',
                'accept': 'application/json'
            });
        });

        it('preserves plain object records', () => {
            const obj = { 'content-type': 'application/json', 'x-trace': 'trace-abc' };
            const normalized = normalize_headers(obj);
            expect(normalized).toEqual(obj);
        });
    });

    describe('BaseApiService request execution', () => {
        let mockFetch: ReturnType<typeof vi.fn>;
        let mockAddSpan: ReturnType<typeof vi.fn>;
        let mockUpdateSpan: ReturnType<typeof vi.fn>;
        let mockGetSettings: ReturnType<typeof vi.fn>;
        let config: ApiServiceConfig;
        let service: BaseApiService;

        beforeEach(() => {
            mockFetch = vi.fn();
            mockAddSpan = vi.fn();
            mockUpdateSpan = vi.fn();
            mockGetSettings = vi.fn().mockReturnValue({
                tadpole_os_url: 'http://localhost:8000',
                tadpole_os_api_key: 'test-api-token'
            });

            config = {
                httpAdapter: {
                    fetch: mockFetch as unknown as typeof fetch,
                    crypto: crypto
                },
                telemetryPort: {
                    addSpan: mockAddSpan,
                    updateSpan: mockUpdateSpan
                },
                settingsPort: {
                    getSettings: mockGetSettings
                }
            };

            service = new BaseApiService(config);
        });

        it('executes successful GET request and parses JSON body', async () => {
            mockFetch.mockResolvedValueOnce(
                new Response(JSON.stringify({ status: 'ok', data: 123 }), {
                    status: 200,
                    headers: { 'Content-Type': 'application/json' }
                })
            );

            const result = await service.request<{ status: string; data: number }>('/api/status');
            expect(result).toEqual({ status: 'ok', data: 123 });
            expect(mockFetch).toHaveBeenCalledTimes(1);
            expect(mockAddSpan).toHaveBeenCalledTimes(1);
            expect(mockUpdateSpan).toHaveBeenCalledWith(
                expect.any(String),
                expect.objectContaining({ status: 'success' })
            );
        });

        it('properly propagates custom headers passed as Headers instance without dropping', async () => {
            mockFetch.mockResolvedValueOnce(
                new Response(JSON.stringify({ ok: true }), { status: 200 })
            );

            const headersInstance = new Headers();
            headersInstance.set('X-Custom-Tenant', 'tenant-999');
            headersInstance.set('X-Request-Id', 'custom-uuid-456');

            await service.request('/api/custom', {
                headers: headersInstance
            });

            expect(mockFetch).toHaveBeenCalledTimes(1);
            const callArgs = mockFetch.mock.calls[0];
            const passedHeaders = callArgs[1]?.headers as Record<string, string>;

            expect(passedHeaders['x-custom-tenant'] || passedHeaders['X-Custom-Tenant']).toBe('tenant-999');
            expect(passedHeaders['X-Request-Id'] || passedHeaders['x-request-id']).toBe('custom-uuid-456');
            expect(passedHeaders['Authorization']).toBe('Bearer test-api-token');
        });

        it('handles 204 No Content by returning null', async () => {
            mockFetch.mockResolvedValueOnce(new Response(null, { status: 204 }));

            const result = await service.request('/api/empty');
            expect(result).toBeNull();
        });

        it('supports response_type = "text"', async () => {
            mockFetch.mockResolvedValueOnce(new Response('raw-log-output', { status: 200 }));

            const result = await service.request('/api/logs', { response_type: 'text' });
            expect(result).toBe('raw-log-output');
        });
    });
});
