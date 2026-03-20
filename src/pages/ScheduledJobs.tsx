import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Clock, Play, Pause, Trash2, ChevronRight, Activity, CheckCircle2, AlertCircle, Plus, Edit2 } from 'lucide-react';
import { TadpoleOSService, type ScheduledJob, type ScheduledJobRun, type WorkflowEntry } from '../services/tadpoleosService';
import { useAgentStore } from '../stores/agentStore';
import { EventBus } from '../services/eventBus';
import { Tooltip } from '../components/ui';
import { ConfirmDialog } from '../components/ui/ConfirmDialog';
import { i18n } from '../i18n';

const ScheduledJobs: React.FC = () => {
    const [jobs, setJobs] = useState<ScheduledJob[]>([]);
    const [workflows, setWorkflows] = useState<(WorkflowEntry & { type: 'continuity' | 'passive' })[]>([]);
    const [loading, setLoading] = useState(true);
    const [expandedJob, setExpandedJob] = useState<string | null>(null);
    const [runsMap, setRunsMap] = useState<Record<string, ScheduledJobRun[]>>({});
    const [confirmDelete, setConfirmDelete] = useState<{ id: string; name: string } | null>(null);

    // Form state
    const [isCreating, setIsCreating] = useState(false);
    const [editingJobId, setEditingJobId] = useState<string | null>(null);
    const [jobType, setJobType] = useState<'agent' | 'workflow'>('agent');
    const [targetSearch, setTargetSearch] = useState('');
    const [newJob, setNewJob] = useState<Partial<ScheduledJob>>({
        name: '',
        agent_id: '',
        workflow_id: null,
        prompt: '',
        cron_expr: '0 * * * *',
        budget_usd: 0.10,
        max_failures: 3
    });

    const agents = useAgentStore(state => state.agents);

    // Filtered and Sorted Agent List
    const filteredAgents = React.useMemo(() => {
        if (!Array.isArray(agents)) return [];
        return [...agents]
            .filter(a => 
                a.name.toLowerCase().includes(targetSearch.toLowerCase()) || 
                a.role.toLowerCase().includes(targetSearch.toLowerCase())
            )
            .sort((a, b) => a.name.localeCompare(b.name));
    }, [agents, targetSearch]);

    // Filtered and Sorted Workflow List
    const filteredWorkflows = React.useMemo(() => {
        if (!Array.isArray(workflows)) return [];
        return [...workflows]
            .filter(w => w.name.toLowerCase().includes(targetSearch.toLowerCase()))
            .sort((a, b) => a.name.localeCompare(b.name));
    }, [workflows, targetSearch]);

    useEffect(() => {
        useAgentStore.getState().fetchAgents();
        fetchJobs();
        fetchWorkflows();
    }, []);

    const fetchWorkflows = async () => {
        try {
            const [continuityData, capabilitiesData] = await Promise.all([
                TadpoleOSService.listContinuityWorkflows(),
                TadpoleOSService.getCapabilities()
            ]);

            const continuityWfs = continuityData.map(w => ({ ...w, type: 'continuity' as const }));
            const passiveWfs = ((capabilitiesData.workflows || []) as { name: string; content: string }[]).map(w => ({
                id: w.name, // Passive workflows use name as ID in capabilities
                name: w.name,
                description: w.content?.slice(0, 100),
                created_at: new Date().toISOString(),
                type: 'passive' as const
            }));

            setWorkflows([...continuityWfs, ...passiveWfs]);
        } catch (error: unknown) {
            console.error('Failed to fetch workflows:', error);
        }
    };

    const fetchJobs = async () => {
        try {
            const data = await TadpoleOSService.getScheduledJobs();
            setJobs(data);
        } catch (error: unknown) {
            console.error('Failed to fetch scheduled jobs:', error);
        } finally {
            setLoading(false);
        }
    };

    const fetchRuns = async (jobId: string) => {
        try {
            const data = await TadpoleOSService.getScheduledJobRuns(jobId);
            setRunsMap(prev => ({ ...prev, [jobId]: data }));
        } catch (error: unknown) {
            console.error('Failed to fetch runs:', error);
        }
    };

    const toggleExpand = (jobId: string) => {
        if (expandedJob === jobId) {
            setExpandedJob(null);
        } else {
            setExpandedJob(jobId);
            if (!runsMap[jobId]) {
                fetchRuns(jobId);
            }
        }
    };

    const toggleEnable = async (job: ScheduledJob) => {
        try {
            await TadpoleOSService.updateScheduledJob(job.id, { enabled: !job.enabled });
            EventBus.emit({
                source: 'System',
                text: i18n.t(job.enabled ? 'scheduled_jobs.event_disabled' : 'scheduled_jobs.event_enabled', { name: job.name }),
                severity: 'info'
            });
            fetchJobs();
        } catch (error: unknown) {
            console.error('Failed to toggle job:', error);
        }
    };

    const handleEdit = (job: ScheduledJob) => {
        setEditingJobId(job.id);
        setJobType(job.workflow_id ? 'workflow' : 'agent');
        setNewJob({
            name: job.name,
            agent_id: job.agent_id,
            workflow_id: job.workflow_id,
            prompt: job.prompt || '',
            cron_expr: job.cron_expr,
            budget_usd: job.budget_usd,
            max_failures: job.max_failures
        });
        setIsCreating(true);
    };

    const deleteJob = (id: string, name: string) => {
        setConfirmDelete({ id, name });
    };

    const handleConfirmDelete = async () => {
        if (!confirmDelete) return;
        const { id, name } = confirmDelete;
        
        try {
            await TadpoleOSService.deleteScheduledJob(id);
            EventBus.emit({
                source: 'System',
                text: i18n.t('scheduled_jobs.event_terminated', { name: name }),
                severity: 'info'
            });
            setConfirmDelete(null);
            fetchJobs();
        } catch (error: unknown) {
            console.error('Failed to delete job:', error);
            EventBus.emit({
                source: 'System',
                text: i18n.t('scheduled_jobs.event_deletion_failed', { error: error instanceof Error ? error.message : i18n.t('common.unknown_error') }),
                severity: 'error'
            });
            setConfirmDelete(null);
        }
    };

    const handleCreate = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            if (editingJobId) {
                await TadpoleOSService.updateScheduledJob(editingJobId, newJob);
                EventBus.emit({
                    source: 'System',
                    text: i18n.t('scheduled_jobs.event_updated', { name: newJob.name ?? '' }),
                    severity: 'success'
                });
            } else {
                await TadpoleOSService.createScheduledJob(newJob);
                EventBus.emit({
                    source: 'System',
                    text: i18n.t('scheduled_jobs.event_created', { name: newJob.name ?? '' }),
                    severity: 'success'
                });
            }
            setIsCreating(false);
            setEditingJobId(null);
            setNewJob({ name: '', agent_id: '', workflow_id: null, prompt: '', cron_expr: '0 * * * *', budget_usd: 0.10, max_failures: 3 });
            fetchJobs();
        } catch (error: unknown) { 
            console.error('Failed to handle job:', error);
            EventBus.emit({
                source: 'System',
                text: i18n.t('scheduled_jobs.event_failed_action', { action: editingJobId ? 'update' : 'create' }),
                severity: 'error'
            });
        }
    };

    const getStatusIndicator = (job: ScheduledJob) => {
        if (!job.enabled) return <span className="px-2 py-0.5 rounded text-[10px] bg-zinc-800 text-zinc-500 font-mono">{i18n.t('scheduled_jobs.status_disabled')}</span>;
        if (job.consecutive_failures > 0) return <span className="px-2 py-0.5 rounded text-[10px] bg-amber-500/20 text-amber-500 border border-amber-500/30 font-mono">{i18n.t('scheduled_jobs.status_fails', { count: job.consecutive_failures })}</span>;
        return <span className="px-2 py-0.5 rounded text-[10px] bg-emerald-500/10 text-emerald-500 border border-emerald-500/20 font-mono">{i18n.t('scheduled_jobs.status_active')}</span>;
    };

    if (loading) {
        return <div className="p-8 text-zinc-500 flex items-center gap-2"><Clock className="animate-pulse" /> {i18n.t('scheduled_jobs.loading')}</div>;
    }

    return (
        <div className="p-8 space-y-8 min-h-screen bg-zinc-950 text-zinc-100">
            <header className="flex justify-between items-end">
                <Tooltip content={i18n.t('scheduled_jobs.header_tooltip')} position="right">
                    <div>
                        <h1 className="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-white to-zinc-500 cursor-help">
                            {i18n.t('scheduled_jobs.title')}
                        </h1>
                        <div className="mt-1">
                        </div>
                    </div>
                </Tooltip>
                <div className="flex gap-3">
                    <button
                        onClick={() => {
                            setEditingJobId(null);
                            setNewJob({ name: '', agent_id: '', workflow_id: null, prompt: '', cron_expr: '0 * * * *', budget_usd: 0.10, max_failures: 3 });
                            setIsCreating(true);
                        }}
                        className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white font-bold rounded-lg text-[10px] transition-all flex items-center gap-2 shadow-lg shadow-blue-500/20 uppercase tracking-widest"
                    >
                        <Plus size={14} /> {i18n.t('scheduled_jobs.new_job')}
                    </button>
                </div>
            </header>

            {isCreating && (
                <div className="bg-zinc-900 border border-blue-500/30 rounded-xl p-6 shadow-2xl max-h-[85vh] overflow-y-auto custom-scrollbar">
                    <div className="flex justify-between items-center mb-4 sticky top-0 bg-zinc-900 z-10 pb-2">
                        <h2 className="text-lg font-bold text-blue-400 font-mono uppercase">
                            {editingJobId ? i18n.t('scheduled_jobs.modify_config') : i18n.t('scheduled_jobs.configure_new')}
                        </h2>
                        <div className="flex bg-zinc-950 p-1 rounded-lg border border-zinc-800">
                            <button
                                onClick={() => { setJobType('agent'); setTargetSearch(''); }}
                                className={`px-3 py-1 text-[10px] font-bold rounded ${jobType === 'agent' ? 'bg-blue-600 text-white' : 'text-zinc-500 hover:text-zinc-300'}`}
                            >
                                {i18n.t('scheduled_jobs.single_agent')}
                            </button>
                            <button
                                onClick={() => { setJobType('workflow'); setTargetSearch(''); }}
                                className={`px-3 py-1 text-[10px] font-bold rounded ${jobType === 'workflow' ? 'bg-blue-600 text-white' : 'text-zinc-500 hover:text-zinc-300'}`}
                            >
                                {i18n.t('scheduled_jobs.multi_step_workflow')}
                            </button>
                        </div>
                    </div>
                    <form onSubmit={handleCreate} className="space-y-4">
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 min-w-0">
                            <div className="min-w-0">
                                <Tooltip content={i18n.t('scheduled_jobs.job_name_tooltip')} position="top">
                                    <label htmlFor="jobName" className="block text-xs font-mono text-zinc-500 mb-1 cursor-help">{i18n.t('scheduled_jobs.job_name')}</label>
                                </Tooltip>
                                <input id="jobName" required type="text" className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-sm" value={newJob.name} onChange={e => setNewJob({ ...newJob, name: e.target.value })} placeholder={i18n.t('scheduled_jobs.job_name_placeholder')} />
                            </div>
                            <div className="min-w-0">
                                {jobType === 'agent' ? (
                                    <>
                                        <Tooltip content={i18n.t('scheduled_jobs.target_agent_tooltip')} position="top">
                                            <label htmlFor="targetAgent" className="block text-xs font-mono text-zinc-500 mb-1 cursor-help">{i18n.t('scheduled_jobs.target_agent')}</label>
                                        </Tooltip>
                                        <div className="space-y-2">
                                            <input 
                                                type="text" 
                                                placeholder={i18n.t('scheduled_jobs.filter_agents')} 
                                                className="w-full bg-zinc-950 border border-zinc-800 rounded p-1.5 text-[10px] font-mono focus:border-blue-500/50 outline-none"
                                                value={targetSearch}
                                                onChange={e => setTargetSearch(e.target.value)}
                                            />
                                            <select id="targetAgent" required className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-sm" value={newJob.agent_id} onChange={e => setNewJob({ ...newJob, agent_id: e.target.value, workflow_id: null })}>
                                                <option value="">{i18n.t('scheduled_jobs.select_agent')}</option>
                                                {filteredAgents.map(a => <option key={a.id} value={a.id}>{a.name} ({a.role})</option>)}
                                            </select>
                                        </div>
                                    </>
                                ) : (
                                    <>
                                        <Tooltip content={i18n.t('scheduled_jobs.target_workflow_tooltip')} position="top">
                                            <label htmlFor="targetWorkflow" className="block text-xs font-mono text-zinc-500 mb-1 cursor-help">{i18n.t('scheduled_jobs.target_workflow')}</label>
                                        </Tooltip>
                                        <div className="space-y-2">
                                            <input 
                                                type="text" 
                                                placeholder={i18n.t('scheduled_jobs.filter_workflows')} 
                                                className="w-full bg-zinc-950 border border-zinc-800 rounded p-1.5 text-[10px] font-mono focus:border-blue-500/50 outline-none"
                                                value={targetSearch}
                                                onChange={(e) => setTargetSearch(e.target.value)}
                                            />
                                            <select id="targetWorkflow" required className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-sm" value={newJob.workflow_id || ''} onChange={e => setNewJob({ ...newJob, workflow_id: e.target.value, agent_id: '' })}>
                                                <option value="">{i18n.t('scheduled_jobs.select_workflow')}</option>
                                                <optgroup label={i18n.t('scheduled_jobs.mission_sequences')} className="bg-zinc-900 text-blue-400 font-mono text-[10px]">
                                                    {filteredWorkflows.filter(w => w.type === 'continuity').map(w => (
                                                        <option key={w.id} value={w.id} className="text-zinc-300 bg-zinc-950">{w.name}</option>
                                                    ))}
                                                </optgroup>
                                                <optgroup label={i18n.t('scheduled_jobs.guiding_protocols')} className="bg-zinc-900 text-amber-400 font-mono text-[10px]">
                                                    {filteredWorkflows.filter(w => w.type === 'passive').map(w => (
                                                        <option key={w.id} value={w.id} className="text-zinc-300 bg-zinc-950">{w.name}</option>
                                                    ))}
                                                </optgroup>
                                            </select>
                                        </div>
                                    </>
                                )}
                            </div>
                        </div>
                        <div>
                            <Tooltip content={i18n.t('scheduled_jobs.cron_tooltip')} position="top">
                                <label htmlFor="cronExpr" className="block text-xs font-mono text-zinc-500 mb-1 cursor-help">{i18n.t('scheduled_jobs.cron_expression')}</label>
                            </Tooltip>
                            <input id="cronExpr" required type="text" className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-sm font-mono" value={newJob.cron_expr} onChange={e => setNewJob({ ...newJob, cron_expr: e.target.value })} placeholder={i18n.t('scheduled_jobs.cron_placeholder')} />
                        </div>
                        {jobType === 'agent' && (
                            <div>
                                <Tooltip content={i18n.t('scheduled_jobs.mission_prompt_tooltip')} position="top">
                                    <label htmlFor="missionPrompt" className="block text-xs font-mono text-zinc-500 mb-1 cursor-help">{i18n.t('scheduled_jobs.mission_prompt')}</label>
                                </Tooltip>
                                <textarea id="missionPrompt" required rows={3} className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-sm resize-none custom-scrollbar" value={newJob.prompt} onChange={e => setNewJob({ ...newJob, prompt: e.target.value })} placeholder={i18n.t('scheduled_jobs.mission_prompt_placeholder')} />
                            </div>
                        )}
                        <div className="grid grid-cols-2 gap-4">
                            <div>
                                <Tooltip content={i18n.t('scheduled_jobs.budget_cap_tooltip')} position="top">
                                    <label htmlFor="budgetUsd" className="block text-xs font-mono text-zinc-500 mb-1 cursor-help">{i18n.t('scheduled_jobs.budget_cap')}</label>
                                </Tooltip>
                                <input id="budgetUsd" type="number" step="0.01" className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-sm" value={newJob.budget_usd} onChange={e => setNewJob({ ...newJob, budget_usd: parseFloat(e.target.value) })} />
                            </div>
                            <div>
                                <Tooltip content={i18n.t('scheduled_jobs.max_failures_tooltip')} position="top">
                                    <label htmlFor="maxFailures" className="block text-xs font-mono text-zinc-500 mb-1 cursor-help">{i18n.t('scheduled_jobs.max_failures_label')}</label>
                                </Tooltip>
                                <input id="maxFailures" type="number" className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-sm" value={newJob.max_failures} onChange={e => setNewJob({ ...newJob, max_failures: parseInt(e.target.value, 10) })} />
                            </div>
                        </div>
                        <div className="flex gap-2 pt-2 border-t border-zinc-800">
                            <button type="submit" className="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 rounded text-sm font-bold shadow-lg shadow-emerald-500/20">
                                {editingJobId ? i18n.t('scheduled_jobs.update_sync') : i18n.t('scheduled_jobs.save_job')}
                            </button>
                            <button type="button" onClick={() => {
                                setIsCreating(false);
                                setEditingJobId(null);
                            }} className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded text-sm text-zinc-300">{i18n.t('scheduled_jobs.cancel')}</button>
                        </div>
                    </form>
                </div>
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
                            const isExpanded = expandedJob === job.id;
                            const isWorkflow = !!job.workflow_id;

                            return (
                                <React.Fragment key={job.id}>
                                    <tr className={`group hover:bg-zinc-900/50 transition-colors ${isExpanded ? 'bg-zinc-900/50' : ''}`}>
                                        <td className="px-6 py-4 w-10">
                                            <button onClick={() => toggleExpand(job.id)} className="text-zinc-500 hover:text-white transition-colors">
                                                <ChevronRight size={16} className={`transform transition-transform ${isExpanded ? 'rotate-90' : ''}`} />
                                            </button>
                                        </td>
                                        <td className="px-6 py-4 whitespace-nowrap">
                                            <div className="font-semibold text-zinc-100">{job.name}</div>
                                            <div className="text-[10px] text-zinc-500 font-mono mt-1">{job.id.slice(0, 8)}</div>
                                        </td>
                                        <td className="px-6 py-4 whitespace-nowrap">
                                            <div className="flex items-center gap-2">
                                                {isWorkflow ? (
                                                    <>
                                                        <div className="w-5 h-5 rounded overflow-hidden bg-blue-500/20 flex items-center justify-center border border-blue-500/30">
                                                            <Activity size={10} className="text-blue-400" />
                                                        </div>
                                                        <span className="text-blue-400 font-bold font-mono text-xs italic tracking-tighter">
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
                                        <td className="px-6 py-4 whitespace-nowrap text-blue-400 font-mono text-xs font-bold">{job.cron_expr}</td>
                                        <td className="px-6 py-4 whitespace-nowrap text-zinc-400 font-mono text-xs">
                                            {job.enabled ? new Date(job.next_run_at).toLocaleString() : '-'}
                                        </td>
                                        <td className="px-6 py-4 whitespace-nowrap">
                                            {getStatusIndicator(job)}
                                        </td>
                                        <td className="px-6 py-4 whitespace-nowrap space-x-2 text-right">
                                            <Tooltip content={job.enabled ? i18n.t('scheduled_jobs.pause_execution') : i18n.t('scheduled_jobs.resume_execution')} position="top">
                                                <button onClick={() => toggleEnable(job)} className="p-1.5 text-zinc-500 hover:text-white rounded bg-zinc-800 hover:bg-zinc-700 transition-colors">
                                                    {job.enabled ? <Pause size={14} /> : <Play size={14} />}
                                                </button>
                                            </Tooltip>
                                            <Tooltip content={i18n.t('scheduled_jobs.modify_config')} position="top">
                                                <button onClick={() => handleEdit(job)} className="p-1.5 text-zinc-500 hover:text-blue-400 rounded bg-zinc-800 hover:bg-zinc-700 transition-colors">
                                                    <Edit2 size={14} />
                                                </button>
                                            </Tooltip>
                                            <Tooltip content={i18n.t('scheduled_jobs.purge_job_tooltip')} position="top">
                                                 <button 
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        deleteJob(job.id, job.name);
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
                                        {isExpanded && (
                                            <tr className="bg-zinc-900/30">
                                                <td colSpan={7} className="p-0 border-b border-zinc-800">
                                                    <motion.div
                                                        initial={{ height: 0, opacity: 0 }}
                                                        animate={{ height: 'auto', opacity: 1 }}
                                                        exit={{ height: 0, opacity: 0 }}
                                                        className="overflow-hidden"
                                                    >
                                                        <div className="p-6 ml-12 border-l-2 border-zinc-800 my-4 space-y-4">
                                                            <div className="bg-black/20 rounded p-4 font-mono text-xs text-zinc-400 border border-zinc-900/50">
                                                                <span className="text-zinc-600 block mb-2 uppercase tracking-widest text-[9px]">{i18n.t('scheduled_jobs.mission_prompt')}</span>
                                                                {job.prompt}
                                                            </div>
 
                                                            <h4 className="font-mono text-[10px] text-zinc-500 uppercase tracking-widest pt-2">{i18n.t('scheduled_jobs.run_history')}</h4>
                                                            {!runsMap[job.id] ? (
                                                                <div className="text-zinc-600 text-xs flex items-center gap-2"><Activity className="animate-pulse" size={14} /> {i18n.t('scheduled_jobs.fetching_history')}</div>
                                                            ) : runsMap[job.id].length === 0 ? (
                                                                <div className="text-zinc-600 text-xs italic">{i18n.t('scheduled_jobs.no_history')}</div>
                                                            ) : (
                                                                <div className="space-y-2">
                                                                    {runsMap[job.id].map(run => (
                                                                        <div key={run.id} className="flex items-center justify-between bg-zinc-950 p-2 rounded border border-zinc-800 text-xs font-mono group">
                                                                            <div className="flex items-center gap-4">
                                                                                {run.status === 'completed' ? <CheckCircle2 size={14} className="text-emerald-500" /> : <AlertCircle size={14} className="text-rose-500" />}
                                                                                <span className="text-zinc-400">{new Date(run.started_at).toLocaleString()}</span>
                                                                                <span className={`${run.status === 'completed' ? 'text-emerald-400' : 'text-rose-400'}`}>
                                                                                    {run.status.toUpperCase()}
                                                                                </span>
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

            <ConfirmDialog
                isOpen={!!confirmDelete}
                title={i18n.t('scheduled_jobs.confirm_purge_title')}
                message={i18n.t('scheduled_jobs.confirm_purge_message', { name: confirmDelete?.name ?? '' })}
                confirmLabel={i18n.t('scheduled_jobs.confirm_purge_button')}
                onConfirm={handleConfirmDelete}
                onCancel={() => setConfirmDelete(null)}
                variant="danger"
            />
        </div>
    );
};

export default ScheduledJobs;
