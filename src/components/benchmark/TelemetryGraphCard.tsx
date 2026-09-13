/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Benchmark / TelemetryGraphCard
 * - **Primary Entrypoints**: `TelemetryGraphCard`, `TelemetryGraphCardProps`
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
import { motion, AnimatePresence } from 'framer-motion';
import { BarChart3, ArrowRightLeft } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import { DualTracePlayback, type TraceStep } from '../dashboard/Dual_Trace_Playback';
import type { BenchmarkResult, DeltaMetrics } from './types';

export interface TelemetryGraphCardProps {
    isVisible: boolean;
    comparisonData: { t1: BenchmarkResult; t2: BenchmarkResult } | null;
    deltaMetrics: DeltaMetrics | null;
    traceA: TraceStep[];
    traceB: TraceStep[];
    onSwapComparison: () => void;
    onClearSelection: () => void;
    formatMs: (ms?: number) => string;
}

export const TelemetryGraphCard: React.FC<TelemetryGraphCardProps> = ({
    isVisible,
    comparisonData,
    deltaMetrics,
    traceA,
    traceB,
    onSwapComparison,
    onClearSelection,
    formatMs,
}) => {
    return (
        <AnimatePresence>
            {isVisible && comparisonData?.t1 && comparisonData?.t2 && (
                <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -20 }}
                    className="grid grid-cols-1 md:grid-cols-3 gap-6 p-6 rounded-2xl bg-[color:var(--color-surface)]/50 border border-[color:var(--color-border)] backdrop-blur-xl relative overflow-hidden"
                >
                    <div className="absolute top-0 right-0 p-8 opacity-5 pointer-events-none">
                        <ArrowRightLeft size={120} />
                    </div>

                    <div className="col-span-full mb-4 flex items-center justify-between border-b border-[color:var(--color-border)] pb-4">
                        <Tooltip content={i18n.t('benchmark.tooltip_delta')} position="bottom">
                            <h2 className="text-lg font-semibold flex items-center gap-2 cursor-help">
                                <BarChart3 className="text-green-400" size={20} />
                                {i18n.t('benchmark.label_delta_analysis')}
                            </h2>
                        </Tooltip>
                        <div className="flex items-center gap-3">
                            <button
                                onClick={onSwapComparison}
                                className="text-xs text-emerald-400 hover:text-emerald-300 flex items-center gap-1 transition-colors cursor-pointer bg-emerald-500/10 border border-emerald-500/20 px-2.5 py-1 rounded-lg font-mono uppercase tracking-tighter"
                                title="Swap baseline and target comparison runs"
                            >
                                <ArrowRightLeft size={12} />
                                <span>Swap</span>
                            </button>
                            <button
                                onClick={onClearSelection}
                                className="text-xs text-zinc-500 hover:text-white transition-colors cursor-pointer"
                            >
                                {i18n.t('benchmark.btn_clear')}
                            </button>
                        </div>
                    </div>

                    <div className="space-y-4">
                        <div className="text-xs uppercase tracking-widest text-zinc-500 font-bold">{i18n.t('benchmark.label_baseline')}</div>
                        <div className="p-4 rounded-xl bg-[color:var(--color-background)] border border-[color:var(--color-border)]">
                            <div className="text-sm font-semibold truncate">{comparisonData.t2.name}</div>
                            <div className="text-2xl font-mono mt-1">{formatMs(comparisonData.t2.mean_ms)}</div>
                        </div>
                    </div>

                    <div className="flex flex-col items-center justify-center space-y-2">
                        <div className="text-xs uppercase tracking-widest text-zinc-500 font-bold">{i18n.t('benchmark.label_variance')}</div>
                        {deltaMetrics && (
                            <div className={`text-3xl font-mono ${deltaMetrics.isImprovement ? 'text-emerald-400' : 'text-rose-400'}`}>
                                {deltaMetrics.isImprovement ? '-' : '+'}{deltaMetrics.percentage}%
                            </div>
                        )}
                        <div className="text-[10px] text-zinc-600 uppercase font-mono tracking-tighter text-center">
                            {i18n.t('benchmark.label_latency_delta')}
                        </div>
                    </div>

                    <div className="space-y-4">
                        <div className="text-xs uppercase tracking-widest text-zinc-500 font-bold">{i18n.t('benchmark.label_current_target')}</div>
                        <div className="p-4 rounded-xl bg-green-500/10 border border-green-500/20">
                            <div className="text-sm font-semibold truncate">{comparisonData.t1.name}</div>
                            <div className="text-2xl font-mono mt-1">{formatMs(comparisonData.t1.mean_ms)}</div>
                        </div>
                    </div>

                    {/* Schema Harness Inspired Dual-Trace Replay Visualizer */}
                    <div className="col-span-full mt-4 border-t border-zinc-800/80 pt-4">
                        <DualTracePlayback
                            titleA={`Baseline (${comparisonData.t2.name})`}
                            titleB={`Optimized (${comparisonData.t1.name})`}
                            traceA={traceA}
                            traceB={traceB}
                        />
                    </div>
                </motion.div>
            )}
        </AnimatePresence>
    );
};

export default TelemetryGraphCard;
