/**
 * @docs ARCHITECTURE:Services
 * 
 * ### AI Assist Note
 * **Infrastructure Service**: Standardized HTTP client and OpenTelemetry pipeline. 
 * Orchestrates trace propagation (W3C TraceContext), automatic retries with exponential backoff, and Maturity Level 3 HATEOAS error handling.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: 401/403 Auth Failure (invalid bearer token), 408/504 Timeout (exceeding `DEFAULT_TIMEOUT`), or traceparent corruption.
 * - **Telemetry Link**: Every request emits an `X-Request-Id`. Search browser/proxy logs for this ID or `[BaseAPI]`.
 * 
 * @aiContext
 * - **Dependencies**: `settings_store`, `trace_store`.
 * - **Side Effects**: Unified telemetry emission (Add Span/Update Span). Performs automatic retries.
 * - **Mocking**: Mock global `fetch` for unit tests.
 */

import { get_settings } from '../stores/settings_store';
import { use_trace_store } from '../stores/trace_store';
import { is_allowed_origin } from './socket';

export const DEFAULT_TIMEOUT = 30000; // 30 seconds default
export const DEPLOY_TIMEOUT = 7200000; // 2 hours for deployment
export const MAX_RETRIES = 3;
export const INITIAL_RETRY_DELAY = 1000;

export interface SpanData {
    id: string;
    trace_id: string;
    name: string;
    agent_id: string;
    mission_id: string;
    start_time: number;
    status: 'running' | 'success' | 'error';
    attributes: Record<string, string | number | boolean>;
}

export interface TelemetrySpanUpdate {
    end_time: number;
    status: 'success' | 'error';
    attributes?: Record<string, string | number | boolean>;
}

/**
 * Interfaces for Ports & Adapters (Dependency Injection)
 */
export interface HttpClientAdapter {
    fetch: typeof fetch;
    crypto: typeof crypto;
}

export interface TelemetryPort {
    addSpan: (span: SpanData) => void;
    updateSpan: (id: string, updates: TelemetrySpanUpdate) => void;
}

export interface SettingsPort {
    getSettings: () => { tadpole_os_url?: string; tadpole_os_api_key?: string };
}

export interface ApiServiceConfig {
    httpAdapter: HttpClientAdapter;
    telemetryPort: TelemetryPort;
    settingsPort: SettingsPort;
    timers?: {
        setTimeout: typeof setTimeout;
        clearTimeout: typeof clearTimeout;
    };
}

/**
 * Represents a standard RFC 9457 Problem Details error returned by the Sovereign engine.
 */
export class ApiError extends Error {
    public type: string;
    public status: number;
    public error_code: string | null;
    public help_link: string | null;

    constructor(
        message: string,
        type: string,
        status: number,
        error_code: string | null = null,
        help_link: string | null = null
    ) {
        super(message);
        this.type = type;
        this.status = status;
        this.error_code = error_code;
        this.help_link = help_link;
        this.name = 'ApiError';
        // Ensure the prototype is set correctly for stack traces
        Object.setPrototypeOf(this, ApiError.prototype);
    }
}

export class AuthError extends ApiError {
    constructor(message: string, type: string, status: number, error_code: string | null = null, help_link: string | null = null) {
        super(message, type, status, error_code, help_link);
        this.name = 'AuthError';
        Object.setPrototypeOf(this, AuthError.prototype);
    }
}

export class RateLimitError extends ApiError {
    constructor(message: string, type: string, status: number, error_code: string | null = null, help_link: string | null = null) {
        super(message, type, status, error_code, help_link);
        this.name = 'RateLimitError';
        Object.setPrototypeOf(this, RateLimitError.prototype);
    }
}

export class ValidationError extends ApiError {
    constructor(message: string, type: string, status: number, error_code: string | null = null, help_link: string | null = null) {
        super(message, type, status, error_code, help_link);
        this.name = 'ValidationError';
        Object.setPrototypeOf(this, ValidationError.prototype);
    }
}

export class ServerError extends ApiError {
    constructor(message: string, type: string, status: number, error_code: string | null = null, help_link: string | null = null) {
        super(message, type, status, error_code, help_link);
        this.name = 'ServerError';
        Object.setPrototypeOf(this, ServerError.prototype);
    }
}

/**
 * Mappings for Api Errors to subclasses.
 */
export function map_api_error_to_subclass(err: ApiError): ApiError {
    if (err.status === 401 || err.status === 403) {
        return new AuthError(err.message, err.type, err.status, err.error_code, err.help_link);
    }
    if (err.status === 429) {
        return new RateLimitError(err.message, err.type, err.status, err.error_code, err.help_link);
    }
    if (err.status === 400) {
        return new ValidationError(err.message, err.type, err.status, err.error_code, err.help_link);
    }
    if (err.status >= 500) {
        return new ServerError(err.message, err.type, err.status, err.error_code, err.help_link);
    }
    return err;
}

/**
 * @deprecated Use map_api_error_to_subclass instead.
 */
export function map_api_error(err: unknown): never {
    if (err instanceof ApiError) {
        throw map_api_error_to_subclass(err);
    }
    throw err;
}

export function with_timeout(timeout_ms: number = DEFAULT_TIMEOUT): { signal: AbortSignal; clear: () => void } {
    const controller = new AbortController();
    const id = setTimeout(() => controller.abort('TIMEOUT'), timeout_ms);
    return { signal: controller.signal, clear: () => clearTimeout(id) };
}

/**
 * Api Error Event Pub/Sub for Decoupled Side-Effects
 */
type ApiErrorListener = (error: ApiError) => void;
const api_error_listeners = new Set<ApiErrorListener>();

export const subscribe_api_errors = (listener: ApiErrorListener): (() => void) => {
    api_error_listeners.add(listener);
    return () => {
        api_error_listeners.delete(listener);
    };
};

export const emit_api_error = (error: ApiError): void => {
    api_error_listeners.forEach(l => {
        try { l(error); } catch { /* ignore */ }
    });
};

/**
 * Request Interceptor for Virtual Endpoints / Mocking
 */
export type RequestInterceptor = (path: string, options?: unknown) => Promise<unknown> | null;
export const request_interceptors = new Set<RequestInterceptor>();

export function register_request_interceptor(interceptor: RequestInterceptor): () => void {
    request_interceptors.add(interceptor);
    return () => {
        request_interceptors.delete(interceptor);
    };
}

/**
 * Robust AbortSignal Combiner
 */
export function combine_signals(...signals: (AbortSignal | null | undefined)[]): { signal?: AbortSignal; cleanup?: () => void } {
    const active_signals = signals.filter((s): s is AbortSignal => !!s);
    if (active_signals.length === 0) {
        return {};
    }
    if (active_signals.length === 1) {
        return { signal: active_signals[0] };
    }

    if (typeof AbortSignal.any === 'function') {
        return { signal: AbortSignal.any(active_signals) };
    }

    const controller = new AbortController();
    const onAbort = (e: Event) => {
        controller.abort((e.target as AbortSignal).reason);
    };

    for (const signal of active_signals) {
        if (signal.aborted) {
            controller.abort(signal.reason);
            break;
        }
        signal.addEventListener('abort', onAbort);
    }

    const cleanup = () => {
        for (const signal of active_signals) {
            signal.removeEventListener('abort', onAbort);
        }
    };

    return { signal: controller.signal, cleanup };
}

/**
 * SSRF and Sanitization Protection for base URLs
 */
export function validate_and_sanitize_url(url_str: string): string {
    const trimmed = url_str.trim();
    if (!trimmed) {
        throw new Error('URL is empty');
    }

    let parsed: URL;
    try {
        parsed = new URL(trimmed);
    } catch {
        throw new Error(`Invalid URL format: ${trimmed}`);
    }

    // Strip basic auth credentials
    parsed.username = '';
    parsed.password = '';

    const protocol = parsed.protocol.toLowerCase();
    const hostname = parsed.hostname.toLowerCase();

    const clean_hostname = hostname.replace(/^\[|\]$/g, '');
    const is_loopback = 
        clean_hostname === 'localhost' || 
        clean_hostname === '127.0.0.1' || 
        clean_hostname === '::1' ||
        clean_hostname.endsWith('.localhost');

    if (protocol !== 'https:' && !is_loopback) {
        throw new Error(`Insecure transmission blocked: external connection to ${hostname} must use HTTPS.`);
    }

    return parsed.toString().replace(/\/$/, '');
}

/**
 * Truncate payloads for telemetry logs to prevent OOM
 */
export function truncate_payload(data: string, max_length = 1024): string {
    if (data.length <= max_length) {
        return data;
    }
    return `${data.substring(0, max_length)}... [TRUNCATED ${data.length} bytes]`;
}

/**
 * Redaction / Scrubbing logic
 */
export function scrub_string(str: string): string {
    return str
        .replace(/sk-[a-zA-Z0-9-_]{12,}/g, '[REDACTED]')
        .replace(/Bearer\s+[a-zA-Z0-9-_.]+/gi, 'Bearer [REDACTED]');
}

/**
 * Sanitizes backend detail messages to prevent credentials, file paths and stack traces leaks
 */
export function sanitize_error_detail(detail: string): string {
    if (!detail) return detail;
    let sanitized = detail;

    // 1. Connection strings: e.g. postgres://user:pass@host or http://user:pass@host
    sanitized = sanitized.replace(/[a-zA-Z0-9+-.]+:\/\/[^/:\s]+:[^/:\s]+@[^\s/]+/gi, '[CONNECTION_STRING_REDACTED]');

    // 2. Absolute file paths (both POSIX and Windows directories)
    sanitized = sanitized.replace(/(?:\b[a-zA-Z]:\\|\/)(?:[^\\/\s]+[\\/])+[^\s\\/]+/gi, '[PATH_REDACTED]');

    // 3. Strip "Error:" prefix from start
    sanitized = sanitized.replace(/^Error:\s*/i, '');

    // 4. Strip "at " stack trace lines (using multiline flag to match start of any line)
    sanitized = sanitized.replace(/^\s*at\s+[^\r\n]+/gim, '');

    // 5. Scrub any remaining secrets
    sanitized = scrub_string(sanitized);

    return sanitized.trim();
}

export function scrub_secrets(body: unknown): unknown {
    if (body === null || body === undefined) {
        return body;
    }
    try {
        if (typeof body === 'string') {
            try {
                const parsed = JSON.parse(body);
                const cloned = typeof structuredClone !== 'undefined' ? structuredClone(parsed) : JSON.parse(JSON.stringify(parsed));
                const scrubbed = scrub_secrets_object(cloned);
                return JSON.stringify(scrubbed);
            } catch {
                return scrub_string(body);
            }
        }
        if (body instanceof FormData) {
            const scrubbed = new FormData();
            for (const [key, val] of body.entries()) {
                if (/^(key|token|secret|password|auth|bearer)$/i.test(key) || /\b(api_key|authorization|token|apiKey|access_token)\b/i.test(key)) {
                    scrubbed.append(key, '[REDACTED]');
                } else if (typeof val === 'string') {
                    scrubbed.append(key, scrub_string(val));
                } else {
                    scrubbed.append(key, val);
                }
            }
            return scrubbed;
        }
        if (typeof body === 'object') {
            const cloned = typeof structuredClone !== 'undefined' ? structuredClone(body) : JSON.parse(JSON.stringify(body));
            return scrub_secrets_object(cloned);
        }
    } catch {
        return '[UNSCRUBBABLE: Circular/Function]';
    }
    return body;
}

function is_sensitive_key(key: string): boolean {
    const sensitive = /^(key|token|secret|password|auth|authorization|cookie|jwt|bearer)s?$/i;
    if (sensitive.test(key)) {
        return true;
    }
    if (/(?:_|-)(key|token|secret|password|auth|authorization|cookie|jwt|bearer)s?$/i.test(key)) {
        return true;
    }
    if (/^(key|token|secret|password|auth|authorization|cookie|jwt|bearer)s?(?:_|-)/i.test(key)) {
        return true;
    }
    const camelMatch = key.match(/[a-z](Key|Token|Secret|Password|Auth|Authorization|Cookie|Jwt|Bearer)s?$/);
    if (camelMatch) {
        return true;
    }
    return false;
}

function scrub_secrets_object(obj: unknown): unknown {
    if (obj === null || obj === undefined) {
        return obj;
    }
    if (Array.isArray(obj)) {
        return obj.map(item => scrub_secrets_object(item));
    }
    if (typeof obj === 'object') {
        const record = obj as Record<string, unknown>;
        for (const key of Object.keys(record)) {
            const val = record[key];
            if (is_sensitive_key(key)) {
                record[key] = '[REDACTED]';
            } else if (typeof val === 'string') {
                record[key] = scrub_string(val);
            } else if (typeof val === 'object') {
                record[key] = scrub_secrets_object(val);
            }
        }
    }
    return obj;
}

const get_response_header = (response: Response, name: string): string | undefined => {
    return response.headers?.get?.(name) || undefined;
};

const build_trace_attributes = (
    response: Response,
    extra: Record<string, string | number | boolean> = {},
): Record<string, string | number | boolean> => {
    const attributes: Record<string, string | number | boolean> = {
        'http.status_code': response.status,
        ...extra,
    };
    const request_id = get_response_header(response, 'x-request-id');
    const traceparent = get_response_header(response, 'traceparent');
    if (request_id) attributes['resp.x_request_id'] = request_id;
    if (traceparent) attributes['resp.traceparent'] = traceparent;
    return attributes;
};

/**
 * BaseApiService (Implementing Ports & Adapters Pattern)
 */
export class BaseApiService {
    private readonly config: ApiServiceConfig;

    constructor(config: ApiServiceConfig) {
        this.config = config;
    }

    public get_headers(custom_request_id?: string): { 
        headers: HeadersInit; 
        context: { span_id: string; trace_id: string; traceparent: string; request_id: string } 
    } {
        const { httpAdapter, settingsPort } = this.config;
        const settings = settingsPort.getSettings();
        const raw_token = settings.tadpole_os_api_key;
        const token = (raw_token || '').trim();
        if (!token) {
            throw new Error('Tadpole OS API token is missing. Configure NEURAL_TOKEN in Settings before making requests.');
        }

        const request_id = custom_request_id || 
            (typeof httpAdapter.crypto.randomUUID === 'function' 
                ? httpAdapter.crypto.randomUUID() 
                : `tr-${Date.now()}`);

        const trace_id = custom_request_id
            ? custom_request_id.replace(/-/g, '').padEnd(32, '0').slice(0, 32)
            : Array.from(httpAdapter.crypto.getRandomValues(new Uint8Array(16)))
                .map(b => b.toString(16).padStart(2, '0'))
                .join('');

        const span_id = Array.from(httpAdapter.crypto.getRandomValues(new Uint8Array(8)))
            .map(b => b.toString(16).padStart(2, '0'))
            .join('');

        const traceparent = `00-${trace_id}-${span_id}-01`;

        return {
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`,
                'X-Request-Id': request_id,
                'traceparent': traceparent
            },
            context: { span_id, trace_id, traceparent, request_id }
        };
    }

    public async request<T = unknown>(
        path: string,
        options: RequestInit & { response_type?: 'json' | 'blob' | 'text'; timeout?: number; idempotent?: boolean } = {}
    ): Promise<T> {
        for (const interceptor of request_interceptors) {
            const intercepted = interceptor(path, options);
            if (intercepted !== null) {
                return intercepted as Promise<T>;
            }
        }

        const { httpAdapter, telemetryPort, settingsPort } = this.config;
        const setTimeoutFn = this.config.timers?.setTimeout || setTimeout;
        const clearTimeoutFn = this.config.timers?.clearTimeout || clearTimeout;

        const settings = settingsPort.getSettings();
        const tadpole_os_url = settings.tadpole_os_url;
        if (!tadpole_os_url) {
            throw new Error('Neural Link Configuration Missing: tadpole_os_url is undefined.');
        }

        let base_url: string;
        try {
            base_url = validate_and_sanitize_url(tadpole_os_url);
        } catch (e) {
            throw new Error(`Neural Link Configuration Error: ${(e as Error).message}`, { cause: e });
        }

        if (!is_allowed_origin(base_url)) {
            throw new Error(`Connection to origin refused: ${base_url} is not in the allowed origins list.`);
        }

        const clean_path = path.startsWith('/') ? path : `/${path}`;
        const url = `${base_url}${clean_path}`;

        // Setup Timeout controller
        const timeout_ms = options.timeout ?? DEFAULT_TIMEOUT;
        const timeout_controller = new AbortController();
        const timeout_id = setTimeoutFn(() => timeout_controller.abort('TIMEOUT'), timeout_ms);

        const { signal: combined_signal, cleanup: cleanup_signals } = combine_signals(
            options.signal,
            timeout_controller.signal
        );

        const { headers: base_headers, context } = this.get_headers((options.headers as Record<string, string>)?.['X-Request-Id']);

        const is_form_data = options.body instanceof FormData;
        const final_headers = { ...base_headers };
        if (is_form_data) {
            delete (final_headers as Record<string, string>)['Content-Type'];
        }

        const all_headers = { ...final_headers, ...options.headers };
        const req_attributes: Record<string, string | number | boolean> = {};
        if (options.body) {
            const scrubbed = scrub_secrets(options.body);
            const body_str = typeof scrubbed === 'string' ? scrubbed : JSON.stringify(scrubbed);
            req_attributes['http.request.body'] = truncate_payload(body_str);
        }
        const scrubbed_headers = scrub_secrets(all_headers);
        req_attributes['http.request.headers'] = JSON.stringify(scrubbed_headers);

        telemetryPort.addSpan({
            id: context.span_id,
            trace_id: context.trace_id,
            name: `ui_request: ${path.split('?')[0]}`,
            agent_id: 'frontend',
            mission_id: 'system',
            start_time: Date.now(),
            status: 'running',
            attributes: req_attributes
        });

        try {
            const execute_fetch = async (attempt: number): Promise<Response> => {
                let response: Response;
                try {
                    response = await httpAdapter.fetch(url, {
                        ...options,
                        headers: all_headers,
                        signal: combined_signal
                    });
                } catch (err) {
                    const method = (options.method || 'GET').toUpperCase();
                    const is_retryable = method === 'GET' || method === 'HEAD' || (options.idempotent === true && (method === 'PUT' || method === 'DELETE'));
                    const is_timeout = (combined_signal && combined_signal.aborted && combined_signal.reason === 'TIMEOUT') || (err instanceof Error && err.message === 'TIMEOUT');
                    if (is_timeout) {
                        throw new Error(`Request timed out after ${timeout_ms}ms for: ${url}`, { cause: err });
                    }
                    if (!is_retryable || attempt >= MAX_RETRIES || (err instanceof Error && err.name === 'AbortError')) {
                        if (err instanceof TypeError && err.message === 'Failed to fetch') {
                            throw new Error(`Failed to fetch from ${url}. Please ensure the server is running and CORS allows this origin.`, { cause: err });
                        }
                        throw err;
                    }
                    const backoff = INITIAL_RETRY_DELAY * Math.pow(2, attempt);
                    await new Promise(resolve => setTimeoutFn(resolve, backoff));
                    return execute_fetch(attempt + 1);
                }

                if (!response.ok && response.status >= 500) {
                    const method = (options.method || 'GET').toUpperCase();
                    const is_retryable = method === 'GET' || method === 'HEAD' || (options.idempotent === true && (method === 'PUT' || method === 'DELETE'));
                    if (is_retryable && attempt < MAX_RETRIES) {
                        const backoff = INITIAL_RETRY_DELAY * Math.pow(2, attempt);
                        await new Promise(resolve => setTimeoutFn(resolve, backoff));
                        return execute_fetch(attempt + 1);
                    }
                }

                return response;
            };

            const response = await execute_fetch(0);

            if (!response.ok) {
                const error_text = await response.text();
                let error_json: Record<string, unknown> | null = null;
                try { error_json = JSON.parse(error_text); } catch { /* ignore */ }

                const type = (error_json?.type as string) || 'about:blank';
                const title = (error_json?.title as string) || response.statusText;
                const error_code = (error_json?.error_code as string) || null;
                const help_link = (error_json?.help_link as string) || null;
                let detail = (error_json?.detail as string) || (error_json?.message as string) || 'Unknown Infrastructure Error';

                if (response.status === 401) {
                    const is_local = url.includes('127.0.0.1') || url.includes('localhost');
                    detail = is_local 
                        ? 'Unauthorized. Your Neural Token does not match the engine configuration. Please verify the NEURAL_TOKEN in Settings.'
                        : 'Unauthorized. Invalid API token.';
                } else if (response.status === 429) {
                    detail = 'Too many requests. Local security protocols have triggered a temporary cooling-down period. Please wait a moment and try again.';
                }

                const sanitized_detail = sanitize_error_detail(detail);
                const message = `${title}: ${sanitized_detail}`;

                telemetryPort.updateSpan(context.span_id, {
                    end_time: Date.now(),
                    status: 'error',
                    attributes: build_trace_attributes(
                        response,
                        error_code ? { 'error.code': error_code } : {},
                    )
                });

                const base_error = new ApiError(message, type, response.status, error_code, help_link);
                const mapped_error = map_api_error_to_subclass(base_error);
                emit_api_error(mapped_error);
                throw mapped_error;
            }

            let result: unknown;
            if (response.status === 204) {
                result = null;
            } else if (options.response_type === 'blob') {
                result = await response.blob();
            } else if (options.response_type === 'text') {
                result = await response.text();
            } else {
                const text = await response.text();
                result = text ? JSON.parse(text) : null;
            }

            telemetryPort.updateSpan(context.span_id, {
                end_time: Date.now(),
                status: 'success',
                attributes: build_trace_attributes(response)
            });

            return result as T;
        } catch (err) {
            if (err instanceof ApiError) {
                throw err;
            }
            if (err instanceof Error) {
                const sanitized = sanitize_error_detail(err.message);
                try {
                    err.message = sanitized;
                } catch {
                    try {
                        Object.defineProperty(err, 'message', {
                            value: sanitized,
                            configurable: true,
                            writable: true,
                            enumerable: true
                        });
                    } catch {
                        const cloned_err = new Error(sanitized);
                        cloned_err.name = err.name;
                        cloned_err.stack = err.stack;
                        for (const key of Object.keys(err)) {
                            if (key !== 'message') {
                                try { (cloned_err as unknown as Record<string, unknown>)[key] = (err as unknown as Record<string, unknown>)[key]; } catch { /* ignore */ }
                            }
                        }
                        throw cloned_err;
                    }
                }
            }
            throw err;
        } finally {
            clearTimeoutFn(timeout_id);
            cleanup_signals?.();
        }
    }
}

/**
 * Factory function to create a new BaseApiService instance.
 */
export function createApiService(config?: Partial<ApiServiceConfig>): BaseApiService {
    const final_config: ApiServiceConfig = {
        httpAdapter: {
            fetch: config?.httpAdapter?.fetch || ((...args) => fetch(...args)),
            crypto: config?.httpAdapter ? config.httpAdapter.crypto : (typeof crypto !== 'undefined' ? crypto : (globalThis.crypto as unknown as Crypto))
        },
        telemetryPort: config?.telemetryPort || {
            addSpan: (span) => use_trace_store.getState().add_span(span),
            updateSpan: (id, updates) => use_trace_store.getState().update_span(id, updates)
        },
        settingsPort: config?.settingsPort || {
            getSettings: () => get_settings()
        },
        timers: config?.timers || {
            setTimeout: setTimeout,
            clearTimeout: clearTimeout
        }
    };

    if (!final_config.httpAdapter.crypto) {
        throw new Error('HttpClientAdapter: crypto adapter is mandatory.');
    }

    return new BaseApiService(final_config);
}

/**
 * Singleton production instance shared across the entire application.
 * @deprecated Use createApiService() instead for proper dependency injection.
 */
export const base_api_service_instance = new BaseApiService({
    httpAdapter: {
        fetch: (...args) => fetch(...args),
        crypto: typeof crypto !== 'undefined' ? crypto : (globalThis.crypto as unknown as Crypto)
    },
    telemetryPort: {
        addSpan: (span) => use_trace_store.getState().add_span(span),
        updateSpan: (id, updates) => use_trace_store.getState().update_span(id, updates)
    },
    settingsPort: {
        getSettings: () => get_settings()
    },
    timers: {
        setTimeout: setTimeout,
        clearTimeout: clearTimeout
    }
});

/**
 * Backward compatible wrapper function for over 200 callers in the application.
 * @deprecated Use createApiService() or DI instead.
 */
export function api_request<T = unknown>(
    path: string,
    options: RequestInit & { response_type?: 'json' | 'blob' | 'text'; timeout?: number; idempotent?: boolean } = {}
): Promise<T> {
    return base_api_service_instance.request<T>(path, options);
}

/**
 * Backward compatible trace header context generator.
 * @deprecated Use createApiService() or DI instead.
 */
export function get_headers(custom_request_id?: string) {
    return base_api_service_instance.get_headers(custom_request_id);
}

// Metadata: [base_api_service]

// Metadata: [base_api_service]
