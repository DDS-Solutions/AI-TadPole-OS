/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / factory.test
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
import { createApiService, resolveCrypto } from './factory';
import { BaseApiService } from './service';

describe('factory: resolveCrypto & createApiService', () => {
    describe('resolveCrypto', () => {
        it('returns explicit override if provided', () => {
            const fakeCrypto = {
                getRandomValues: vi.fn(),
                randomUUID: vi.fn()
            } as unknown as Crypto;

            expect(resolveCrypto(fakeCrypto)).toBe(fakeCrypto);
        });

        it('resolves runtime global crypto when available', () => {
            const resolved = resolveCrypto();
            expect(resolved).toBeDefined();
            expect(typeof resolved.getRandomValues).toBe('function');
        });
    });

    describe('createApiService', () => {
        it('constructs a BaseApiService instance with default dependencies', () => {
            const service = createApiService();
            expect(service).toBeInstanceOf(BaseApiService);
        });

        it('supports injecting custom fetch while automatically resolving crypto', () => {
            const customFetch = vi.fn().mockResolvedValue(new Response(JSON.stringify({ ok: true })));
            const service = createApiService({
                httpAdapter: {
                    fetch: customFetch
                } as unknown as { fetch: typeof fetch; crypto: typeof crypto }
            });
            expect(service).toBeInstanceOf(BaseApiService);
        });

        it('throws when crypto adapter is explicitly undefined or falsy in httpAdapter', () => {
            expect(() => {
                createApiService({
                    httpAdapter: {
                        fetch: vi.fn(),
                        crypto: undefined as unknown as typeof crypto
                    }
                });
            }).toThrow(/crypto adapter is mandatory/);
        });

        it('allows custom telemetry, settings, and timers', () => {
            const mockAddSpan = vi.fn();
            const mockUpdateSpan = vi.fn();
            const mockGetSettings = vi.fn().mockReturnValue({ tadpole_os_url: 'http://localhost:8000' });
            const mockSetTimeout = vi.fn().mockReturnValue(123 as unknown as NodeJS.Timeout);
            const mockClearTimeout = vi.fn();

            const service = createApiService({
                telemetryPort: {
                    addSpan: mockAddSpan,
                    updateSpan: mockUpdateSpan
                },
                settingsPort: {
                    getSettings: mockGetSettings
                },
                timers: {
                    setTimeout: mockSetTimeout,
                    clearTimeout: mockClearTimeout
                }
            });

            expect(service).toBeInstanceOf(BaseApiService);
        });
    });
});
