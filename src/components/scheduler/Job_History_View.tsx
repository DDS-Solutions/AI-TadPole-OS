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

import React from 'react';
import { Activity, CheckCircle2, AlertCircle } from 'lucide-react';
import { i18n } from '../../i18n';
import type { Scheduled_Job, Scheduled_Job_Run } from '../../services/tadpoleos_service';

interface JobHistoryViewProps {
    job: Scheduled_Job;
    runs: Scheduled_Job_Run[] | undefined;
}

export const Job_History_View: React.FC<JobHistoryViewProps> = ({ job, runs }) => {
    return (
        <div className="p-6 ml-12 border-l-2 border-[color:var(--color-border)] my-4 space-y-4">
            <div className="bg-black/20 rounded p-4 font-mono text-xs text-zinc-400 border border-[color:var(--color-surface)]/50">
                <span className="text-zinc-600 block mb-2 uppercase tracking-widest text-[9px]">{i18n.t('scheduled_jobs.mission_prompt')}</span>
                {job.prompt || i18n.t('common.no_data')}
            </div>
    
            <h4 className="font-mono text-[10px] text-zinc-500 uppercase tracking-widest pt-2">{i18n.t('scheduled_jobs.run_history')}</h4>
            {!runs ? (
                <div className="text-zinc-600 text-xs flex items-center gap-2">
                    <Activity className="animate-pulse" size={14} /> {i18n.t('scheduled_jobs.fetching_history')}
                </div>
            ) : runs.length === 0 ? (
                <div className="text-zinc-600 text-xs italic">{i18n.t('scheduled_jobs.no_history')}</div>
            ) : (
                <div className="space-y-2">
                    {runs.map(run => (
                        <div key={run.id} className="flex items-center justify-between bg-[color:var(--color-background)] p-2 rounded border border-[color:var(--color-border)] text-xs font-mono group">
                            <div className="flex items-center gap-4">
                                {run.status === 'completed' ? <CheckCircle2 size={14} className="text-emerald-500" /> : <AlertCircle size={14} className="text-rose-500" />}
                                <span className="text-zinc-400">{new Date(run.started_at).toLocaleString()}</span>
                                <span className={`${run.status === 'completed' ? 'text-emerald-400' : 'text-rose-400'}`}>{run.status.toUpperCase()}</span>
                            </div>
                            <div className="flex items-center gap-4 text-zinc-500">
                                <span>${run.cost_usd.toFixed(4)}</span>
                                <span className="text-zinc-700 group-hover:text-amber-500/50 transition-colors w-16 truncate max-w-[8rem] text-right" title={run.mission_id || ''}>
                                    {run.mission_id || i18n.t('scheduled_jobs.no_mission')}
                                </span>
                            </div>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};

// Metadata: [Job_History_View]
