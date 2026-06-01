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
import type { Benchmark_Record } from '../system_api_types';

export const benchmarks_api = {
    get_benchmarks: async (): Promise<Benchmark_Record[]> => {
        return api_request<Benchmark_Record[]>('/v1/benchmarks', { method: 'GET' });
    },

    run_benchmark: async (test_id: string): Promise<Benchmark_Record> => {
        return api_request<Benchmark_Record>(`/v1/benchmarks/run/${test_id}`, { method: 'POST' });
    }
};

// Metadata: [benchmarks_api]
