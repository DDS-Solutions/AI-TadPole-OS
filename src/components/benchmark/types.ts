/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Benchmark / types
 * - **Primary Entrypoints**: `RUNNER_BUTTONS`, `BenchmarkResult`, `RunnerButtonConfig`, `DeltaMetrics`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import React from 'react';
import { Zap, BarChart3, ArrowRightLeft } from 'lucide-react';

export interface BenchmarkResult {
    id: string;
    name: string;
    category: string;
    test_id: string;
    mean_ms: number;
    p95_ms?: number;
    p99_ms?: number;
    target_value?: string;
    status: string;
    metadata?: string;
    created_at: string;
}

export interface RunnerButtonConfig {
    id: string;
    tooltipKey: string;
    labelKey: string;
    color: string;
    Icon: React.ComponentType<{ size?: number; className?: string }>;
}

export const RUNNER_BUTTONS: readonly RunnerButtonConfig[] = [
    { id: 'BM-RUN-01', tooltipKey: 'benchmark.tooltip_runner', labelKey: 'benchmark.btn_run_runner', color: 'bg-green-600 hover:bg-green-500 shadow-green-500/20', Icon: Zap },
    { id: 'BM-DB-01', tooltipKey: 'benchmark.tooltip_db', labelKey: 'benchmark.btn_run_db', color: 'bg-green-600 hover:bg-green-500 shadow-green-500/20', Icon: BarChart3 },
    { id: 'BM-RL-01', tooltipKey: 'benchmark.tooltip_rl', labelKey: 'benchmark.btn_run_rl', color: 'bg-cyan-600 hover:bg-cyan-500 shadow-cyan-500/20', Icon: ArrowRightLeft },
] as const;

export interface DeltaMetrics {
    value: string;
    percentage: number;
    isImprovement: boolean;
}
