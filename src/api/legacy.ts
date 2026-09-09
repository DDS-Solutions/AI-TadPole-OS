/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / legacy
 * - **Primary Entrypoints**: `api_request`, `get_headers`, `base_api_service_instance`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { get_settings } from '@/stores/settings_store';
import { use_trace_store } from '@/stores/trace_store';
import { BaseApiService } from './service';
import { api_error_listeners, request_interceptors } from './channels';
import type { RequestOptions } from './types';
import { resolveCrypto } from './factory';

let _base_api_instance: BaseApiService | null = null;

export function get_base_api_service(): BaseApiService {
    if (!_base_api_instance) {
        _base_api_instance = new BaseApiService({
            httpAdapter: {
                fetch: (...args) => fetch(...args),
                crypto: resolveCrypto()
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
            },
            errorListeners: api_error_listeners,
            requestInterceptors: request_interceptors
        });
    }
    return _base_api_instance;
}

/**
 * Lazy singleton proxy shared across legacy callers.
 * Defers resolveCrypto() until first access to avoid import-time crashes in non-secure contexts.
 * @deprecated Use createApiService() instead for proper dependency injection.
 */
export const base_api_service_instance = new Proxy({} as BaseApiService, {
    get(_target, prop, receiver) {
        const instance = get_base_api_service();
        const value = Reflect.get(instance, prop, receiver);
        return typeof value === 'function' ? value.bind(instance) : value;
    }
});

/**
 * Backward compatible wrapper function for over 200 callers in the application.
 * @deprecated Use createApiService() or DI instead.
 */
export function api_request<T = unknown>(
    path: string,
    options: RequestOptions = {}
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
