/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Singleton production instance shared across the entire application.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[legacy]` in observability traces.
 */

import { get_settings } from '@/stores/settings_store';
import { use_trace_store } from '@/stores/trace_store';
import { BaseApiService } from './service';
import { api_error_listeners, request_interceptors } from './channels';
import type { RequestOptions } from './types';
import { resolveCrypto } from './factory';

/**
 * Singleton production instance shared across the entire application.
 * @deprecated Use createApiService() instead for proper dependency injection.
 */
export const base_api_service_instance = new BaseApiService({
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

// Metadata: [legacy]
