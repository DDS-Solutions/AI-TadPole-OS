/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Workflow_Run_Details]` in observability traces.
 */

import React, { useEffect, useState, useCallback, useRef } from 'react';
import { 
    Activity, 
    AlertCircle, 
    Trophy, 
    Layers, 
    ChevronRight, 
    Clock, 
    Coins, 
    Terminal,
    Network
} from 'lucide-react';
import { tadpole_os_service } from '../../services/tadpoleos_service';
import type { Workflow_Step_Run, FanOutRunItem, TournamentCandidateItem } from '../../services/system_api_types';
import { i18n } from '../../i18n';

interface WorkflowRunDetailsProps {
    run_id: string;
}

export const Workflow_Run_Details: React.FC<WorkflowRunDetailsProps> = ({ run_id }) => {
    const [step_runs, set_step_runs] = useState<Workflow_Step_Run[]>([]);
    const [is_loading, set_is_loading] = useState(true);
    const [expanded_step_run, set_expanded_step_run] = useState<string | null>(null);

    const fetch_step_runs = useCallback(async () => {
        try {
            const data = await tadpole_os_service.get_workflow_run_steps(run_id);
            set_step_runs(data);
        } catch (error) {
            console.error('Failed to fetch step runs:', error);
        } finally {
            set_is_loading(false);
        }
    }, [run_id]);

    const step_runs_ref = useRef(step_runs);

    useEffect(() => {
        step_runs_ref.current = step_runs;
    }, [step_runs]);

    useEffect(() => {
        const timeout = setTimeout(() => {
            void fetch_step_runs();
        }, 0);

        // If any step is running, poll for updates in real time
        const interval = setInterval(() => {
            const is_running = step_runs_ref.current.some(run => run.status === 'running');
            if (is_running || step_runs_ref.current.length === 0) {
                void fetch_step_runs();
            }
        }, 3000);

        return () => {
            clearTimeout(timeout);
            clearInterval(interval);
        };
    }, [fetch_step_runs]);

    const toggle_step_expand = (id: string) => {
        set_expanded_step_run(prev => (prev === id ? null : id));
    };

    if (is_loading && step_runs.length === 0) {
        return (
            <div className="p-4 text-xs font-mono text-zinc-500 flex items-center gap-2">
                <Activity className="animate-spin text-zinc-400" size={12} strokeWidth={1.5} />
                {i18n.t('scheduled_jobs.loading_steps') || 'Loading execution step traces...'}
            </div>
        );
    }

    if (step_runs.length === 0) {
        return (
            <div className="p-4 text-xs font-mono text-zinc-600 italic">
                {i18n.t('scheduled_jobs.no_steps') || 'No step runs recorded for this execution.'}
            </div>
        );
    }

    // Group runs by step_id to identify loop runs
    const grouped_runs: Record<string, Workflow_Step_Run[]> = {};
    const step_order_map: Record<string, number> = {};
    const step_name_map: Record<string, string> = {};

    step_runs.forEach(run => {
        if (!grouped_runs[run.step_id]) {
            grouped_runs[run.step_id] = [];
        }
        grouped_runs[run.step_id].push(run);
        step_order_map[run.step_id] = run.step_order;
        step_name_map[run.step_id] = run.step_name;
    });

    const sorted_step_ids = Object.keys(grouped_runs).sort(
        (a, b) => step_order_map[a] - step_order_map[b]
    );

    return (
        <div className="p-4 bg-zinc-950/40 rounded-lg border border-zinc-800/40 my-3 ml-6 space-y-4 backdrop-blur-md">
            <header className="flex items-center justify-between border-b border-zinc-800/60 pb-2">
                <div className="flex items-center gap-2">
                    <Network size={14} className="text-zinc-400" strokeWidth={1.5} />
                    <span className="font-mono text-[10px] text-zinc-200 uppercase tracking-widest">
                        {i18n.t('scheduled_jobs.execution_trace') || 'Execution Trace'}
                    </span>
                </div>
                <span className="font-mono text-[9px] text-zinc-500 uppercase">
                    ID: {run_id}
                </span>
            </header>

            <div className="relative border-l border-zinc-800/80 ml-3 pl-6 space-y-6">
                {sorted_step_ids.map(step_id => {
                    const runs_for_step = grouped_runs[step_id];
                    const latest_run = runs_for_step[runs_for_step.length - 1];
                    const attempt_count = runs_for_step.length;

                    const fan_out = !!latest_run.step_config?.fan_out;
                    const tournament = !!latest_run.step_config?.tournament;

                    let status_color = 'text-zinc-500';
                    let bg_status = 'bg-zinc-900/20';
                    let border_glow = 'border-zinc-800/40 hover:border-zinc-700/60';

                    if (latest_run.status === 'running') {
                        status_color = 'text-blue-400';
                        bg_status = 'bg-blue-950/20';
                        border_glow = 'border-blue-500/50 shadow-lg shadow-blue-500/10';
                    } else if (latest_run.status === 'completed') {
                        status_color = 'text-emerald-400';
                        bg_status = 'bg-emerald-950/20';
                        border_glow = 'border-emerald-500/20 hover:border-emerald-500/40';
                    } else if (latest_run.status === 'failed') {
                        status_color = 'text-rose-400';
                        bg_status = 'bg-rose-950/20';
                        border_glow = 'border-rose-500/20 hover:border-rose-500/40';
                    }

                    return (
                        <div key={step_id} className="relative animate-fade-in">
                            {/* Chronological Step Circle Indicator */}
                            <div className={`absolute -left-[31px] top-1.5 w-3 h-3 rounded-full border-2 ${border_glow.split(' ')[0]} bg-zinc-950 flex items-center justify-center`}>
                                {latest_run.status === 'running' && (
                                    <div className="w-1.5 h-1.5 rounded-full bg-blue-400 animate-ping" />
                                )}
                            </div>

                            <div className={`p-4 rounded-xl border ${border_glow} ${bg_status} transition-all duration-200 ease-in`}>
                                <div className="flex items-center justify-between cursor-pointer" onClick={() => toggle_step_expand(latest_run.id)}>
                                    <div className="flex items-center gap-3">
                                        <ChevronRight 
                                            size={14} 
                                            strokeWidth={1.5}
                                            className={`text-zinc-500 transform transition-transform ${expanded_step_run === latest_run.id ? 'rotate-90' : ''}`} 
                                        />
                                        <div className="space-y-0.5">
                                            <div className="flex items-center gap-2">
                                                <h5 className="font-sans font-semibold text-xs text-zinc-100">
                                                    {latest_run.step_name}
                                                </h5>
                                                {attempt_count > 1 && (
                                                    <span className="px-1.5 py-0.5 rounded text-[8px] font-mono bg-zinc-800 text-amber-500 font-bold border border-zinc-700 uppercase">
                                                        {attempt_count} Runs
                                                    </span>
                                                )}
                                                {fan_out && (
                                                    <span className="px-1.5 py-0.5 rounded text-[8px] font-mono bg-zinc-800 text-blue-400 font-bold border border-zinc-700 uppercase">
                                                        MapReduce
                                                    </span>
                                                )}
                                                {tournament && (
                                                    <span className="px-1.5 py-0.5 rounded text-[8px] font-mono bg-zinc-800 text-amber-500 font-bold border border-zinc-700 uppercase">
                                                        Tournament
                                                    </span>
                                                )}
                                            </div>
                                            <span className="font-mono text-[9px] text-zinc-500 uppercase tracking-wider block">
                                                Agent: {latest_run.agent_id}
                                            </span>
                                        </div>
                                    </div>

                                    <div className="flex items-center gap-4 text-xs font-mono text-zinc-500">
                                        <span className="flex items-center gap-1">
                                            <Clock size={10} strokeWidth={1.5} />
                                            {latest_run.completed_at 
                                                ? `${Math.max(1, Math.round((new Date(latest_run.completed_at).getTime() - new Date(latest_run.started_at).getTime()) / 1000))}s`
                                                : 'Running...'}
                                        </span>
                                        <span className="flex items-center gap-0.5 text-zinc-600">
                                            <Coins size={10} strokeWidth={1.5} />
                                            ${latest_run.cost_usd.toFixed(4)}
                                        </span>
                                        <span className={`font-bold tracking-widest text-[9px] uppercase ${status_color}`}>
                                            {latest_run.status}
                                        </span>
                                    </div>
                                </div>

                                {/* Expanded Step Output & Detailed Widgets */}
                                {expanded_step_run === latest_run.id && (
                                    <div className="mt-4 pt-3 border-t border-zinc-800/40 space-y-4">
                                        {/* Loops attempts logs */}
                                        {attempt_count > 1 && (
                                            <div className="space-y-1.5">
                                                <span className="font-mono text-[9px] text-zinc-500 uppercase tracking-widest">History Loop Iterations</span>
                                                <div className="space-y-1">
                                                    {runs_for_step.slice(0, -1).map((past_run, idx) => (
                                                        <div key={past_run.id} className="flex items-center justify-between p-2 rounded bg-zinc-900/60 border border-zinc-800/30 text-[10px] font-mono text-zinc-500">
                                                            <div className="flex items-center gap-2">
                                                                <AlertCircle size={10} className="text-zinc-600" strokeWidth={1.5} />
                                                                <span>Iteration {idx + 1}</span>
                                                            </div>
                                                            <span>Started: {new Date(past_run.started_at).toLocaleTimeString()}</span>
                                                            <span className="text-rose-400/80 uppercase text-[9px]">Failed</span>
                                                        </div>
                                                    ))}
                                                </div>
                                            </div>
                                        )}

                                        {/* Widget A: FanOutMonitor (MapReduce Grid) */}
                                        {fan_out && latest_run.metadata && latest_run.metadata.runs && (
                                            <div className="space-y-3">
                                                <span className="font-mono text-[9px] text-zinc-400 uppercase tracking-widest flex items-center gap-1">
                                                    <Layers size={10} strokeWidth={1.5} /> Parallel Sub-Tasks Dispatcher
                                                </span>
                                                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2.5">
                                                    {(latest_run.metadata.runs as FanOutRunItem[]).map((run_item, idx) => {
                                                        let item_border = 'border-zinc-800 hover:border-zinc-700';
                                                        let item_bg = 'bg-zinc-900/20';
                                                        let dot_color = 'bg-zinc-600';

                                                        if (run_item.status === 'completed') {
                                                            item_border = 'border-emerald-500/20 hover:border-emerald-500/40';
                                                            item_bg = 'bg-emerald-500/5';
                                                            dot_color = 'bg-emerald-500';
                                                        } else if (run_item.status === 'failed') {
                                                            item_border = 'border-rose-500/20 hover:border-rose-500/40';
                                                            item_bg = 'bg-rose-500/5';
                                                            dot_color = 'bg-rose-500';
                                                        }

                                                        return (
                                                            <div key={idx} className={`p-3 rounded-lg border ${item_border} ${item_bg} flex flex-col justify-between gap-2 transition-all duration-200 ease-in group`}>
                                                                <div className="flex items-start justify-between">
                                                                    <span className="font-mono text-[10px] text-zinc-300 font-bold truncate max-w-[85%]" title={typeof run_item.item === 'string' ? run_item.item : JSON.stringify(run_item.item)}>
                                                                        {typeof run_item.item === 'string' ? run_item.item.split('/').pop() : `Task ${idx + 1}`}
                                                                    </span>
                                                                    <div className={`w-1.5 h-1.5 rounded-full ${dot_color}`} />
                                                                </div>
                                                                <div className="flex items-center justify-between text-[9px] font-mono text-zinc-500">
                                                                    <span>{run_item.elapsed_ms}ms</span>
                                                                    <span className="uppercase text-[8px] font-bold tracking-wider">{run_item.status}</span>
                                                                </div>
                                                            </div>
                                                        );
                                                    })}
                                                </div>
                                            </div>
                                        )}

                                        {/* Widget B: TournamentViewer (VotingDiff side-by-side) */}
                                        {tournament && latest_run.metadata && latest_run.metadata.candidates && (
                                            <div className="space-y-4 animate-slide-down">
                                                {/* Judge winner highlights */}
                                                {latest_run.metadata.judge && (
                                                    <div className="p-3 bg-amber-500/5 border border-amber-500/20 rounded-xl flex gap-3 shadow-lg shadow-amber-500/5">
                                                        <Trophy className="text-amber-500 shrink-0 mt-0.5 animate-pulse" size={16} strokeWidth={1.5} />
                                                        <div className="space-y-1">
                                                            <h6 className="font-sans font-bold text-xs text-amber-500">
                                                                {i18n.t('scheduled_jobs.tournament_winner') || 'Consensus Winner selected'}
                                                            </h6>
                                                            <p className="font-mono text-[10px] text-zinc-300 leading-relaxed">
                                                                {latest_run.metadata.judge.output}
                                                            </p>
                                                            <div className="font-mono text-[8px] text-zinc-500 uppercase tracking-widest pt-1 flex gap-3">
                                                                <span>Judge: {latest_run.metadata.judge.agent_id}</span>
                                                                <span>Time: {latest_run.metadata.judge.elapsed_ms}ms</span>
                                                            </div>
                                                        </div>
                                                    </div>
                                                )}

                                                <span className="font-mono text-[9px] text-zinc-400 uppercase tracking-widest flex items-center gap-1">
                                                    <Terminal size={10} strokeWidth={1.5} /> Candidate Responses Comparison
                                                </span>

                                                <div className="flex gap-3 overflow-x-auto pb-2 scrollbar-thin scrollbar-thumb-zinc-800">
                                                    {(latest_run.metadata.candidates as TournamentCandidateItem[]).map((cand, idx) => (
                                                        <div key={idx} className="min-w-[280px] max-w-[320px] flex-1 p-3 rounded-lg border border-zinc-800 bg-zinc-900/30 font-mono text-[10px] space-y-2 hover:border-zinc-700/60 transition-all duration-200 ease-in">
                                                            <div className="flex items-center justify-between border-b border-zinc-800/40 pb-1.5">
                                                                <span className="font-bold text-zinc-300">{cand.agent_id}</span>
                                                                <span className="text-zinc-500">{cand.elapsed_ms}ms</span>
                                                            </div>
                                                            <div className="text-zinc-400 max-h-36 overflow-y-auto whitespace-pre-wrap leading-relaxed pr-1 scrollbar-thin">
                                                                {cand.output}
                                                            </div>
                                                        </div>
                                                    ))}
                                                </div>
                                            </div>
                                        )}

                                        {/* Standard output logs viewer */}
                                        {!tournament && latest_run.output_text && (
                                            <div className="bg-zinc-950/40 rounded-xl p-3 border border-zinc-800/40 flex flex-col gap-1.5 font-mono text-[10px] leading-relaxed">
                                                <div className="flex items-center gap-2 text-zinc-500 uppercase tracking-wider text-[8px] font-bold border-b border-zinc-800/40 pb-1">
                                                    <Terminal size={10} strokeWidth={1.5} /> Output Text Logs
                                                </div>
                                                <div className="text-zinc-300 max-h-48 overflow-y-auto whitespace-pre-wrap pr-1 scrollbar-thin scrollbar-thumb-zinc-800">
                                                    {latest_run.output_text}
                                                </div>
                                            </div>
                                        )}
                                    </div>
                                )}
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
};

// Metadata: [Workflow_Run_Details]
