/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / History / Agent99_Debrief_Viewer
 * - **Primary Entrypoints**: `Agent99_Debrief_Viewer`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Post-mission QA audit insights, behavioral drift meter, actionable recommendations.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React from 'react';
import { 
    Cpu, 
    ShieldCheck, 
    ShieldAlert, 
    CheckSquare, 
    Lightbulb, 
    Activity,
} from 'lucide-react';
import type { DebriefPayload } from '../../../types/mission_dossier';
import { Tooltip } from '../../ui';

interface Agent99DebriefViewerProps {
    debrief: DebriefPayload;
    mission_id: string;
}

export const Agent99_Debrief_Viewer: React.FC<Agent99DebriefViewerProps> = ({
    debrief,
    mission_id,
}) => {
    const drift_val = debrief.drift_score;
    const is_success = debrief.verdict === 'VERIFIED_SUCCESS';
    const is_tampered = debrief.verdict === 'COMPLETED_WITH_TAMPER_FLAG';

    return (
        <div className="flex flex-col h-full bg-zinc-950 border border-zinc-800/80 rounded-xl overflow-hidden p-6 gap-6 overflow-y-auto custom-scrollbar">
            {/* Verdict Banner */}
            <div className={`p-4 rounded-xl border flex items-center justify-between gap-4 ${
                is_success
                    ? 'bg-emerald-950/20 border-emerald-500/40 text-emerald-200'
                    : is_tampered
                    ? 'bg-rose-950/20 border-rose-500/40 text-rose-200'
                    : 'bg-amber-950/20 border-amber-500/40 text-amber-200'
            }`}>
                <div className="flex items-center gap-3">
                    <Tooltip content={is_success ? "Audit proof: All steps verified with zero tamper anomalies" : is_tampered ? "Security warning: Tamper flag detected in Merkle audit trail" : "Flight analysis underway"} position="bottom">
                        <div className="p-2 rounded-lg bg-slate-950/40 border border-current cursor-help">
                            {is_success ? (
                                <ShieldCheck size={24} className="text-emerald-400" />
                            ) : (
                                <ShieldAlert size={24} className={is_tampered ? 'text-rose-400' : 'text-amber-400'} />
                            )}
                        </div>
                    </Tooltip>
                    <div className="flex flex-col">
                        <span className="text-xs font-bold uppercase tracking-widest text-zinc-400">
                            Agent 99 QA Auditor Verdict
                        </span>
                        <h2 className="text-base font-bold tracking-tight">
                            {debrief.verdict?.replace(/_/g, ' ') || 'ANALYSIS IN PROGRESS'}
                        </h2>
                    </div>
                </div>

                <div className="font-mono text-xs text-right">
                    <span className="text-zinc-500">Mission Target:</span>
                    <div className="font-bold text-zinc-200">{mission_id}</div>
                </div>
            </div>

            {/* Behavioral Drift Detection Box */}
            <div className="p-4 rounded-xl bg-zinc-900/40 border border-zinc-800/80 flex flex-col gap-3">
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <Activity size={16} className={drift_val !== null ? 'text-emerald-400' : 'text-zinc-500'} />
                        <h3 className="text-xs font-bold text-zinc-100 uppercase tracking-wider">
                            SEC-06 Behavioral Drift Analysis (LanceDB Vector Space)
                        </h3>
                    </div>
                    <Tooltip content="LanceDB cosine distance between mission execution tokens and canonical agent identity" position="top">
                        <span className="font-mono text-xs text-zinc-400 cursor-help">
                            {drift_val !== null ? (
                                <span className="text-emerald-400 font-bold">
                                    Drift: {drift_val.toFixed(4)} (Threshold: &lt; 0.8500)
                                </span>
                            ) : (
                                <span className="text-zinc-500">Not Profiled</span>
                            )}
                        </span>
                    </Tooltip>
                </div>

                {drift_val !== null ? (
                    <>
                        {/* Progress Meter */}
                        <div className="flex flex-col gap-1.5">
                            <div className="w-full bg-zinc-950 rounded-full h-2.5 p-0.5 border border-zinc-800 overflow-hidden">
                                <div 
                                    className={`h-full rounded-full transition-all ${
                                        drift_val > 0.85 ? 'bg-rose-500' : drift_val >= 0.15 ? 'bg-amber-500' : 'bg-emerald-500'
                                    }`}
                                    style={{ width: `${Math.max(2, Math.min(100, (drift_val / 0.85) * 100))}%` }}
                                />
                            </div>
                            <div className="flex justify-between text-[10px] font-mono text-zinc-500">
                                <span>0.000 (Exact Baseline Identity)</span>
                                <span>0.150 (Moderate Divergence Threshold)</span>
                                <span className="text-rose-400">0.850 (Circuit Breaker Tripped)</span>
                            </div>
                        </div>
                        <p className="text-xs text-zinc-400 leading-relaxed font-mono">
                            {drift_val > 0.85 ? (
                                <span className="text-rose-400 font-semibold">
                                    Vector drift exceeded sovereign threshold (0.850). Agent reasoning diverged significantly from baseline directive.
                                </span>
                            ) : drift_val >= 0.15 ? (
                                <span className="text-amber-400">
                                    Moderate behavioral drift detected. Agent explored alternative execution pathways.
                                </span>
                            ) : (
                                <span className="text-emerald-400">
                                    Normalized cosine distance between agent reasoning tokens and the core identity context remained within strict sovereign tolerances throughout this flight.
                                </span>
                            )}
                        </p>
                    </>
                ) : (
                    <p className="text-xs text-zinc-500 leading-relaxed font-mono italic">
                        Vector memory semantic drift calculation was not engaged for this mission (UNPROFILED).
                    </p>
                )}
            </div>

            {/* Agent 99 QA Auditor Post-Mission Notes */}
            <div className="p-4 rounded-xl bg-zinc-900/40 border border-zinc-800/80 flex flex-col gap-3">
                <div className="flex items-center gap-2">
                    <Cpu size={16} className="text-cyan-400" />
                    <h3 className="text-xs font-bold text-zinc-100 uppercase tracking-wider">
                        Distilled Learnings & Auditor Transcript
                    </h3>
                </div>
                <div className="p-3.5 rounded-lg bg-slate-950/50 border border-zinc-900 font-mono text-xs text-zinc-300 leading-relaxed whitespace-pre-wrap">
                    {debrief.agent99_notes ? (
                        debrief.agent99_notes
                    ) : (
                        <span className="text-zinc-500 italic">
                            No Agent 99 debrief transcript recorded for this flight.
                        </span>
                    )}
                </div>
            </div>

            {/* Actionable Recommendations Checklist */}
            <div className="p-4 rounded-xl bg-zinc-900/40 border border-zinc-800/80 flex flex-col gap-3">
                <div className="flex items-center gap-2">
                    <Lightbulb size={16} className="text-amber-400" />
                    <h3 className="text-xs font-bold text-zinc-100 uppercase tracking-wider">
                        Sovereign Recommendations & Next Steps
                    </h3>
                </div>
                <div className="flex flex-col gap-2">
                    {debrief.recommendations.map((rec, i) => (
                        <div
                            key={i}
                            className="p-3 rounded-lg bg-zinc-950/60 border border-zinc-800/60 flex items-start gap-2.5 text-xs text-zinc-200 font-sans"
                        >
                            <CheckSquare size={14} className="text-cyan-400 flex-shrink-0 mt-0.5" />
                            <span>{rec}</span>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
};
