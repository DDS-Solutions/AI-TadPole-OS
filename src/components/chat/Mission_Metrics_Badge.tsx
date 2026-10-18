/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Chat / Mission_Metrics_Badge
 * - **Primary Entrypoints**: `Mission_Metrics_Badge`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Displays aggregated execution metrics and context compaction warnings deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React from 'react';
import { Activity, AlertTriangle, Cpu, DollarSign, RefreshCw } from 'lucide-react';
import clsx from 'clsx';
import { use_sovereign_store } from '../../stores/sovereign_store';
import { Tooltip } from '../ui';

export const Mission_Metrics_Badge: React.FC = () => {
    const metrics = use_sovereign_store((s) => s.latest_metrics);

    if (!metrics || metrics.turns === 0) {
        return null;
    }

    const format_tokens = (num: number): string => {
        if (num >= 1_000_000) return `${(num / 1_000_000).toFixed(1)}M`;
        if (num >= 1_000) return `${(num / 1_000).toFixed(1)}k`;
        return `${num}`;
    };

    const cost_usd = (metrics.total_cost_micro_usd / 1_000_000).toFixed(4);
    const total_tokens = metrics.total_input_tokens + metrics.total_output_tokens;
    const is_thrashing = metrics.excessive_summarization_warning || metrics.total_summarizations >= 3;

    return (
        <div className="flex items-center gap-2">
            {/* Context Thrashing Alert */}
            {is_thrashing && (
                <Tooltip
                    content={`High Compaction Rate: ${metrics.total_summarizations} summarizations triggered. Working memory is thrashing. Recommend subagent delegation.`}
                    position="bottom"
                >
                    <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-amber-500/15 border border-amber-500/40 text-amber-300 text-[10px] font-mono animate-pulse">
                        <AlertTriangle size={11} className="text-amber-400" />
                        <span className="font-bold">Thrashing Risk</span>
                    </div>
                </Tooltip>
            )}

            {/* Metrics HUD Pill */}
            <Tooltip
                content={`Mission Execution Rollup: ${metrics.turns} turns, ${metrics.tool_calls_attempted} tool calls (${metrics.tool_calls_failed} failed), ${metrics.cached_reads_hit} cached reads, ${metrics.files_modified_count} files edited, ${metrics.total_summarizations} compactions, ${metrics.total_sub_agents} subagents.`}
                position="bottom"
            >
                <div className="flex items-center gap-2 px-2.5 py-1 rounded-lg bg-zinc-900/80 border border-zinc-800 text-[10px] font-mono text-zinc-400">
                    {/* Turns */}
                    <div className="flex items-center gap-1">
                        <Activity size={10} className="text-zinc-500" />
                        <span className="text-zinc-300 font-semibold">T:{metrics.turns}</span>
                    </div>

                    <span className="text-zinc-700">|</span>

                    {/* Tokens */}
                    <div className="flex items-center gap-1">
                        <Cpu size={10} className="text-zinc-500" />
                        <span className="text-zinc-300">{format_tokens(total_tokens)}</span>
                    </div>

                    <span className="text-zinc-700">|</span>

                    {/* Cost */}
                    <div className="flex items-center gap-0.5">
                        <DollarSign size={10} className="text-emerald-500" />
                        <span className="text-zinc-300">${cost_usd}</span>
                    </div>

                    {/* Compactions count if > 0 */}
                    {metrics.total_summarizations > 0 && (
                        <>
                            <span className="text-zinc-700">|</span>
                            <div className={clsx(
                                "flex items-center gap-1",
                                is_thrashing ? "text-amber-400 font-bold" : "text-zinc-400"
                            )}>
                                <RefreshCw size={9} />
                                <span>{metrics.total_summarizations}c</span>
                            </div>
                        </>
                    )}
                </div>
            </Tooltip>
        </div>
    );
};

Mission_Metrics_Badge.displayName = 'Mission_Metrics_Badge';
