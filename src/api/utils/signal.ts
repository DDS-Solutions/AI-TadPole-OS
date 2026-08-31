/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / signal
 * - **Primary Entrypoints**: `with_timeout`, `delay_with_signal`, `combine_signals`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { DEFAULT_TIMEOUT } from '../constants';

export function with_timeout(timeout_ms: number = DEFAULT_TIMEOUT): { signal: AbortSignal; clear: () => void } {
    const controller = new AbortController();
    const id = setTimeout(() => controller.abort('TIMEOUT'), timeout_ms);
    return { signal: controller.signal, clear: () => clearTimeout(id) };
}

export function delay_with_signal(ms: number, signal?: AbortSignal, setTimeoutFn = setTimeout): Promise<void> {
    return new Promise<void>((resolve, reject) => {
        if (signal?.aborted) {
            return reject(signal.reason || new Error('Aborted'));
        }
        const on_abort = () => {
            clearTimeout(timer);
            signal?.removeEventListener('abort', on_abort);
            reject(signal?.reason || new Error('Aborted'));
        };
        const timer = setTimeoutFn(() => {
            signal?.removeEventListener('abort', on_abort);
            resolve();
        }, ms);
        signal?.addEventListener('abort', on_abort);
    });
}

export function combine_signals(...signals: (AbortSignal | null | undefined)[]): { signal?: AbortSignal; cleanup?: () => void } {
    const active_signals = signals.filter((s): s is AbortSignal => !!s);
    if (active_signals.length === 0) {
        return {};
    }
    if (active_signals.length === 1) {
        return { signal: active_signals[0] };
    }

    if (typeof AbortSignal.any === 'function') {
        return { signal: AbortSignal.any(active_signals) };
    }

    const controller = new AbortController();
    const onAbort = (e: Event) => {
        controller.abort((e.target as AbortSignal).reason);
    };

    for (const signal of active_signals) {
        if (signal.aborted) {
            controller.abort(signal.reason);
            break;
        }
        signal.addEventListener('abort', onAbort);
    }

    const cleanup = () => {
        for (const signal of active_signals) {
            signal.removeEventListener('abort', onAbort);
        }
    };

    return { signal: controller.signal, cleanup };
}
