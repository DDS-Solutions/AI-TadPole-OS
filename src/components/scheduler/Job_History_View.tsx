/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Job_History_View]` in observability traces.
 */

import React, { useState } from 'react';
import { Activity, CheckCircle2, AlertCircle, ChevronRight } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { i18n } from '../../i18n';
import type { Scheduled_Job, Scheduled_Job_Run } from '../../services/tadpoleos_service';
import { Workflow_Run_Details } from './Workflow_Run_Details';

interface JobHistoryViewProps {
    job: Scheduled_Job;
    runs: Scheduled_Job_Run[] | undefined;
}

export const Job_History_View: React.FC<JobHistoryViewProps> = ({ job, runs }) => {
    const [expanded_run, set_expanded_run] = useState<string | null>(null);
    const is_workflow = !!job.workflow_id;

    const toggle_run = (run: Scheduled_Job_Run) => {
        if (!is_workflow || !run.mission_id) return;
        set_expanded_run(prev => (prev === run.id ? null : run.id));
    };

    return (
        <div className="p-6 ml-12 border-l-2 border-[color:var(--color-border)] my-4 space-y-4 max-w-full overflow-hidden">
            <div className="bg-black/20 rounded p-4 font-mono text-xs text-zinc-400 border border-[color:var(--color-surface)]/50 break-words whitespace-pre-wrap overflow-hidden max-w-full">
                <span className="text-zinc-600 block mb-2 uppercase tracking-widest text-[9px]">{i18n.t('scheduled_jobs.mission_prompt')}</span>
                {job.prompt || i18n.t('common.no_data')}
            </div>
    
            <h4 className="font-mono text-[10px] text-zinc-500 uppercase tracking-widest pt-2">{i18n.t('scheduled_jobs.run_history')}</h4>
            {!runs ? (
                <div className="text-zinc-600 text-xs flex items-center gap-2">
                    <Activity className="animate-pulse" size={14} strokeWidth={1.5} /> {i18n.t('scheduled_jobs.fetching_history')}
                </div>
            ) : runs.length === 0 ? (
                <div className="text-zinc-600 text-xs italic">{i18n.t('scheduled_jobs.no_history')}</div>
            ) : (
                <div className="space-y-3 max-w-full overflow-hidden">
                    {runs.map(run => {
                        const can_expand = is_workflow && !!run.mission_id;
                        const is_expanded = expanded_run === run.id;

                        return (
                            <div key={run.id} className="rounded border border-[color:var(--color-border)] overflow-hidden max-w-full">
                                <div 
                                    onClick={() => toggle_run(run)}
                                    className={`flex items-center justify-between bg-[color:var(--color-background)] p-3 text-xs font-mono group transition-colors overflow-hidden ${
                                        can_expand ? 'cursor-pointer hover:bg-zinc-800/40' : ''
                                    }`}
                                >
                                    <div className="flex items-center gap-3 min-w-0 flex-1 overflow-hidden pr-2">
                                        {can_expand && (
                                            <ChevronRight 
                                                size={14} 
                                                strokeWidth={1.5}
                                                className={`text-zinc-500 transform transition-transform shrink-0 ${is_expanded ? 'rotate-90' : ''}`} 
                                            />
                                        )}
                                        {run.status === 'completed' ? <CheckCircle2 size={14} strokeWidth={1.5} className="text-emerald-500 shrink-0" /> : <AlertCircle size={14} strokeWidth={1.5} className="text-rose-500 shrink-0" />}
                                        <span className="text-zinc-400 shrink-0">{new Date(run.started_at).toLocaleString()}</span>
                                        <span className={`shrink-0 font-bold ${run.status === 'completed' ? 'text-emerald-400' : 'text-rose-400'}`}>{run.status.toUpperCase()}</span>
                                    </div>
                                    <div className="flex items-center gap-4 text-zinc-500 shrink-0 ml-auto">
                                        <span className="shrink-0">${run.cost_usd.toFixed(4)}</span>
                                        <span className="text-zinc-700 group-hover:text-amber-500/50 transition-colors w-24 truncate max-w-[9rem] text-right shrink-0" title={run.mission_id || ''}>
                                            {run.mission_id || i18n.t('scheduled_jobs.no_mission')}
                                        </span>
                                    </div>
                                </div>

                                {run.output_summary && (
                                    <div className="p-3 bg-black/40 border-t border-[color:var(--color-border)]/50 text-[11px] font-mono text-zinc-300 break-words whitespace-pre-wrap overflow-x-auto max-w-full leading-relaxed">
                                        <div className="text-[9px] uppercase tracking-wider text-zinc-500 font-bold mb-1">Execution Summary</div>
                                        {run.output_summary}
                                    </div>
                                )}

                                <AnimatePresence>
                                    {is_expanded && run.mission_id && (
                                        <motion.div
                                            initial={{ height: 0, opacity: 0 }}
                                            animate={{ height: 'auto', opacity: 1 }}
                                            exit={{ height: 0, opacity: 0 }}
                                            className="overflow-hidden border-t border-zinc-800/40"
                                        >
                                            <Workflow_Run_Details run_id={run.mission_id} />
                                        </motion.div>
                                    )}
                                </AnimatePresence>
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
};

// Metadata: [Job_History_View]
