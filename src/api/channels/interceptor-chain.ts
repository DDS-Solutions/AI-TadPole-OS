/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[interceptor_chain]` in observability traces.
 */

import type { RequestInterceptor, RequestInterceptorIterable } from '../types/service-config';

export class InterceptorChain {
    private readonly interceptors = new Set<RequestInterceptor>();

    constructor(initialInterceptors?: Set<RequestInterceptor> | RequestInterceptorIterable | RequestInterceptor[]) {
        if (initialInterceptors) {
            initialInterceptors.forEach(i => this.interceptors.add(i));
        }
    }

    public register(interceptor: RequestInterceptor): () => void {
        this.interceptors.add(interceptor);
        return () => {
            this.interceptors.delete(interceptor);
        };
    }

    public async run<T>(path: string, options?: unknown): Promise<T | null> {
        for (const interceptor of this.interceptors) {
            const result = interceptor(path, options);
            if (result !== null) {
                return result as Promise<T>;
            }
        }
        return null;
    }

    public clear(): void {
        this.interceptors.clear();
    }

    public get size(): number {
        return this.interceptors.size;
    }

    // Allows exposing the internal set for legacy compatibility tests
    public add(interceptor: RequestInterceptor): void {
        this.interceptors.add(interceptor);
    }

    public delete(interceptor: RequestInterceptor): boolean {
        return this.interceptors.delete(interceptor);
    }

    public forEach(callback: (value: RequestInterceptor) => void): void {
        this.interceptors.forEach(callback);
    }

    public [Symbol.iterator](): Iterator<RequestInterceptor> {
        return this.interceptors[Symbol.iterator]();
    }
}

// Metadata: [interceptor_chain]
