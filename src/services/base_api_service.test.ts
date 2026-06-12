/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the base HTTP and Telemetry API clients.**
 * Verifies core HTTP requests, sanitization, secret scrubbing, retries, and request interception.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Interceptor exceptions, incorrect URL parsing, or failure to scrub credentials.
 * - **Telemetry Link**: Search `[base_api_service.test]` in tracing logs.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
    BaseApiService,
    combine_signals,
    validate_and_sanitize_url,
    scrub_secrets,
    scrub_string,
    truncate_payload,
    AuthError,
    ValidationError,
    ServerError,
    subscribe_api_errors,
    sanitize_error_detail,
    createApiService,
    register_request_interceptor
} from './base_api_service';

describe('base_api_service utilities', () => {
    describe('validate_and_sanitize_url', () => {
        it('should strip username and password credentials from URL', () => {
            const sanitized = validate_and_sanitize_url('https://user:pass@example.com/api');
            expect(sanitized).toBe('https://example.com/api');
        });

        it('should allow local loopback HTTP connections', () => {
            const localhost = validate_and_sanitize_url('http://localhost:8000');
            expect(localhost).toBe('http://localhost:8000');

            const ip = validate_and_sanitize_url('http://127.0.0.1:3000');
            expect(ip).toBe('http://127.0.0.1:3000');

            const ipv6 = validate_and_sanitize_url('http://[::1]:3000');
            expect(ipv6).toBe('http://[::1]:3000');

            const subLocalhost = validate_and_sanitize_url('http://my-service.localhost:3000');
            expect(subLocalhost).toBe('http://my-service.localhost:3000');
        });

        it('should block external and RFC1918 HTTP connections and enforce HTTPS', () => {
            expect(() => validate_and_sanitize_url('http://external-api.com')).toThrow(
                /Insecure transmission blocked/
            );
            expect(() => validate_and_sanitize_url('http://10.0.0.1')).toThrow(
                /Insecure transmission blocked/
            );
            expect(() => validate_and_sanitize_url('http://10.0.0.1')).toThrow(
                /Insecure transmission blocked/
            );
            expect(() => validate_and_sanitize_url('http://10.0.0.1')).toThrow(
                /Insecure transmission blocked/
            );
        });

        it('should strip trailing slash', () => {
            const sanitized = validate_and_sanitize_url('https://example.com/');
            expect(sanitized).toBe('https://example.com');
        });

        it('should throw on empty or invalid URL format', () => {
            expect(() => validate_and_sanitize_url('')).toThrow('URL is empty');
            expect(() => validate_and_sanitize_url('not a url')).toThrow('Invalid URL format');
        });
    });

    describe('combine_signals', () => {
        it('should return empty object if no signals are passed', () => {
            const result = combine_signals();
            expect(result.signal).toBeUndefined();
            expect(result.cleanup).toBeUndefined();
        });

        it('should return single signal if only one signal is passed', () => {
            const controller = new AbortController();
            const result = combine_signals(controller.signal);
            expect(result.signal).toBe(controller.signal);
            expect(result.cleanup).toBeUndefined();
        });

        it('should combine multiple signals and abort combined when one aborts', () => {
            const c1 = new AbortController();
            const c2 = new AbortController();

            const { signal, cleanup } = combine_signals(c1.signal, c2.signal);
            expect(signal).toBeDefined();
            expect(signal?.aborted).toBe(false);

            c1.abort('reason-1');
            expect(signal?.aborted).toBe(true);
            if (cleanup) cleanup();
        });

        it('should combine multiple signals and abort instantly if one is already aborted', () => {
            const c1 = new AbortController();
            c1.abort('instant-reason');
            const c2 = new AbortController();

            const { signal } = combine_signals(c1.signal, c2.signal);
            expect(signal?.aborted).toBe(true);
        });
    });

    describe('scrub_secrets and scrub_string', () => {
        it('should redact sensitive keys from objects without modifying the original (deep clone)', () => {
            const original = {
                message: 'Hello',
                api_key: 'sk-proj-1234567890',
                nested: {
                    token: 'Bearer xyz',
                    secureValue: 'safe'
                }
            };

            const scrubbed = scrub_secrets(original) as typeof original;
            expect(scrubbed).not.toBe(original);
            expect(scrubbed.api_key).toBe('[REDACTED]');
            expect(scrubbed.nested.token).toBe('[REDACTED]');
            expect(scrubbed.nested.secureValue).toBe('safe');

            // Verify original was not mutated
            expect(original.api_key).toBe('sk-proj-1234567890');
            expect(original.nested.token).toBe('Bearer xyz');
        });

        it('should not redact keys matching word sub-strings that are safe', () => {
            const original = {
                monkey: 'banana',
                tokenize: 'some-value',
                api_key: 'secret'
            };

            const scrubbed = scrub_secrets(original) as any;
            expect(scrubbed.monkey).toBe('banana');
            expect(scrubbed.tokenize).toBe('some-value');
            expect(scrubbed.api_key).toBe('[REDACTED]');
        });

        it('should redact Bearer and OpenAI keys in strings', () => {
            const str = 'Got error with sk-1234567890abcdef and Authorization Bearer mysecrettoken';
            const scrubbed = scrub_string(str);
            expect(scrubbed).toBe('Got error with [REDACTED] and Authorization Bearer [REDACTED]');
        });

        it('should safely guard against circular references in scrub_secrets', () => {
            const obj: any = { message: 'hello' };
            obj.self = obj;

            const result = scrub_secrets(obj);
            expect(result).toBe('[UNSCRUBBABLE: Circular/Function]');
        });
    });

    describe('truncate_payload', () => {
        it('should not truncate short string', () => {
            expect(truncate_payload('hello', 10)).toBe('hello');
        });

        it('should truncate long string to max limit', () => {
            const longStr = 'a'.repeat(20);
            expect(truncate_payload(longStr, 10)).toBe('aaaaaaaaaa... [TRUNCATED 20 bytes]');
        });
    });

    describe('sanitize_error_detail', () => {
        it('should redact database connection strings', () => {
            const err = 'Failed to connect to postgresql://user:my-pass123@db.internal:5432/main';
            expect(sanitize_error_detail(err)).toBe('Failed to connect to [CONNECTION_STRING_REDACTED]/main');
        });

        it('should redact POSIX and Windows absolute file paths', () => {
            const posixErr = 'Crash in file /var/www/nodes/index.js at execution';
            expect(sanitize_error_detail(posixErr)).toBe('Crash in file [PATH_REDACTED] at execution');

            const winErr = 'Cannot open drive D:\\TadpoleOS-Dev\\src\\utils\\telemetry.ts';
            expect(sanitize_error_detail(winErr)).toBe('Cannot open drive [PATH_REDACTED]');
        });

        it('should strip stack traces', () => {
            const withStack = 'Error: Failed to process\n  at Function.execute (index.js:10:2)\n  at run (main.js:5:1)';
            expect(sanitize_error_detail(withStack)).toBe('Failed to process');
        });
    });
});

describe('BaseApiService class', () => {
    let mockFetch: any;
    let mockTelemetry: any;
    let mockSettings: any;
    let mockTimers: any;
    let service: BaseApiService;

    beforeEach(() => {
        mockFetch = vi.fn().mockResolvedValue({
            ok: true,
            status: 200,
            headers: new Headers({
                'x-request-id': 'req-123',
                'traceparent': '00-trace-span-01'
            }),
            text: async () => JSON.stringify({ ok: true })
        });
        mockTelemetry = {
            addSpan: vi.fn(),
            updateSpan: vi.fn()
        };
        mockSettings = {
            getSettings: vi.fn().mockReturnValue({
                tadpole_os_url: 'http://localhost:8000',
                tadpole_os_api_key: 'my-secret-key'
            })
        };
        mockTimers = {
            setTimeout: (fn: any, _delay: number) => setTimeout(fn, 0) as any,
            clearTimeout: (id: any) => clearTimeout(id)
        };

        service = new BaseApiService({
            httpAdapter: {
                fetch: mockFetch,
                crypto: crypto
            },
            telemetryPort: mockTelemetry,
            settingsPort: mockSettings,
            timers: mockTimers
        });
    });

    it('should generate W3C compliant trace ID and span ID', () => {
        const { headers, context } = service.get_headers();

        expect(headers).toHaveProperty('traceparent');
        expect((headers as any)['Authorization']).toBe('Bearer my-secret-key');
        
        // W3C trace ID should be 32 hex chars (16 bytes)
        expect(context.trace_id).toMatch(/^[0-9a-f]{32}$/);
        // W3C span ID should be 16 hex chars (8 bytes)
        expect(context.span_id).toMatch(/^[0-9a-f]{16}$/);
        // traceparent matches 00-{trace_id}-{span_id}-01
        expect(context.traceparent).toBe(`00-${context.trace_id}-${context.span_id}-01`);
    });

    it('should execute request with telemetry spans tracked', async () => {
        const result = await service.request('/v1/status', { method: 'GET' });
        expect(result).toEqual({ ok: true });

        expect(mockTelemetry.addSpan).toHaveBeenCalledWith(expect.objectContaining({
            name: 'ui_request: /v1/status',
            status: 'running'
        }));

        expect(mockTelemetry.updateSpan).toHaveBeenCalledWith(
            expect.any(String),
            expect.objectContaining({
                status: 'success'
            })
        );
    });

    it('should retry on HTTP 5xx responses for idempotent methods (GET)', async () => {
        mockFetch
            .mockResolvedValueOnce({
                ok: false,
                status: 502,
                statusText: 'Bad Gateway',
                headers: new Headers(),
                text: async () => 'Error'
            })
            .mockResolvedValueOnce({
                ok: true,
                status: 200,
                headers: new Headers(),
                text: async () => JSON.stringify({ success: true })
            });

        const result = await service.request('/v1/retry', { method: 'GET' });
        expect(result).toEqual({ success: true });
        expect(mockFetch).toHaveBeenCalledTimes(2);
    });

    it('should retry on HTTP 5xx responses for PUT/DELETE only if options.idempotent is true', async () => {
        // Attempt 1: PUT with options.idempotent === true
        mockFetch
            .mockResolvedValueOnce({
                ok: false,
                status: 503,
                statusText: 'Service Unavailable',
                headers: new Headers(),
                text: async () => 'Error'
            })
            .mockResolvedValueOnce({
                ok: true,
                status: 200,
                headers: new Headers(),
                text: async () => JSON.stringify({ updated: true })
            });

        const putResult = await service.request('/v1/update', { method: 'PUT', idempotent: true });
        expect(putResult).toEqual({ updated: true });
        expect(mockFetch).toHaveBeenCalledTimes(2);

        // Attempt 2: PUT without options.idempotent (default: no retry)
        mockFetch.mockReset();
        mockFetch.mockResolvedValueOnce({
            ok: false,
            status: 503,
            statusText: 'Service Unavailable',
            headers: new Headers(),
            text: async () => 'Error'
        });

        await expect(service.request('/v1/update', { method: 'PUT' })).rejects.toThrow(ServerError);
        expect(mockFetch).toHaveBeenCalledTimes(1);
    });

    it('should not retry on HTTP 5xx responses for non-idempotent methods (POST)', async () => {
        mockFetch.mockResolvedValueOnce({
            ok: false,
            status: 500,
            statusText: 'Internal Server Error',
            headers: new Headers(),
            text: async () => 'Error'
        });

        await expect(service.request('/v1/create', { method: 'POST' })).rejects.toThrow(ServerError);
        expect(mockFetch).toHaveBeenCalledTimes(1);
    });

    it('should throw typed subclass error instances (e.g. AuthError, ValidationError) and broadcast to subscribers', async () => {
        const errorListener = vi.fn();
        const unsubscribe = subscribe_api_errors(errorListener);

        // 401 -> AuthError
        mockFetch.mockResolvedValueOnce({
            ok: false,
            status: 401,
            statusText: 'Unauthorized',
            headers: new Headers(),
            text: async () => JSON.stringify({
                type: 'unauthorized_error',
                detail: 'Token is invalid'
            })
        });

        await expect(service.request('/v1/check')).rejects.toThrow(AuthError);
        expect(errorListener).toHaveBeenCalledTimes(1);
        expect(errorListener.mock.calls[0][0]).toBeInstanceOf(AuthError);

        // 400 -> ValidationError
        mockFetch.mockResolvedValueOnce({
            ok: false,
            status: 400,
            statusText: 'Bad Request',
            headers: new Headers(),
            text: async () => JSON.stringify({
                type: 'validation_error',
                detail: 'Invalid parameters'
            })
        });

        await expect(service.request('/v1/check')).rejects.toThrow(ValidationError);
        expect(errorListener.mock.calls[1][0]).toBeInstanceOf(ValidationError);

        unsubscribe();
    });
});

describe('createApiService factory function', () => {
    it('should throw if crypto adapter is missing', () => {
        expect(() => createApiService({
            httpAdapter: {
                fetch: fetch,
                crypto: undefined as any
            }
        })).toThrow(/crypto adapter is mandatory/);
    });

    it('should construct BaseApiService correctly with default adapters', () => {
        const customService = createApiService();
        expect(customService).toBeInstanceOf(BaseApiService);
    });
});

describe('Request Interceptor', () => {
    it('should intercept request path and return virtual response', async () => {
        const customService = createApiService({
            settingsPort: {
                getSettings: () => ({ tadpole_os_url: '' })
            }
        });
        const unregister = register_request_interceptor((path) => {
            if (path === '/virtual/test') {
                return Promise.resolve({ data: 'intercepted' });
            }
            return null;
        });

        const result = await customService.request('/virtual/test');
        expect(result).toEqual({ data: 'intercepted' });

        unregister();

        // After unregistering, it should fall through to standard fetch (which will throw immediately since URL is empty)
        await expect(customService.request('/virtual/test')).rejects.toThrow(/Neural Link Configuration Missing/);
    });
});

// Metadata: [base_api_service_test]

// Metadata: [base_api_service_test]
