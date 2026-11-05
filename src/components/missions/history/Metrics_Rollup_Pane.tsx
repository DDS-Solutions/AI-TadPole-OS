/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / History / Metrics_Rollup_Pane
 * - **Primary Entrypoints**: `Metrics_Rollup_Pane`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` 5-way KPI summary cards, JetBrains Mono metrics, DESIGN.md token alignment.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React from 'react';
import { 
    ShieldCheck, 
    ShieldAlert, 
    Coins, 
    GitCommit, 
    FileDiff, 
    Cpu,
    CheckCircle2,
    AlertCircle,
} from 'lucide-react';
import type { MissionDossier } from '../../../types/mission_dossier';
import { Tooltip } from '../../ui';

interface MetricsRollupPaneProps {
    dossier: MissionDossier;
}

export const Metrics_Rollup_Pane: React.FC<MetricsRollupPaneProps> = ({ dossier }) => {
    const { mission, logs, diffs, debrief, economics } = dossier;

    const error_count = logs.filter((l) => l.severity === 'error').length;
    const warn_count = logs.filter((l) => l.severity === 'warning').length;
    const is_empty = logs.length === 0;
    const has_tamper = logs.some((l) => l.hash !== null && !l.is_verified) || Boolean(debrief.verdict?.includes('TAMPER'));
    const has_unverified = is_empty || logs.some((l) => l.hash === null);
    const error_density = logs.length > 0 ? ((error_count / logs.length) * 100).toFixed(1) : '0.0';

    const total_additions = diffs.reduce((acc, d) => acc + d.diff_summary.additions, 0);
    const total_deletions = diffs.reduce((acc, d) => acc + d.diff_summary.deletions, 0);

    const budget = economics.budget_usd > 0 ? economics.budget_usd : mission.budget_usd;
    const cost = economics.total_cost_usd > 0 ? economics.total_cost_usd : mission.cost_usd;
    const spend_ratio = budget > 0 ? Math.min(100, Math.round((cost / budget) * 100)) : 0;

    const drift_val = debrief.drift_score;
    const drift_status = drift_val !== null 
        ? (drift_val < 0.15 ? 'NOMINAL' : drift_val < 0.85 ? 'ELEVATED' : 'ANOMALY')
        : 'UNPROFILED';

    return (
        <div className="grid grid-cols-2 md:grid-cols-5 gap-3">
            {/* Card 1: Status & Verification */}
            <div className="p-3.5 rounded-xl bg-zinc-900/60 border border-zinc-800/80 backdrop-blur-md flex flex-col justify-between">
                <Tooltip content="ALCOA+ cryptographic audit integrity and execution status" position="top">
                    <div className="flex items-center justify-between text-zinc-400 mb-1 cursor-help">
                        <span className="text-[10px] font-bold uppercase tracking-wider">Status & Verification</span>
                        {has_tamper ? (
                            <ShieldAlert size={14} className="text-rose-400" />
                        ) : has_unverified ? (
                            <AlertCircle size={14} className="text-amber-400" />
                        ) : (
                            <ShieldCheck size={14} className="text-emerald-400" />
                        )}
                    </div>
                </Tooltip>
                <div className="flex items-baseline gap-2 mt-1">
                    <span className="text-base font-bold text-zinc-100 uppercase tracking-tight">
                        {mission.status}
                    </span>
                </div>
                <div className="mt-2 pt-2 border-t border-zinc-800/60 flex items-center justify-between text-[11px] font-mono">
                    <span className="text-zinc-500">ALCOA+</span>
                    <Tooltip content={
                        has_tamper 
                            ? "Tamper detected: SHA-256 hash mismatch in Merkle chain" 
                            : has_unverified 
                            ? (is_empty ? "No execution steps recorded in flight log" : "Legacy entries without cryptographic hash present")
                            : "All Merkle log hashes cryptographically verified"
                    } position="top">
                        {has_tamper ? (
                            <span className="inline-flex items-center gap-1 text-rose-400 font-semibold cursor-help">
                                <AlertCircle size={10} /> Tamper Flagged
                            </span>
                        ) : has_unverified ? (
                            <span className="inline-flex items-center gap-1 text-amber-400 font-semibold cursor-help">
                                <AlertCircle size={10} /> {is_empty ? "No Logs" : "Unverified Legacy"}
                            </span>
                        ) : (
                            <span className="inline-flex items-center gap-1 text-emerald-400 font-semibold cursor-help">
                                <CheckCircle2 size={10} /> Merkle Valid
                            </span>
                        )}
                    </Tooltip>
                </div>
            </div>

            {/* Card 2: Cost & Budget */}
            <div className="p-3.5 rounded-xl bg-zinc-900/60 border border-zinc-800/80 backdrop-blur-md flex flex-col justify-between">
                <Tooltip content="Real-time LLM token execution cost vs allocated ceiling" position="top">
                    <div className="flex items-center justify-between text-zinc-400 mb-1 cursor-help">
                        <span className="text-[10px] font-bold uppercase tracking-wider">Economics / Spend</span>
                        <Coins size={14} className="text-amber-400" />
                    </div>
                </Tooltip>
                <div className="flex items-baseline gap-1 mt-1 font-mono">
                    <span className="text-base font-bold text-zinc-100">
                        ${cost.toFixed(4)}
                    </span>
                    <span className="text-[10px] text-zinc-500">
                        / ${budget.toFixed(2)}
                    </span>
                </div>
                <Tooltip content={`Budget consumption: $${cost.toFixed(4)} spent of $${budget.toFixed(2)} ceiling (${spend_ratio}% burn, ${economics.total_tokens !== undefined ? economics.total_tokens.toLocaleString() : 0} tokens)`} position="top">
                    <div className="mt-2 pt-2 border-t border-zinc-800/60 flex flex-col gap-1 cursor-help">
                        <div className="w-full bg-zinc-800 rounded-full h-1.5 overflow-hidden">
                            <div 
                                className={`h-full rounded-full transition-all ${
                                    spend_ratio > 90 ? 'bg-rose-500' : spend_ratio > 60 ? 'bg-amber-500' : 'bg-cyan-500'
                                }`}
                                style={{ width: `${Math.max(4, spend_ratio)}%` }}
                            />
                        </div>
                        <div className="flex items-center justify-between text-[10px] text-zinc-500 font-mono">
                            <span>{spend_ratio}% burn</span>
                            <span>{economics.total_tokens !== undefined ? `${economics.total_tokens.toLocaleString()} tok` : '0 tok'}</span>
                        </div>
                    </div>
                </Tooltip>
            </div>

            {/* Card 3: Merkle Audit Turns */}
            <div className="p-3.5 rounded-xl bg-zinc-900/60 border border-zinc-800/80 backdrop-blur-md flex flex-col justify-between">
                <Tooltip content="Total discrete execution steps logged in Merkle ledger" position="top">
                    <div className="flex items-center justify-between text-zinc-400 mb-1 cursor-help">
                        <span className="text-[10px] font-bold uppercase tracking-wider">Audit Chain Turns</span>
                        <GitCommit size={14} className="text-cyan-400" />
                    </div>
                </Tooltip>
                <div className="flex items-baseline gap-2 mt-1 font-mono">
                    <span className="text-base font-bold text-zinc-100">
                        {logs.length}
                    </span>
                    <span className="text-[10px] text-zinc-500">records</span>
                </div>
                <div className="mt-2 pt-2 border-t border-zinc-800/60 flex items-center justify-between text-[11px] font-mono">
                    <span className="text-zinc-500">Alerts</span>
                    <Tooltip content={`${error_count} errors, ${warn_count} warnings across ${logs.length} turns (${error_density}% error density)`} position="top">
                        <div className="flex items-center gap-2 cursor-help">
                            {error_count > 0 && (
                                <span className="text-rose-400 font-bold">{error_count} err ({error_density}%)</span>
                            )}
                            {warn_count > 0 && (
                                <span className="text-amber-400">{warn_count} warn</span>
                            )}
                            {error_count === 0 && warn_count === 0 && (
                                <span className="text-emerald-400">0 anomalies</span>
                            )}
                        </div>
                    </Tooltip>
                </div>
            </div>

            {/* Card 4: CAS Mutated Files */}
            <div className="p-3.5 rounded-xl bg-zinc-900/60 border border-zinc-800/80 backdrop-blur-md flex flex-col justify-between">
                <Tooltip content="Files mutated during mission with pre-state snapshots" position="top">
                    <div className="flex items-center justify-between text-zinc-400 mb-1 cursor-help">
                        <span className="text-[10px] font-bold uppercase tracking-wider">CAS Mutation Ledger</span>
                        <FileDiff size={14} className="text-purple-400" />
                    </div>
                </Tooltip>
                <div className="flex items-baseline gap-2 mt-1 font-mono">
                    <span className="text-base font-bold text-zinc-100">
                        {diffs.length}
                    </span>
                    <span className="text-[10px] text-zinc-500">files changed</span>
                </div>
                <div className="mt-2 pt-2 border-t border-zinc-800/60 flex items-center justify-between text-[11px] font-mono">
                    <span className="text-zinc-500">Diff Lines</span>
                    <Tooltip content={`${total_additions} additions, ${total_deletions} deletions across file mutations`} position="top">
                        <span className="text-zinc-300 cursor-help">
                            <span className="text-emerald-400 font-semibold">+{total_additions}</span>{' '}
                            <span className="text-rose-400 font-semibold">-{total_deletions}</span>
                        </span>
                    </Tooltip>
                </div>
            </div>

            {/* Card 5: Behavioral Drift & Integrity */}
            <div className="p-3.5 rounded-xl bg-zinc-900/60 border border-zinc-800/80 backdrop-blur-md flex flex-col justify-between col-span-2 md:col-span-1">
                <Tooltip content="Cosine similarity metric measuring deviation from mission directive" position="top">
                    <div className="flex items-center justify-between text-zinc-400 mb-1 cursor-help">
                        <span className="text-[10px] font-bold uppercase tracking-wider">Agent 99 Drift</span>
                        <Cpu size={14} className="text-emerald-400" />
                    </div>
                </Tooltip>
                <div className="flex items-baseline gap-2 mt-1 font-mono">
                    <span className={`text-base font-bold ${
                        drift_val === null ? 'text-zinc-500' : drift_val < 0.15 ? 'text-emerald-400' : drift_val < 0.85 ? 'text-amber-400' : 'text-rose-400'
                    }`}>
                        {drift_val !== null ? drift_val.toFixed(3) : 'N/A'}
                    </span>
                    <span className="text-[10px] text-zinc-500">{drift_val !== null ? '< 0.850' : 'No Vector Drift'}</span>
                </div>
                <div className="mt-2 pt-2 border-t border-zinc-800/60 flex items-center justify-between text-[11px] font-mono">
                    <span className="text-zinc-500">Integrity</span>
                    <Tooltip content={
                        drift_status === 'NOMINAL' 
                            ? 'Execution strictly aligned with sovereign intent (< 0.150)' 
                            : drift_status === 'ELEVATED'
                            ? 'Moderate behavioral divergence within sovereign threshold (< 0.850)'
                            : drift_status === 'UNPROFILED' 
                            ? 'No behavioral drift vector recorded' 
                            : 'Vector drift exceeded sovereign threshold (>= 0.850)'
                    } position="top">
                        <span className={`px-1.5 py-0.2 rounded text-[10px] font-bold cursor-help ${
                            drift_status === 'NOMINAL'
                                ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                                : drift_status === 'ELEVATED'
                                ? 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                                : drift_status === 'UNPROFILED'
                                ? 'bg-zinc-800 text-zinc-400'
                                : 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
                        }`}>
                            {drift_status}
                        </span>
                    </Tooltip>
                </div>
            </div>
        </div>
    );
};
