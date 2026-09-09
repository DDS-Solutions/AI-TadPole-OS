/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / benchmarks_api
 * - **Primary Entrypoints**: `benchmarks_api`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
