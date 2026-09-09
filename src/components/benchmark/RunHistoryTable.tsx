/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Benchmark / RunHistoryTable
 * - **Primary Entrypoints**: `RunHistoryTable`, `RunHistoryTableProps`
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
import { motion } from 'framer-motion';
import { History, ChevronRight } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { BenchmarkResult } from './types';

export interface RunHistoryTableProps {
    benchmarks: BenchmarkResult[];
    loading: boolean;
    selectedTests: string[];
    onToggleSelection: (id: string) => void;
    formatMs: (ms?: number) => string;
    getStatusIcon: (status: string) => React.ReactNode;
}

export const RunHistoryTable: React.FC<RunHistoryTableProps> = ({
    benchmarks,
    loading,
    selectedTests,
    onToggleSelection,
    formatMs,
    getStatusIcon,
}) => {
    return (
        <div className="rounded-2xl bg-[color:var(--color-surface)]/30 border border-[color:var(--color-border)]/50 overflow-hidden">
            <div className="p-6 border-b border-[color:var(--color-border)] flex items-center justify-between bg-[color:var(--color-surface)]/20">
                <Tooltip content={i18n.t('benchmark.tooltip_compare')} position="left">
                    <h2 className="text-lg font-semibold flex items-center gap-2 cursor-help">
                        <History size={20} className="text-zinc-400" />
                        {i18n.t('benchmark.label_historical_runs')}
                    </h2>
                </Tooltip>
                <div className="text-[10px] uppercase tracking-widest text-zinc-500 font-mono">
                    {i18n.t('benchmark.label_select_compare')}
                </div>
            </div>

            <div className="overflow-x-auto max-h-[520px] overflow-y-auto custom-scrollbar">
                <table className="w-full text-left border-collapse">
                    <thead className="sticky top-0 bg-[color:var(--color-surface)] backdrop-blur-md z-10">
                        <tr className="text-[10px] uppercase tracking-widest text-zinc-500 font-bold border-b border-[color:var(--color-border)]">
                            <th className="px-6 py-4">{i18n.t('benchmark.header_status')}</th>
                            <th className="px-6 py-4">{i18n.t('benchmark.header_test_id')}</th>
                            <th className="px-6 py-4">{i18n.t('benchmark.header_mean')}</th>
                            <th className="px-6 py-4">{i18n.t('benchmark.header_p95_p99')}</th>
                            <th className="px-6 py-4">{i18n.t('benchmark.header_target')}</th>
                            <th className="px-6 py-4">{i18n.t('benchmark.header_timestamp')}</th>
                            <th className="px-6 py-4 w-10">{i18n.t('benchmark.header_select')}</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-zinc-800/50">
                        {loading ? (
                            <tr>
                                <td colSpan={7} className="px-6 py-12 text-center text-zinc-500 animate-pulse font-mono uppercase tracking-widest text-xs">
                                    {i18n.t('benchmark.loading')}
                                </td>
                            </tr>
                        ) : benchmarks.length === 0 ? (
                            <tr>
                                <td colSpan={7} className="px-6 py-12 text-center text-zinc-500 font-mono uppercase tracking-widest text-xs">
                                    {i18n.t('benchmark.empty')}
                                </td>
                            </tr>
                        ) : (
                            benchmarks.map((bench) => (
                                <motion.tr
                                    key={bench.id}
                                    whileHover={{ backgroundColor: 'rgba(255,255,255,0.02)' }}
                                    className={`group cursor-pointer transition-colors ${selectedTests.includes(bench.id) ? 'bg-green-500/5' : ''}`}
                                    onClick={() => onToggleSelection(bench.id)}
                                    onKeyDown={(e) => {
                                        if (e.key === 'Enter' || e.key === ' ') {
                                            onToggleSelection(bench.id);
                                            e.preventDefault();
                                        }
                                    }}
                                    tabIndex={0}
                                    role="row"
                                >
                                    <td className="px-6 py-4">
                                        {getStatusIcon(bench.status)}
                                    </td>
                                    <td className="px-6 py-4">
                                        <div className="font-semibold text-sm">{bench.name}</div>
                                        <div className="text-[10px] font-mono text-zinc-500 uppercase tracking-tighter flex items-center gap-1 mt-0.5">
                                            <span className="px-1.5 py-0.5 rounded bg-zinc-800">{bench.category}</span>
                                            <ChevronRight size={10} className="text-zinc-700" />
                                            <span>{bench.test_id}</span>
                                        </div>
                                    </td>
                                    <td className="px-6 py-4 font-mono text-sm tracking-tighter">
                                        {formatMs(bench.mean_ms)}
                                    </td>
                                    <td className="px-6 py-4 font-mono text-[10px] text-zinc-400">
                                        {formatMs(bench.p95_ms)} / {formatMs(bench.p99_ms)}
                                    </td>
                                    <td className="px-6 py-4">
                                        <div className="text-[10px] text-zinc-500 font-mono italic max-w-[200px] truncate" title={bench.target_value}>
                                            {bench.target_value || i18n.t('benchmark.label_no_target')}
                                        </div>
                                    </td>
                                    <td className="px-6 py-4 text-xs text-zinc-500">
                                        {new Date(bench.created_at).toLocaleString()}
                                    </td>
                                    <td className="px-6 py-4">
                                        <div className={`w-4 h-4 rounded border transition-all ${
                                            selectedTests.includes(bench.id)
                                                ? 'border-green-500 bg-green-500 shadow-[0_0_8px_rgba(59,130,246,0.5)]'
                                                : 'border-zinc-700 bg-[color:var(--color-background)] group-hover:border-zinc-500'
                                        }`} />
                                    </td>
                                </motion.tr>
                            ))
                        )}
                    </tbody>
                </table>
            </div>
        </div>
    );
};

export default RunHistoryTable;
