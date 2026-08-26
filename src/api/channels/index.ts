/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / index
 * - **Primary Entrypoints**: `register_request_interceptor`, `api_error_listeners`, `request_interceptors`, `subscribe_api_errors`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
