/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[index]` in observability traces.
 */

import { ErrorBus } from './error-bus';
import { InterceptorChain } from './interceptor-chain';
import type { ApiErrorListener, RequestInterceptor } from '../types/service-config';
import type { ApiError } from '../errors/api-error';

export * from './error-bus';
export * from './interceptor-chain';

// Legacy compatibility singletons (module-level)
export const api_error_listeners = new ErrorBus();
export const request_interceptors = new InterceptorChain();

export const subscribe_api_errors = (listener: ApiErrorListener): (() => void) => {
    return api_error_listeners.subscribe(listener);
};

export const emit_api_error = (error: ApiError): void => {
    api_error_listeners.emit(error);
};

export function register_request_interceptor(interceptor: RequestInterceptor): () => void {
    return request_interceptors.register(interceptor);
}

// Metadata: [index]
