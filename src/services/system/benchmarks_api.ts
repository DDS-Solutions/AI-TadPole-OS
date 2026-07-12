/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[benchmarks_api]` in observability traces.
 */

import { api_request } from '../base_api_service';
import type { Benchmark_Record, RequestOptions } from '../system_api_types';

const sanitize_id = (id: string): string => {
    if (!id || !/^[a-zA-Z0-9_.-]{1,64}$/.test(id)) {
        throw new Error(`Invalid identifier: ${id}`);
    }
    return encodeURIComponent(id);
};

export const benchmarks_api = {
    get_benchmarks: async (options?: RequestOptions): Promise<Benchmark_Record[]> => {
        return api_request<Benchmark_Record[]>('/v1/benchmarks', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    run_benchmark: async (test_id: string, options?: RequestOptions): Promise<Benchmark_Record> => {
        const clean_test_id = sanitize_id(test_id);
        return api_request<Benchmark_Record>(`/v1/benchmarks/run/${clean_test_id}`, { 
            method: 'POST',
            signal: options?.signal,
            timeout: options?.timeout
        });
    }
};

// Metadata: [benchmarks_api]

