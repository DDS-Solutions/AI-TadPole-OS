/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Job_Table_Row]` in observability traces.
 */

import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronRight, Activity, Pause, Play, Edit2, Trash2 } from 'lucide-react';
import { i18n } from '../../i18n';
import { Tooltip } from '../ui';
import { Job_History_View } from './Job_History_View';
import type { Scheduled_Job, Scheduled_Job_Run, Workflow_Entry } from '../../services/tadpoleos_service';
import type { Agent } from '../../types';

interface JobTableRowProps {
    job: Scheduled_Job;
    agents: Agent[];
    workflows: Workflow_Entry[];
    is_expanded: boolean;
    runs: Scheduled_Job_Run[] | undefined;
    toggle_expand: (id: string) => void;
    toggle_enable: (job: Scheduled_Job) => void;
    handle_edit: (job: Scheduled_Job) => void;
    delete_job: (id: string, name: string) => void;
}

export const Job_Table_Row: React.FC<JobTableRowProps> = ({
    job,
    agents,
    workflows,
    is_expanded,
    runs,
    toggle_expand,
    toggle_enable,
    handle_edit,
    delete_job
}) => {
    const agent = agents.find(a => a.id === job.agent_id);
    const workflow = workflows.find(w => w.id === job.workflow_id);
    const is_workflow = !!job.workflow_id;

    const get_status_indicator = (job: Scheduled_Job) => {
        if (!job.enabled) return <span className="px-2 py-0.5 rounded text-[10px] bg-zinc-800 text-zinc-500 font-mono">{i18n.t('scheduled_jobs.status_disabled')}</span>;
        if (job.consecutive_failures > 0) return <span className="px-2 py-0.5 rounded text-[10px] bg-amber-500/20 text-amber-500 border border-amber-500/30 font-mono">{i18n.t('scheduled_jobs.status_fails', { count: job.consecutive_failures })}</span>;
        return <span className="px-2 py-0.5 rounded text-[10px] bg-emerald-500/10 text-emerald-500 border border-emerald-500/20 font-mono">{i18n.t('scheduled_jobs.status_active')}</span>;
    };

    return (
        <React.Fragment>
            <tr className={`group hover:bg-[color:var(--color-surface)]/50 transition-colors ${is_expanded ? 'bg-[color:var(--color-surface)]/50' : ''}`}>
                <td className="px-6 py-4 w-10">
                    <button onClick={() => toggle_expand(job.id)} className="text-zinc-500 hover:text-white transition-colors">
                        <ChevronRight size={16} className={`transform transition-transform ${is_expanded ? 'rotate-90' : ''}`} />
                    </button>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                    <div className="font-semibold text-zinc-100">{job.name}</div>
                    <div className="text-[10px] text-zinc-500 font-mono mt-1">{job.id.slice(0, 8)}</div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center gap-2">
                        {is_workflow ? (
                            <>
                                <div className="w-5 h-5 rounded overflow-hidden bg-green-500/20 flex items-center justify-center border border-green-500/30">
                                    <Activity size={10} className="text-green-400" />
                                </div>
                                <span className="text-green-400 font-bold font-mono text-xs italic tracking-tighter">
                                    {workflow?.name || i18n.t('scheduled_jobs.workflow_default')}
                                </span>
                            </>
                        ) : (
                            <>
                                <div className="w-5 h-5 rounded overflow-hidden bg-zinc-800 shrink-0 border border-zinc-700">
                                    <img src={`https://api.dicebear.com/9.x/bottts-neutral/svg?seed=${job.agent_id}`} alt="Agent" />
                                </div>
                                <span className="text-zinc-300 font-mono text-xs">{agent?.name || job.agent_id}</span>
                            </>
                        )}
                    </div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-green-400 font-mono text-xs font-bold">{job.cron_expr}</td>
                <td className="px-6 py-4 whitespace-nowrap text-zinc-400 font-mono text-xs">
                    {job.enabled ? new Date(job.next_run_at).toLocaleString() : '-'}
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                    {get_status_indicator(job)}
                </td>
                <td className="px-6 py-4 whitespace-nowrap space-x-2 text-right">
                    <Tooltip content={job.enabled ? i18n.t('scheduled_jobs.pause_execution') : i18n.t('scheduled_jobs.resume_execution')} position="top">
                        <button onClick={() => toggle_enable(job)} className="p-1.5 text-zinc-500 hover:text-white rounded bg-zinc-800 hover:bg-zinc-700 transition-colors">
                            {job.enabled ? <Pause size={14} /> : <Play size={14} />}
                        </button>
                    </Tooltip>
                    <Tooltip content={i18n.t('scheduled_jobs.modify_config')} position="top">
                        <button onClick={() => handle_edit(job)} className="p-1.5 text-zinc-500 hover:text-green-400 rounded bg-zinc-800 hover:bg-zinc-700 transition-colors">
                            <Edit2 size={14} />
                        </button>
                    </Tooltip>
                    <Tooltip content={i18n.t('scheduled_jobs.purge_job_tooltip')} position="top">
                            <button 
                            onClick={(e) => {
                                e.stopPropagation();
                                delete_job(job.id, job.name);
                            }} 
                            className="p-2 text-zinc-500 hover:text-rose-400 rounded-lg bg-zinc-800/50 hover:bg-rose-500/10 border border-zinc-700/50 hover:border-rose-500/30 transition-all active:scale-90"
                            aria-label={`${i18n.t('common.delete')} ${job.name}`}
                            >
                                <Trash2 size={16} className="pointer-events-none" />
                            </button>
                        </Tooltip>
                </td>
            </tr>

            <AnimatePresence>
                {is_expanded && (
                    <tr className="bg-[color:var(--color-surface)]/30">
                        <td colSpan={7} className="p-0 border-b border-[color:var(--color-border)]">
                            <motion.div
                                initial={{ height: 0, opacity: 0 }}
                                animate={{ height: 'auto', opacity: 1 }}
                                exit={{ height: 0, opacity: 0 }}
                                className="overflow-hidden"
                            >
                                    <Job_History_View job={job} runs={runs} />
                            </motion.div>
                        </td>
                    </tr>
                )}
            </AnimatePresence>
        </React.Fragment>
    );
};

// Metadata: [Job_Table_Row]
