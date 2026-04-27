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

export const DEFAULT_TIMEOUT = 15000; // 15 seconds default
export const DEPLOY_TIMEOUT = 7200000; // 2 hours for deployment
export const MAX_RETRIES = 3;
export const INITIAL_RETRY_DELAY = 1000;

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

export function with_timeout(timeout_ms: number = DEFAULT_TIMEOUT): { signal: AbortSignal; clear: () => void } {
    const controller = new AbortController();
    const id = setTimeout(() => controller.abort('TIMEOUT'), timeout_ms);
    return { signal: controller.signal, clear: () => clearTimeout(id) };
}

function require_api_token(): string {
    const { tadpole_os_api_key } = get_settings();
    const token = tadpole_os_api_key.trim();
    if (!token) {
        throw new Error('Tadpole OS API token is missing. Configure NEURAL_TOKEN in Settings before making requests.');
    }
    return token;
}

/**
 * Generates standardized headers and context for distributed tracing.
 * 
 * ### 🔦 Observability: W3C TraceContext
 * Generates a unique `trace_id` (32 hex chars) and `span_id` (16 hex chars). 
 * These are combined into a `traceparent` header (version-00) to ensure 
 * that the front-end request is correctly recognized as the "Parent" of any 
 * subsequent Rust/Sidecar spans (OTEL-01).
 */
export function get_headers(custom_request_id?: string): { headers: HeadersInit, context: { span_id: string, trace_id: string, traceparent: string, request_id: string } } {
    const token = require_api_token();

    const request_id = custom_request_id || ((typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : `tr-${Date.now()}`);
    const trace_id = request_id.replace(/-/g, '').padEnd(32, '0').slice(0, 32);

    // Generate a random span ID for the frontend request (16 hex chars)
    let span_id: string;
    if (typeof crypto !== 'undefined' && crypto.getRandomValues) {
        span_id = Array.from(crypto.getRandomValues(new Uint8Array(8)))
            .map(b => b.toString(16).padStart(2, '0'))
            .join('');
    } else {
        span_id = Math.random().toString(16).slice(2, 10) + Math.random().toString(16).slice(2, 10);
    }

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

export async function api_request<T = unknown>(
    path: string,
    options: RequestInit & { response_type?: 'json' | 'blob' | 'text'; timeout?: number } = {}
): Promise<T> {
    const { tadpole_os_url } = get_settings();

    if (!tadpole_os_url) {
        throw new Error('Neural Link Configuration Missing: tadpole_os_url is undefined.');
    }

    const base_url = tadpole_os_url.replace(/\/$/, '');
    const clean_path = path.startsWith('/') ? path : `/${path}`;
    const url = `${base_url}${clean_path}`;
    const { signal, clear } = with_timeout(options.timeout);

    const { headers: base_headers, context } = get_headers((options.headers as Record<string, string>)?.['X-Request-Id']);

    // Multipart/FormData support: if the body is FormData, we must Let the browser set the Content-Type (with boundary)
    const is_form_data = options.body instanceof FormData;
    const final_headers = { ...base_headers };
    if (is_form_data) {
        delete (final_headers as Record<string, string>)['Content-Type'];
    }

    // Emit start span
    use_trace_store.getState().add_span({
        id: context.span_id,
        trace_id: context.trace_id,
        name: `ui_request: ${path.split('?')[0]}`,
        agent_id: 'frontend',
        mission_id: 'system',
        start_time: Date.now(),
        status: 'running',
        attributes: {}
    });

    try {
        const execute_fetch = async (attempt: number): Promise<Response> => {
            try {
                const res = await fetch(url, {
                    ...options,
                    headers: {
                        ...final_headers,
                        ...options.headers
                    },
                    signal: options.signal || signal
                });
                return res;
            } catch (err) {
                // Only retry safe, idempotent methods. POST/PUT/DELETE/PATCH
                // must NOT be retried to avoid duplicate side effects.
                const method = (options.method || 'GET').toUpperCase();
                const is_retryable = method === 'GET' || method === 'HEAD';
                if (!is_retryable || attempt >= MAX_RETRIES || (err instanceof Error && err.name === 'AbortError')) {
                    throw err;
                }
                const backoff = INITIAL_RETRY_DELAY * Math.pow(2, attempt);
                await new Promise(resolve => setTimeout(resolve, backoff));
                return execute_fetch(attempt + 1);
            }
        };

        const response = await execute_fetch(0);

        if (!response.ok) {
            const error_text = await response.text();
            let error_json: Record<string, unknown> | null = null;
            try { error_json = JSON.parse(error_text); } catch { /* ignore */ }

            // ### 🛡️ Defense: RFC 9457 Problem Details
            // The engine returns structured error objects. We map these into 
            // a standardized `ApiError` to ensure the UI can provide 
            // actionable "help_links" or "error_codes" to the user (ERR-01).
            const type = (error_json?.type as string) || 'about:blank';
            const title = (error_json?.title as string) || response.statusText;
            const error_code = (error_json?.error_code as string) || null;
            const help_link = (error_json?.help_link as string) || null;
            let detail = (error_json?.detail as string) || (error_json?.message as string) || 'Unknown Infrastructure Error';
            
            if (response.status === 429) {
                detail = 'Too many requests. Local security protocols have triggered a temporary cooling-down period. Please wait a moment and try again.';
            }

            const message = `${title}: ${detail}`;

            use_trace_store.getState().update_span(context.span_id, {
                end_time: Date.now(),
                status: 'error',
                attributes: { 
                    'http.status_code': response.status, 
                    ...(error_code ? { 'error.code': error_code } : {}) 
                }
            });


            const error = new ApiError(message, type, response.status, error_code, help_link);
            throw error;
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

        use_trace_store.getState().update_span(context.span_id, {
            end_time: Date.now(),
            status: 'success',
            attributes: { 'http.status_code': response.status }
        });

        clear();
        return result as T;
    } catch (err) {
        clear();
        throw err;
    }
}


// Metadata: [base_api_service]

// Metadata: [base_api_service]
