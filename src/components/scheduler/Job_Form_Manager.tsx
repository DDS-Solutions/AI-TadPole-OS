/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Job_Form_Manager]` in observability traces.
 */

import React, { useCallback } from 'react';
import { Edit2, Clock } from 'lucide-react';
import { i18n } from '../../i18n';
import { Tooltip } from '../ui';
import type { Job_Config_State } from '../../hooks/useScheduledJobs';
import type { Workflow_Entry } from '../../services/tadpoleos_service';

interface JobFormManagerProps {
    job_type: 'agent' | 'workflow';
    set_job_type: (type: 'agent' | 'workflow') => void;
    current_job_config: Job_Config_State;
    set_job_config: React.Dispatch<React.SetStateAction<Job_Config_State>>;
    target_search: string;
    set_target_search: (search: string) => void;
    filtered_agents: Array<{ id: string; name: string; role: string }>;
    filtered_workflows: Array<Workflow_Entry & { type: 'continuity' | 'passive' }>;
    editing_job_id: string | null;
    on_cancel: () => void;
    handle_job_submit: (e: React.FormEvent) => Promise<void>;
}

export const Job_Form_Manager: React.FC<JobFormManagerProps> = ({ 
    job_type, 
    set_job_type,
    current_job_config, 
    set_job_config, 
    target_search, 
    set_target_search, 
    filtered_agents, 
    filtered_workflows, 
    editing_job_id,
    on_cancel,
    handle_job_submit
}) => {
    const handle_input_change = useCallback((key: keyof Job_Config_State, value: string | number | boolean | string[] | null) => {
        set_job_config((prev: Job_Config_State) => ({ ...prev, [key]: value }));
    }, [set_job_config]);

    return (
        <div className="bg-[color:var(--color-surface)] border border-green-500/30 rounded-xl p-6 shadow-2xl max-h-[85vh] overflow-y-auto custom-scrollbar mb-8">
            <div className="flex justify-between items-center mb-4 sticky top-0 bg-[color:var(--color-surface)] z-10 pb-2">
                <h2 className="text-lg font-bold text-green-400 font-mono uppercase">
                    {editing_job_id ? i18n.t('scheduled_jobs.modify_config') : i18n.t('scheduled_jobs.configure_new')}
                </h2>
                <div className="flex bg-[color:var(--color-background)] p-1 rounded-lg border border-[color:var(--color-border)]">
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
                <div className="space-y-1.5">
                    <Tooltip content={i18n.t('scheduled_jobs.job_name_tooltip')} position="top">
                        <label htmlFor="job_name" className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest cursor-help flex items-center gap-2">
                            <Edit2 size={10} className="text-green-500/50" />
                            {i18n.t('scheduled_jobs.job_name')}
                        </label>
                    </Tooltip>
                    <input 
                        id="job_name" required 
                        type="text" 
                        className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg p-2.5 text-sm focus:border-green-500/50 transition-all outline-none" 
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
                                        className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded p-1.5 text-[10px] font-mono focus:border-green-500/50 outline-none"
                                        value={target_search}
                                        onChange={e => set_target_search(e.target.value)}
                                    />
                                    <select 
                                        id="target_agent" 
                                        required 
                                        className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded p-2 text-sm" 
                                        value={current_job_config.agent_id || ''} 
                                        onChange={e => {
                                            handle_input_change('agent_id', e.target.value);
                                            handle_input_change('workflow_id', null);
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
                                        className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded p-1.5 text-[10px] font-mono focus:border-green-500/50 outline-none"
                                        value={target_search}
                                        onChange={(e) => set_target_search(e.target.value)}
                                    />
                                    <select 
                                        id="target_workflow" 
                                        required 
                                        className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded p-2 text-sm" 
                                        value={current_job_config.workflow_id || ''} 
                                        onChange={e => {
                                            handle_input_change('workflow_id', e.target.value);
                                            handle_input_change('agent_id', '');
                                        }}
                                    >
                                        <option value="">{i18n.t('scheduled_jobs.select_workflow')}</option>
                                        <optgroup label={i18n.t('scheduled_jobs.mission_sequences')} className="bg-[color:var(--color-surface)] text-green-400 font-mono text-[10px]">
                                            {filtered_workflows.filter(w => w.type === 'continuity').map(w => (
                                                <option key={w.id} value={w.id} className="text-zinc-300 bg-[color:var(--color-background)]">{w.name}</option>
                                            ))}
                                        </optgroup>
                                        <optgroup label={i18n.t('scheduled_jobs.guiding_protocols')} className="bg-[color:var(--color-surface)] text-amber-400 font-mono text-[10px]">
                                            {filtered_workflows.filter(w => w.type === 'passive').map(w => (
                                                <option key={w.id} value={w.id} className="text-zinc-300 bg-[color:var(--color-background)]">{w.name}</option>
                                            ))}
                                        </optgroup>
                                    </select>
                                </div>
                            </>
                        )}
                    </div>
                </div>

                {/* Cron Expression */}
                <div className="space-y-2">
                    <Tooltip content={i18n.t('scheduled_jobs.cron_tooltip')} position="top">
                        <label htmlFor="cron_expr" className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest cursor-help flex items-center gap-2">
                            <Clock size={10} className="text-green-500/50" />
                            {i18n.t('scheduled_jobs.cron_expression')}
                        </label>
                    </Tooltip>
                    <div className="space-y-2">
                        <input 
                            id="cron_expr" required 
                            type="text" 
                            className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg p-2.5 text-sm font-mono focus:border-green-500/50 outline-none" 
                            value={current_job_config.cron_expr} 
                            onChange={e => handle_input_change('cron_expr', e.target.value)} 
                            placeholder={i18n.t('scheduled_jobs.cron_placeholder')} 
                        />
                        <div className="flex flex-wrap gap-1.5">
                            {[
                                { label: i18n.t('scheduled_jobs.preset_hourly'), val: '0 * * * *' },
                                { label: i18n.t('scheduled_jobs.preset_daily'), val: '0 0 * * *' },
                                { label: i18n.t('scheduled_jobs.preset_weekly'), val: '0 0 * * 0' },
                                { label: i18n.t('scheduled_jobs.preset_monthly'), val: '0 0 1 * *' }
                            ].map(preset => (
                                <button
                                    key={preset.val}
                                    type="button"
                                    onClick={() => handle_input_change('cron_expr', preset.val)}
                                    className="px-2 py-1 bg-zinc-800/50 hover:bg-zinc-700 text-[9px] font-bold text-zinc-400 rounded border border-zinc-700/50 transition-colors uppercase tracking-tighter"
                                >
                                    {preset.label}
                                </button>
                            ))}
                        </div>
                    </div>
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
                            className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded p-2 text-sm resize-none custom-scrollbar" 
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
                            className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded p-2 text-sm" 
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
                            className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded p-2 text-sm" 
                            value={current_job_config.max_failures} 
                            onChange={e => handle_input_change('max_failures', parseInt(e.target.value, 10) || 0)} 
                        />
                    </div>
                </div>
                
                {/* Submission Buttons */}
                <div className="flex gap-2 pt-2 border-t border-[color:var(--color-border)]">
                    <button type="submit" className="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 rounded text-sm font-bold shadow-lg shadow-emerald-500/20">
                        {editing_job_id ? i18n.t('scheduled_jobs.update_sync') : i18n.t('scheduled_jobs.save_job')}
                    </button>
                    <button type="button" onClick={on_cancel} className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded text-sm text-zinc-300">
                        {i18n.t('scheduled_jobs.cancel')}
                    </button>
                </div>
            </form>
        </div>
    );
};

// Metadata: [Job_Form_Manager]
