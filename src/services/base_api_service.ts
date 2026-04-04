import { get_settings } from '../stores/settings_store';
import { use_trace_store } from '../stores/trace_store';

export const DEFAULT_TIMEOUT = 15000; // 15 seconds default
export const DEPLOY_TIMEOUT = 7200000; // 2 hours for deployment
export const MAX_RETRIES = 3;
export const INITIAL_RETRY_DELAY = 1000;

export function with_timeout(timeout_ms: number = DEFAULT_TIMEOUT): { signal: AbortSignal; clear: () => void } {
    const controller = new AbortController();
    const id = setTimeout(() => controller.abort('TIMEOUT'), timeout_ms);
    return { signal: controller.signal, clear: () => clearTimeout(id) };
}

export function get_headers(custom_request_id?: string): { headers: HeadersInit, context: { span_id: string, trace_id: string, traceparent: string, request_id: string } } {
    const { tadpole_os_api_key } = get_settings();
    const token = tadpole_os_api_key || 'tadpole-dev-token-2026';

    const request_id = custom_request_id || ((typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : `tr-${Date.now()}`);
    const trace_id = request_id.replace(/-/g, '').padEnd(32, '0').slice(0, 32);

    // Generate a random span ID for the frontend request (16 hex chars)
    let span_id = '';
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
                if (attempt >= MAX_RETRIES || (err instanceof Error && err.name === 'AbortError')) {
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

            const type = (error_json?.type as string) || 'about:blank';
            const title = (error_json?.title as string) || response.statusText;
            const detail = (error_json?.detail as string) || (error_json?.message as string) || 'Unknown Infrastructure Error';
            const message = `${title}: ${detail}`;

            use_trace_store.getState().update_span(context.span_id, {
                end_time: Date.now(),
                status: 'error',
                attributes: { 'http.status_code': response.status }
            });

            const error = new Error(message) as Error & { type?: string; status?: number };
            error.type = type;
            error.status = response.status;
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
