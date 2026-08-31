/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / factory
 * - **Primary Entrypoints**: `resolveCrypto`, `createApiService`
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
import type { ApiServiceConfig } from './types';
import { BaseApiService } from './service';
import { api_error_listeners, request_interceptors } from './channels';

export function resolveCrypto(override?: Crypto): Crypto {
    if (override) return override;
    if (typeof crypto !== 'undefined' && typeof crypto.getRandomValues === 'function') {
        return crypto;
    }
    if (typeof globalThis !== 'undefined' && globalThis.crypto && typeof globalThis.crypto.getRandomValues === 'function') {
        return globalThis.crypto as unknown as Crypto;
    }
    throw new Error(
        'HttpClientAdapter: crypto.getRandomValues is unavailable in this runtime. ' +
        'Provide an explicit httpAdapter.crypto (e.g., from `import { webcrypto } from "node:crypto"` in Node, ' +
        'or `globalThis.crypto` in a secure-context browser).'
    );
}

/**
 * Factory function to create a new BaseApiService instance.
 */
export function createApiService(config?: Partial<ApiServiceConfig>): BaseApiService {
    const hasHttpAdapter = !!config?.httpAdapter;
    const final_config: ApiServiceConfig = {
        httpAdapter: {
            fetch: config?.httpAdapter?.fetch ?? ((...args) => fetch(...args)),
            crypto: hasHttpAdapter ? config.httpAdapter!.crypto : resolveCrypto()
        },
        telemetryPort: config?.telemetryPort ?? {
            addSpan: (span) => use_trace_store.getState().add_span(span),
            updateSpan: (id, updates) => use_trace_store.getState().update_span(id, updates)
        },
        settingsPort: config?.settingsPort ?? {
            getSettings: () => get_settings()
        },
        timers: config?.timers ?? {
            setTimeout: setTimeout,
            clearTimeout: clearTimeout
        },
        errorListeners: config?.errorListeners ?? api_error_listeners,
        requestInterceptors: config?.requestInterceptors ?? request_interceptors
    };

    if (!final_config.httpAdapter.crypto) {
        throw new Error('HttpClientAdapter: crypto adapter is mandatory.');
    }

    return new BaseApiService(final_config);
}
