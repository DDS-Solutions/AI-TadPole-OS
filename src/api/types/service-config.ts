/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / service-config
 * - **Primary Entrypoints**: `ApiErrorListenerIterable`, `RequestInterceptorIterable`, `ApiServiceConfig`, `RequestOptions`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import type { HttpClientAdapter, TelemetryPort, SettingsPort } from './ports';
import type { ApiError } from '../errors/api-error';

export type ApiErrorListener = (error: ApiError) => void;
export type RequestInterceptor = (path: string, options?: unknown) => Promise<unknown> | null;

export interface ApiErrorListenerIterable {
    add(listener: ApiErrorListener): void;
    delete(listener: ApiErrorListener): boolean;
    forEach(callback: (value: ApiErrorListener) => void): void;
    [Symbol.iterator](): Iterator<ApiErrorListener>;
    size: number;
}

export interface RequestInterceptorIterable {
    add(interceptor: RequestInterceptor): void;
    delete(interceptor: RequestInterceptor): boolean;
    forEach(callback: (value: RequestInterceptor) => void): void;
    [Symbol.iterator](): Iterator<RequestInterceptor>;
    size: number;
}

export interface ApiServiceConfig {
    httpAdapter: HttpClientAdapter;
    telemetryPort: TelemetryPort;
    settingsPort: SettingsPort;
    timers?: {
        setTimeout: typeof setTimeout;
        clearTimeout: typeof clearTimeout;
    };
    errorListeners?: Set<ApiErrorListener> | ApiErrorListenerIterable;
    requestInterceptors?: Set<RequestInterceptor> | RequestInterceptorIterable;
}

export interface RequestOptions extends RequestInit {
    response_type?: 'json' | 'blob' | 'text';
    timeout?: number;
    idempotent?: boolean;
}
