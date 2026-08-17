/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Benchmark Header Card**: Renders top-level navigation, titles, and live runner action controls.
 * Provides rapid execution triggers for hardware, database, and reinforcement learning benchmarks.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Benchmark triggering failure or missing active runner ID.
 * - **Telemetry Link**: Search `[BenchmarkHeaderCard]` in tracing logs.
 */

import React from 'react';
import { NavLink } from 'react-router-dom';
import {
    BarChart3,
    LineChart,
    TrendingUp,
    Loader2,
} from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import { RUNNER_BUTTONS } from './types';

export interface BenchmarkHeaderCardProps {
    isAnalyticsView: boolean;
    runningId: string | null;
    onRunBenchmark: (testId: string) => void;
}

export const BenchmarkHeaderCard: React.FC<BenchmarkHeaderCardProps> = ({
    isAnalyticsView,
    runningId,
    onRunBenchmark,
}) => {
    return (
        <div className="space-y-6">
            {/* Sub-Tab Navigation Header */}
            <div className="flex items-center gap-2 border-b border-[color:var(--color-border)] pb-3">
                <Tooltip content="Hardware throughput & latency benchmarks table" position="bottom">
                    <NavLink
                        to="/benchmarks"
                        className={({ isActive }) =>
                            `px-3 py-1.5 rounded-lg text-xs font-bold transition-all flex items-center gap-1.5 ${
                                isActive && !isAnalyticsView
                                    ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/40 shadow-sm'
                                    : 'text-zinc-400 hover:text-zinc-200 border border-transparent'
                            }`
                        }
                    >
                        <BarChart3 size={14} /> Latency Benchmarks
                    </NavLink>
                </Tooltip>
                <Tooltip content="Side-by-side telemetry trace playback inspector" position="bottom">
                    <NavLink
                        to="/analytics"
                        className={({ isActive }) =>
                            `px-3 py-1.5 rounded-lg text-xs font-bold transition-all flex items-center gap-1.5 ${
                                isActive || isAnalyticsView
                                    ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 shadow-sm'
                                    : 'text-zinc-400 hover:text-zinc-200 border border-transparent'
                            }`
                        }
                    >
                        <LineChart size={14} /> Dual-Trace Swarm Analytics
                    </NavLink>
                </Tooltip>
            </div>

            {/* Header */}
            <header className="flex justify-between items-end">
                <Tooltip content={i18n.t('benchmark.tooltip_main')} position="right">
                    <div>
                        <h1 className="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-white to-zinc-500 cursor-help">
                            {isAnalyticsView ? 'Dual-Trace Swarm Analytics' : i18n.t('benchmark.title')}
                        </h1>
                        <p className="text-xs text-zinc-400 mt-1">
                            {isAnalyticsView
                                ? 'Interactive side-by-side telemetry comparison of baseline directives vs 100% certified World Model execution.'
                                : 'Hardware latency distribution and performance profiling for autonomous agent nodes.'}
                        </p>
                    </div>
                </Tooltip>
                <div className="flex gap-4">
                    {!isAnalyticsView && RUNNER_BUTTONS.map(({ id, tooltipKey, labelKey, color, Icon }) => (
                        <Tooltip key={id} content={i18n.t(tooltipKey)} position="bottom">
                            <button
                                onClick={() => onRunBenchmark(id)}
                                disabled={runningId !== null}
                                className={`px-4 py-2 ${color} text-white font-bold rounded-lg text-[10px] transition-all flex items-center gap-2 shadow-lg disabled:opacity-50 uppercase tracking-widest cursor-pointer`}
                            >
                                {runningId === id ? <Loader2 size={12} className="animate-spin" /> : <Icon size={12} />}
                                {runningId === id ? i18n.t('benchmark.btn_executing') : i18n.t(labelKey)}
                            </button>
                        </Tooltip>
                    ))}
                    <Tooltip content="System telemetry status: All worker nodes operating nominally" position="bottom">
                        <div className="px-4 py-2 bg-emerald-500/10 border border-emerald-500/20 rounded-lg flex items-center gap-2 cursor-help">
                            <TrendingUp size={16} className="text-emerald-500" />
                            <span className="text-xs font-semibold text-emerald-500 uppercase tracking-tighter">{i18n.t('benchmark.status_nominal')}</span>
                        </div>
                    </Tooltip>
                </div>
            </header>
        </div>
    );
};

export default BenchmarkHeaderCard;

// Metadata: [BenchmarkHeaderCard]
