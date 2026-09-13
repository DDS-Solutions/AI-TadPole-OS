/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / error-bus
 * - **Primary Entrypoints**: `ErrorBus`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import type { ApiErrorListener, ApiErrorListenerIterable } from '../types/service-config';
import type { ApiError } from '../errors/api-error';

export class ErrorBus {
    private readonly listeners = new Set<ApiErrorListener>();

    constructor(initialListeners?: Set<ApiErrorListener> | ApiErrorListenerIterable | ApiErrorListener[]) {
        if (initialListeners) {
            initialListeners.forEach(l => this.listeners.add(l));
        }
    }

    public subscribe(listener: ApiErrorListener): () => void {
        this.listeners.add(listener);
        return () => {
            this.listeners.delete(listener);
        };
    }

    public emit(error: ApiError): void {
        this.listeners.forEach(l => {
            try { l(error); } catch { /* ignore */ }
        });
    }

    public clear(): void {
        this.listeners.clear();
    }

    public get size(): number {
        return this.listeners.size;
    }

    // Allows exposing the internal set for legacy compatibility tests
    public add(listener: ApiErrorListener): void {
        this.listeners.add(listener);
    }

    public delete(listener: ApiErrorListener): boolean {
        return this.listeners.delete(listener);
    }

    public forEach(callback: (value: ApiErrorListener) => void): void {
        this.listeners.forEach(callback);
    }

    public [Symbol.iterator](): Iterator<ApiErrorListener> {
        return this.listeners[Symbol.iterator]();
    }
}
