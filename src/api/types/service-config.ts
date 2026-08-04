/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[service_config]` in observability traces.
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

// Metadata: [service_config]
