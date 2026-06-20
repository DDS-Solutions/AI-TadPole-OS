/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[base_api_service]` in observability traces.
 */

import { is_allowed_origin } from '@/services/socket';
import type { ApiServiceConfig, ApiErrorListener, RequestInterceptor, RequestOptions } from '../types';
import { ApiError, map_api_error_to_subclass } from '../errors';
import {
    validate_and_sanitize_url,
    combine_signals,
    delay_with_signal,
    isFormData,
    scrub_secrets,
    truncate_payload,
    sanitize_error_detail
} from '../utils';
import { mint_headers, build_trace_attributes } from '../trace';
import { DEFAULT_TIMEOUT, MAX_RETRIES, INITIAL_RETRY_DELAY } from '../constants';
import { ErrorBus, InterceptorChain } from '../channels';
import { api_error_listeners, request_interceptors } from '../channels';

export class BaseApiService {
    private readonly config: ApiServiceConfig;
    private readonly error_listeners: ErrorBus;
    private readonly request_interceptors: InterceptorChain;

    constructor(config: ApiServiceConfig) {
        this.config = config;
        this.error_listeners = config.errorListeners 
            ? (config.errorListeners instanceof ErrorBus ? config.errorListeners : new ErrorBus(config.errorListeners)) 
            : api_error_listeners;
        this.request_interceptors = config.requestInterceptors 
            ? (config.requestInterceptors instanceof InterceptorChain ? config.requestInterceptors : new InterceptorChain(config.requestInterceptors)) 
            : request_interceptors;
    }

    public subscribe_api_errors(listener: ApiErrorListener): () => void {
        return this.error_listeners.subscribe(listener);
    }

    public register_request_interceptor(interceptor: RequestInterceptor): () => void {
        return this.request_interceptors.register(interceptor);
    }

    public emit_api_error(error: ApiError): void {
        this.error_listeners.emit(error);
    }

    public get_headers(custom_request_id?: string, pre_fetched_token?: string): { 
        headers: Record<string, string>; 
        context: { span_id: string; trace_id: string; traceparent: string; request_id: string } 
    } {
        const { httpAdapter, settingsPort } = this.config;
        const token = (pre_fetched_token !== undefined ? pre_fetched_token : (settingsPort.getSettings().tadpole_os_api_key || '')).trim();
        return mint_headers(httpAdapter.crypto, token, custom_request_id);
    }

    public async request<T = unknown>(
        path: string,
        options: RequestOptions = {}
    ): Promise<T> {
        const intercepted = await this.request_interceptors.run<T>(path, options);
        if (intercepted !== null) {
            return intercepted;
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

        const { headers: base_headers, context } = this.get_headers(
            (options.headers as Record<string, string>)?.['X-Request-Id'],
            settings.tadpole_os_api_key
        );

        const is_form_data = isFormData(options.body);
        const final_headers = { ...base_headers };
        if (is_form_data) {
            delete (final_headers as Record<string, string>)['Content-Type'];
        }

        const all_headers = { ...final_headers, ...options.headers };
        const req_attributes: Record<string, string | number | boolean> = {};
        if (options.body) {
            const scrubbed = scrub_secrets(options.body);
            let body_str: string;
            if (isFormData(scrubbed)) {
                const obj: Record<string, unknown> = {};
                for (const [key, val] of scrubbed.entries()) {
                    obj[key] = val;
                }
                body_str = JSON.stringify(obj);
            } else {
                body_str = typeof scrubbed === 'string' ? scrubbed : JSON.stringify(scrubbed);
            }
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
                    const delay = INITIAL_RETRY_DELAY * Math.pow(2, attempt);
                    const backoff = Math.random() * delay;
                    await delay_with_signal(backoff, combined_signal, setTimeoutFn);
                    return execute_fetch(attempt + 1);
                }

                if (!response.ok && response.status >= 500) {
                    const method = (options.method || 'GET').toUpperCase();
                    const is_retryable = method === 'GET' || method === 'HEAD' || (options.idempotent === true && (method === 'PUT' || method === 'DELETE'));
                    if (is_retryable && attempt < MAX_RETRIES) {
                        const delay = INITIAL_RETRY_DELAY * Math.pow(2, attempt);
                        const backoff = Math.random() * delay;
                        await delay_with_signal(backoff, combined_signal, setTimeoutFn);
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

                const sanitized_title = sanitize_error_detail(title);
                const sanitized_detail = sanitize_error_detail(detail);
                const message = `${sanitized_title}: ${sanitized_detail}`;

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
                this.emit_api_error(mapped_error);
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

// Metadata: [base_api_service]
