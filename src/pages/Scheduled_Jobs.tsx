/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Cron-style scheduler for autonomous agent tasks. 
 * Orchestrates the visualization of upcoming jobs, execution history, and task periodicity.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Job execution overlap detection failure, or schedule drift due to local clock desync.
 * - **Telemetry Link**: Search for `[Scheduled_Jobs]` or `JOB_TICK` in service logs.
 */

import React, { useState, useEffect, useMemo, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Clock, Play, Pause, Trash2, ChevronRight, Activity, CheckCircle2, AlertCircle, Plus, Edit2 } from 'lucide-react';
import { tadpole_os_service, type Scheduled_Job, type Scheduled_Job_Run, type Workflow_Entry } from '../services/tadpoleos_service';
import { use_agent_store } from '../stores/agent_store';
import { event_bus } from '../services/event_bus';
import { Tooltip } from '../components/ui';
import { Confirm_Dialog } from '../components/ui/Confirm_Dialog';
import { i18n } from '../i18n';

// --- TYPE DEFINITIONS ---

interface job_config_state {
    name: string;
    agent_id: string;
    workflow_id: string | null;
    prompt: string;
    cron_expr: string;
    budget_usd: number;
    max_failures: number;
}

// --- Component: Job_Form_Manager ---

interface job_form_manager_props {
    job_type: 'agent' | 'workflow';
    set_job_type: (type: 'agent' | 'workflow') => void;
    current_job_config: job_config_state;
    set_job_config: React.Dispatch<React.SetStateAction<job_config_state>>;
    target_search: string;
    set_target_search: React.Dispatch<React.SetStateAction<string>>;
    filtered_agents: Array<{ id: string; name: string; role: string }>;
    filtered_workflows: Array<Workflow_Entry & { type: 'continuity' | 'passive' }>;
    is_creating: boolean;
    set_creating: React.Dispatch<React.SetStateAction<boolean>>;
    editing_job_id: string | null;
    set_editing_job_id: React.Dispatch<React.SetStateAction<string | null>>;
    handle_job_submit: (e: React.FormEvent) => Promise<void>;
}

/**
 * Component: Job_Form_Manager
 * Handles the configuration and submission of new automated jobs.
 * Supports both single-agent missions and multi-step workflows.
 */
const Job_Form_Manager: React.FC<job_form_manager_props> = ({ 
    job_type, 
    set_job_type,
    current_job_config, 
    set_job_config, 
    target_search, 
    set_target_search, 
    filtered_agents, 
    filtered_workflows, 
    set_creating,
    editing_job_id,
    set_editing_job_id,
    handle_job_submit
}) => {
    const handle_input_change = useCallback((key: keyof job_config_state, value: string | number | boolean | string[]) => {
        set_job_config(prev => ({ ...prev, [key]: value }));
    }, [set_job_config]);

    return (
        <div className="bg-zinc-900 border border-green-500/30 rounded-xl p-6 shadow-2xl max-h-[85vh] overflow-y-auto custom-scrollbar">
            <div className="flex justify-between items-center mb-4 sticky top-0 bg-zinc-900 z-10 pb-2">
                <h2 className="text-lg font-bold text-green-400 font-mono uppercase">
                    {editing_job_id ? i18n.t('scheduled_jobs.modify_config') : i18n.t('scheduled_jobs.configure_new')}
                </h2>
                <div className="flex bg-zinc-950 p-1 rounded-lg border border-zinc-800">
                    <button
                        onClick={() => { set_job_type('agent'); set_target_search(''); }}
                        className={`px-3 py-1 text-[10px] font-bold rounded ${job_type === 'agent' ? 'bg-green-600 text-white' : 'text-zinc-500 hover:text-zinc-300'}`}
                    >
                        {i18n.t('scheduled_jobs.single_agent')}
                    </button>
                    <button
                        onClick={() => { set_job_type('workflow'); set_target_search(''); }}
                        className={`px-3 py-1 text-[10px] font-bold rounded ${job_type === 'workflow' ? 'bg-green-600 text-white' : 'text-zinc-500 hover:text-zinc-300'}`}
                    >
                        {i18n.t('scheduled_jobs.multi_step_workflow')}
                    </button>
                </div>
            </div>
            <form onSubmit={handle_job_submit} className="space-y-4">
                {/* Job Name */}
                <div>
                    <Tooltip content={i18n.t('scheduled_jobs.job_name_tooltip')} position="top">
                        <label htmlFor="job_name" className="block text-xs font-mono text-zinc-500 mb-1 cursor-help">{i18n.t('scheduled_jobs.job_name')}</label>
                    </Tooltip>
                    <input 
                        id="job_name" required 
                        type="text" 
                        className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-sm" 
                        value={current_job_config.name} 
                        onChange={e => handle_input_change('name', e.target.value)} 
                        placeholder={i18n.t('scheduled_jobs.job_name_placeholder')} 
                    />
                </div>
                
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 min-w-0">
                    {/* Target Selector */}
                    <div className="min-w-0">
                        {job_type === 'agent' ? (
                            <>
                                <Tooltip content={i18n.t('scheduled_jobs.target_agent_tooltip')} position="top">
                                    <label htmlFor="target_agent" className="block text-xs font-mono text-zinc-500 mb-1 cursor-help">{i18n.t('scheduled_jobs.target_agent')}</label>
                                </Tooltip>
                                <div className="space-y-2">
                                    <input 
                                        type="text" 
                                        placeholder={i18n.t('scheduled_jobs.filter_agents')} 
                                        className="w-full bg-zinc-950 border border-zinc-800 rounded p-1.5 text-[10px] font-mono focus:border-green-500/50 outline-none"
                                        value={target_search}
                                        onChange={e => set_target_search(e.target.value)}
                                    />
                                    <select 
                                        id="target_agent" 
                                        required 
                                        className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-sm" 
                                        value={current_job_config.agent_id || ''} 
                                        onChange={e => {
                                            handle_input_change('agent_id', e.target.value);
                                            handle_input_change('workflow_id', '');
                                        }}
                                    >
                                        <option value="">{i18n.t('scheduled_jobs.select_agent')}</option>
                                        {filtered_agents.map(a => <option key={a.id} value={a.id}>{a.name} ({a.role})</option>)}
                                    </select>
                                </div>
                            </>
                        ) : (
                            <>
                                <Tooltip content={i18n.t('scheduled_jobs.target_workflow_tooltip')} position="top">
                                    <label htmlFor="target_workflow" className="block text-xs font-mono text-zinc-500 mb-1 cursor-help">{i18n.t('scheduled_jobs.target_workflow')}</label>
                                </Tooltip>
                                <div className="space-y-2">
                                    <input 
                                        type="text" 
                                        placeholder={i18n.t('scheduled_jobs.filter_workflows')} 
                                        className="w-full bg-zinc-950 border border-zinc-800 rounded p-1.5 text-[10px] font-mono focus:border-green-500/50 outline-none"
                                        value={target_search}
                                        onChange={(e) => set_target_search(e.target.value)}
                                    />
                                    <select 
                                        id="target_workflow" 
                                        required 
                                        className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-sm" 
                                        value={current_job_config.workflow_id || ''} 
                                        onChange={e => {
                                            handle_input_change('workflow_id', e.target.value);
                                            handle_input_change('agent_id', '');
                                        }}
                                    >
                                        <option value="">{i18n.t('scheduled_jobs.select_workflow')}</option>
                                        <optgroup label={i18n.t('scheduled_jobs.mission_sequences')} className="bg-zinc-900 text-green-400 font-mono text-[10px]">
                                            {filtered_workflows.filter(w => w.type === 'continuity').map(w => (
                                                <option key={w.id} value={w.id} className="text-zinc-300 bg-zinc-950">{w.name}</option>
                                            ))}
                                        </optgroup>
                                        <optgroup label={i18n.t('scheduled_jobs.guiding_protocols')} className="bg-zinc-900 text-amber-400 font-mono text-[10px]">
                                            {filtered_workflows.filter(w => w.type === 'passive').map(w => (
                                                <option key={w.id} value={w.id} className="text-zinc-300 bg-zinc-950">{w.name}</option>
                                            ))}
                                        </optgroup>
                                    </select>
                                </div>
                            </>
                        )}
                    </div>
                </div>

                {/* Cron Expression */}
                <div>
                    <Tooltip content={i18n.t('scheduled_jobs.cron_tooltip')} position="top">
                        <label htmlFor="cron_expr" className="block text-xs font-mono text-zinc-500 mb-1 cursor-help">{i18n.t('scheduled_jobs.cron_expression')}</label>
                    </Tooltip>
                    <input 
                        id="cron_expr" required 
                        type="text" 
                        className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-sm font-mono" 
                        value={current_job_config.cron_expr} 
                        onChange={e => handle_input_change('cron_expr', e.target.value)} 
                        placeholder={i18n.t('scheduled_jobs.cron_placeholder')} 
                    />
                </div>
                
                {/* Mission Prompt (if Agent job) */}
                {job_type === 'agent' && (
                    <div>
                        <Tooltip content={i18n.t('scheduled_jobs.mission_prompt_tooltip')} position="top">
                            <label htmlFor="mission_prompt" className="block text-xs font-mono text-zinc-500 mb-1 cursor-help">{i18n.t('scheduled_jobs.mission_prompt')}</label>
                        </Tooltip>
                        <textarea 
                            id="mission_prompt" 
                            required 
                            rows={3} 
                            className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-sm resize-none custom-scrollbar" 
                            value={current_job_config.prompt} 
                            onChange={e => handle_input_change('prompt', e.target.value)} 
                            placeholder={i18n.t('scheduled_jobs.mission_prompt_placeholder')} 
                        />
                    </div>
                )}

                {/* Budget and Max Failures */}
                <div className="grid grid-cols-2 gap-4">
                    <div>
                        <Tooltip content={i18n.t('scheduled_jobs.budget_cap_tooltip')} position="top">
                            <label htmlFor="budget_usd" className="block text-xs font-mono text-zinc-500 mb-1 cursor-help">{i18n.t('scheduled_jobs.budget_cap')}</label>
                        </Tooltip>
                        <input 
                            id="budget_usd" 
                            type="number" step="0.01" 
                            className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-sm" 
                            value={current_job_config.budget_usd} 
                            onChange={e => handle_input_change('budget_usd', parseFloat(e.target.value) || 0)} 
                        />
                    </div>
                    <div>
                        <Tooltip content={i18n.t('scheduled_jobs.max_failures_tooltip')} position="top">
                            <label htmlFor="max_failures" className="block text-xs font-mono text-zinc-500 mb-1 cursor-help">{i18n.t('scheduled_jobs.max_failures_label')}</label>
                        </Tooltip>
                        <input 
                            id="max_failures" 
                            type="number" 
                            className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-sm" 
                            value={current_job_config.max_failures} 
                            onChange={e => handle_input_change('max_failures', parseInt(e.target.value, 10) || 0)} 
                        />
                    </div>
                </div>
                
                {/* Submission Buttons */}
                <div className="flex gap-2 pt-2 border-t border-zinc-800">
                    <button type="submit" className="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 rounded text-sm font-bold shadow-lg shadow-emerald-500/20">
                        {editing_job_id ? i18n.t('scheduled_jobs.update_sync') : i18n.t('scheduled_jobs.save_job')}
                    </button>
                    <button type="button" onClick={() => {
                        set_creating(false);
                        set_editing_job_id(null);
                    }} className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded text-sm text-zinc-300">
                        {i18n.t('scheduled_jobs.cancel')}
                    </button>
                </div>
            </form>
        </div>
    );
};

// --- Component: Job_History_View ---

interface job_history_view_props {
    job: Scheduled_Job;
    runs: Scheduled_Job_Run[] | undefined;
}

/**
 * Component: Job_History_View
 * Displays the execution log for a specific job, including cost and mission ID links.
 */
const Job_History_View: React.FC<job_history_view_props> = ({ job, runs }) => {
    return (
        <div className="p-6 ml-12 border-l-2 border-zinc-800 my-4 space-y-4">
            <div className="bg-black/20 rounded p-4 font-mono text-xs text-zinc-400 border border-zinc-900/50">
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
                        <div key={run.id} className="flex items-center justify-between bg-zinc-950 p-2 rounded border border-zinc-800 text-xs font-mono group">
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
}

/**
 * Scheduled_Jobs Page
 * 
 * ### 🗓️ Administrative Interface
 * Manages automated agent tasks and workflow triggers. Features:
 * - **Cron Scheduling**: Standard UNIX cron expression mapping.
 * - **Budget Guard**: Per-job budget enforcement.
 * - **Run History**: Real-time state management of previous execution results.
 */
const Scheduled_Jobs: React.FC = () => {
    const [jobs, set_jobs] = useState<Scheduled_Job[]>([]);
    const [workflows, set_workflows] = useState<(Workflow_Entry & { type: 'continuity' | 'passive' })[]>([]);
    const [is_loading, set_is_loading] = useState(true);
    const [expanded_job, set_expanded_job] = useState<string | null>(null);
    const [runs_map, set_runs_map] = useState<Record<string, Scheduled_Job_Run[]>>({});
    const [confirm_delete, set_confirm_delete] = useState<{ id: string; name: string } | null>(null);

    // Form state
    const [is_creating, set_is_creating] = useState(false);
    const [editing_job_id, set_editing_job_id] = useState<string | null>(null);
    const [job_type, set_job_type] = useState<'agent' | 'workflow'>('agent');
    const [target_search, set_target_search] = useState('');
    const [job_config, set_job_config] = useState<job_config_state>({
        name: '',
        agent_id: '',
        workflow_id: null,
        prompt: '',
        cron_expr: '0 * * * *',
        budget_usd: 0.10,
        max_failures: 3
    });

    const agents = use_agent_store(state => state.agents);

    // Filtered and Sorted Agent List
    const filtered_agents = useMemo(() => {
        if (!Array.isArray(agents)) return [];
        return [...agents]
            .filter(a => 
                a.name.toLowerCase().includes(target_search.toLowerCase()) || 
                a.role.toLowerCase().includes(target_search.toLowerCase())
            )
            .sort((a, b) => a.name.localeCompare(b.name));
    }, [agents, target_search]);

    // Filtered and Sorted Workflow List
    const filtered_workflows = useMemo(() => {
        if (!Array.isArray(workflows)) return [];
        return [...workflows]
            .filter(w => w.name.toLowerCase().includes(target_search.toLowerCase()))
            .sort((a, b) => a.name.localeCompare(b.name));
    }, [workflows, target_search]);



    const fetch_workflows = useCallback(async () => {
        try {
            const [continuity_data, skills_data] = await Promise.all([
                tadpole_os_service.list_continuity_workflows(),
                tadpole_os_service.get_unified_skills()
            ]);

            const continuity_wfs = continuity_data.map(w => ({ ...w, type: 'continuity' as const }));
            const passive_wfs = ((skills_data.workflows || []) as { name: string; content: string }[]).map(w => ({
                id: w.name, 
                name: w.name,
                description: w.content?.slice(0, 100) || '',
                content: w.content,
                created_at: new Date().toISOString(),
                type: 'passive' as const
            }));

            set_workflows([...continuity_wfs, ...passive_wfs]);
        } catch (error: unknown) {
            console.error('Failed to fetch workflows:', error);
        }
    }, []);

    const fetch_jobs = useCallback(async () => {
        try {
            const data = await tadpole_os_service.get_scheduled_jobs();
            set_jobs(data);
        } catch (error: unknown) {
            console.error('Failed to fetch scheduled jobs:', error);
        } finally {
            set_is_loading(false);
        }
    }, []);

    const fetch_runs = useCallback(async (job_id: string) => {
        try {
            const data = await tadpole_os_service.get_scheduled_job_runs(job_id);
            set_runs_map(prev => ({ ...prev, [job_id]: data }));
        } catch (error: unknown) {
            console.error('Failed to fetch runs:', error);
        }
    }, []);

    useEffect(() => {
        void (async () => {
            await Promise.resolve();
            use_agent_store.getState().fetch_agents();
            fetch_workflows();
            fetch_jobs();
        })();
    }, [fetch_jobs, fetch_workflows]);

    const toggle_expand = (job_id: string) => {
        if (expanded_job === job_id) {
            set_expanded_job(null);
        } else {
            set_expanded_job(job_id);
            if (!runs_map[job_id]) {
                fetch_runs(job_id);
            }
        }
    };

    const toggle_enable = useCallback(async (job: Scheduled_Job) => {
        try {
            await tadpole_os_service.update_scheduled_job(job.id, { enabled: !job.enabled });
            event_bus.emit_log({
                source: 'System',
                text: i18n.t(job.enabled ? 'scheduled_jobs.event_disabled' : 'scheduled_jobs.event_enabled', { name: job.name }),
                severity: 'info'
            });
            fetch_jobs();
        } catch (error: unknown) {
            console.error('Failed to toggle job:', error);
        }
    }, [fetch_jobs]);

    const handle_edit = (job: Scheduled_Job) => {
        set_editing_job_id(job.id);
        const type = job.workflow_id ? 'workflow' : 'agent';
        set_job_type(type);
        set_job_config({
            name: job.name,
            agent_id: job.agent_id || '',
            workflow_id: job.workflow_id || null,
            prompt: job.prompt || '',
            cron_expr: job.cron_expr,
            budget_usd: job.budget_usd,
            max_failures: job.max_failures
        });
        set_is_creating(true);
    };

    const delete_job = (id: string, name: string) => {
        set_confirm_delete({ id, name });
    };

    const handle_confirm_delete = async () => {
        if (!confirm_delete) return;
        const { id, name } = confirm_delete;
        
        try {
            await tadpole_os_service.delete_scheduled_job(id);
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('scheduled_jobs.event_terminated', { name: name }),
                severity: 'info'
            });
            set_confirm_delete(null);
            fetch_jobs();
        } catch (error: unknown) {
            console.error('Failed to delete job:', error);
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('scheduled_jobs.event_deletion_failed', { error: error instanceof Error ? error.message : i18n.t('common.unknown_error') }),
                severity: 'error'
            });
            set_confirm_delete(null);
        }
    };

    const handle_job_submit = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            if (editing_job_id) {
                await tadpole_os_service.update_scheduled_job(editing_job_id, job_config);
                event_bus.emit_log({
                    source: 'System',
                    text: i18n.t('scheduled_jobs.event_updated', { name: job_config.name }),
                    severity: 'success'
                });
            } else {
                await tadpole_os_service.create_scheduled_job(job_config);
                event_bus.emit_log({
                    source: 'System',
                    text: i18n.t('scheduled_jobs.event_created', { name: job_config.name }),
                    severity: 'success'
                });
            }
            set_is_creating(false);
            set_editing_job_id(null);
            set_job_config({ name: '', agent_id: '', workflow_id: null, prompt: '', cron_expr: '0 * * * *', budget_usd: 0.10, max_failures: 3 });
            fetch_jobs();
        } catch (error: unknown) { 
            console.error('Failed to handle job:', error);
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('scheduled_jobs.event_failed_action', { action: editing_job_id ? 'update' : 'create' }),
                severity: 'error'
            });
        }
    };

    const get_status_indicator = (job: Scheduled_Job) => {
        if (!job.enabled) return <span className="px-2 py-0.5 rounded text-[10px] bg-zinc-800 text-zinc-500 font-mono">{i18n.t('scheduled_jobs.status_disabled')}</span>;
        if (job.consecutive_failures > 0) return <span className="px-2 py-0.5 rounded text-[10px] bg-amber-500/20 text-amber-500 border border-amber-500/30 font-mono">{i18n.t('scheduled_jobs.status_fails', { count: job.consecutive_failures })}</span>;
        return <span className="px-2 py-0.5 rounded text-[10px] bg-emerald-500/10 text-emerald-500 border border-emerald-500/20 font-mono">{i18n.t('scheduled_jobs.status_active')}</span>;
    };

    if (is_loading) {
        return <div className="p-8 text-zinc-500 flex items-center gap-2"><Clock className="animate-pulse" /> {i18n.t('scheduled_jobs.loading')}</div>;
    }

    return (
        <div className="p-8 space-y-8 min-h-screen bg-zinc-950 text-zinc-100">
            {/* GEO Optimization: Structured Data & Semantic Header */}

            <script type="application/ld+json">
            {JSON.stringify({
              "@context": "https://schema.org",
              "@type": "Service",
              "name": "Tadpole OS Automation Scheduler",
              "description": "Enterprise-grade cron-style scheduler for autonomous agent missions and workflow triggers.",
              "provider": { "@type": "Organization", "name": "Sovereign Engineering" },
              "author": { "@type": "Person", "name": "Agent of Nine" }
            })}
            </script>

            <header className="flex justify-between items-end">
                <Tooltip content={i18n.t('scheduled_jobs.header_tooltip')} position="right">
                    <div>
                        <h1 className="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-white to-zinc-500 cursor-help">
                            {i18n.t('scheduled_jobs.title')}
                        </h1>
                    </div>
                </Tooltip>
                <div className="flex gap-3">
                    <button
                        onClick={() => {
                            set_editing_job_id(null);
                            set_job_config({ name: '', agent_id: '', workflow_id: null, prompt: '', cron_expr: '0 * * * *', budget_usd: 0.10, max_failures: 3 });
                            set_is_creating(true);
                        }}
                        className="px-4 py-2 bg-green-600 hover:bg-green-500 text-white font-bold rounded-lg text-[10px] transition-all flex items-center gap-2 shadow-lg shadow-green-500/20 uppercase tracking-widest"
                    >
                        <Plus size={14} /> {i18n.t('scheduled_jobs.new_job')}
                    </button>
                </div>
            </header>

            {is_creating && (
                <Job_Form_Manager 
                    job_type={job_type}
                    set_job_type={set_job_type}
                    current_job_config={job_config}
                    set_job_config={set_job_config}
                    target_search={target_search}
                    set_target_search={set_target_search}
                    filtered_agents={filtered_agents}
                    filtered_workflows={filtered_workflows}
                    is_creating={is_creating}
                    set_creating={set_is_creating}
                    editing_job_id={editing_job_id}
                    set_editing_job_id={set_editing_job_id}
                    handle_job_submit={handle_job_submit}
                />
            )}

            <div className="bg-zinc-950 border border-zinc-800 rounded-xl overflow-hidden shadow-2xl">
                <table className="w-full text-left text-sm">
                    <thead className="bg-zinc-900 border-b border-zinc-800 text-zinc-400 font-mono text-xs uppercase">
                        <tr>
                            <th className="px-6 py-4 font-medium tracking-wider w-10"></th>
                            <th className="px-6 py-4 font-medium tracking-wider">{i18n.t('scheduled_jobs.th_job_name')}</th>
                            <th className="px-6 py-4 font-medium tracking-wider">{i18n.t('scheduled_jobs.th_target')}</th>
                            <th className="px-6 py-4 font-medium tracking-wider">{i18n.t('scheduled_jobs.th_schedule')}</th>
                            <th className="px-6 py-4 font-medium tracking-wider">{i18n.t('scheduled_jobs.th_next_run')}</th>
                            <th className="px-6 py-4 font-medium tracking-wider">{i18n.t('scheduled_jobs.th_status')}</th>
                            <th className="px-6 py-4 font-medium tracking-wider text-right">{i18n.t('scheduled_jobs.th_actions')}</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-zinc-800">
                        {(Array.isArray(jobs) ? jobs : []).map((job) => {
                            const agent = Array.isArray(agents) ? agents.find(a => a.id === job.agent_id) : undefined;
                            const workflow = Array.isArray(workflows) ? workflows.find(w => w.id === job.workflow_id) : undefined;
                            const is_expanded = expanded_job === job.id;
                            const is_workflow = !!job.workflow_id;

                            return (
                                <React.Fragment key={job.id}>
                                    <tr className={`group hover:bg-zinc-900/50 transition-colors ${is_expanded ? 'bg-zinc-900/50' : ''}`}>
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
                                            <tr className="bg-zinc-900/30">
                                                <td colSpan={7} className="p-0 border-b border-zinc-800">
                                                    <motion.div
                                                        initial={{ height: 0, opacity: 0 }}
                                                        animate={{ height: 'auto', opacity: 1 }}
                                                        exit={{ height: 0, opacity: 0 }}
                                                        className="overflow-hidden"
                                                    >
                                                            <Job_History_View job={job} runs={runs_map[job.id]} />
                                                    </motion.div>
                                                </td>
                                            </tr>
                                        )}
                                    </AnimatePresence>
                                </React.Fragment>
                            );
                        })}
                        {jobs.length === 0 && (
                            <tr>
                                <td colSpan={7} className="px-6 py-12 text-center text-zinc-500 font-mono text-sm">
                                    {i18n.t('scheduled_jobs.no_jobs_deployed')}
                                </td>
                            </tr>
                        )}
                    </tbody>
                </table>
            </div>

            <Confirm_Dialog
                is_open={!!confirm_delete}
                title={i18n.t('scheduled_jobs.confirm_purge_title')}
                message={i18n.t('scheduled_jobs.confirm_purge_message', { name: confirm_delete?.name ?? '' })}
                confirm_label={i18n.t('scheduled_jobs.confirm_purge_button')}
                on_confirm={handle_confirm_delete}
                on_cancel={() => set_confirm_delete(null)}
                variant="danger"
            />
        </div>
    );
};

export default Scheduled_Jobs;


// Metadata: [Scheduled_Jobs]
