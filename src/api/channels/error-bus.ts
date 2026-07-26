/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[error_bus]` in observability traces.
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

// Metadata: [error_bus]
