/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[useScheduledJobs]` in observability traces.
 */

import { useState, useEffect, useMemo, useCallback } from 'react';
import { tadpole_os_service, type Scheduled_Job, type Scheduled_Job_Run, type Workflow_Entry } from '../services/tadpoleos_service';
import { use_agent_store } from '../stores/agent_store';
import { agent_service } from '../services/agent_service';
import { event_bus } from '../services/event_bus';
import { i18n } from '../i18n';
import type { Agent } from '../types';

export interface Job_Config_State {
    name: string;
    agent_id: string;
    workflow_id: string | null;
    prompt: string;
    cron_expr: string;
    budget_usd: number;
    max_failures: number;
}

export interface UseScheduledJobsHook {
    jobs: Scheduled_Job[];
    workflows: (Workflow_Entry & { type: 'continuity' | 'passive' })[];
    is_loading: boolean;
    expanded_job: string | null;
    runs_map: Record<string, Scheduled_Job_Run[]>;
    confirm_delete: { id: string; name: string } | null;
    
    // Form state
    is_creating: boolean;
    editing_job_id: string | null;
    job_type: 'agent' | 'workflow';
    target_search: string;
    job_config: Job_Config_State;
    
    // Filtered data
    filtered_agents: Agent[];
    filtered_workflows: (Workflow_Entry & { type: 'continuity' | 'passive' })[];
    
    // Actions
    set_is_creating: (creating: boolean) => void;
    set_editing_job_id: (id: string | null) => void;
    set_job_type: (type: 'agent' | 'workflow') => void;
    set_target_search: (search: string) => void;
    set_job_config: React.Dispatch<React.SetStateAction<Job_Config_State>>;
    set_confirm_delete: (confirm: { id: string; name: string } | null) => void;
    
    toggle_expand: (job_id: string) => void;
    toggle_enable: (job: Scheduled_Job) => Promise<void>;
    handle_edit: (job: Scheduled_Job) => void;
    delete_job: (id: string, name: string) => void;
    handle_confirm_delete: () => Promise<void>;
    handle_job_submit: (e: React.FormEvent) => Promise<void>;
    reset_form: () => void;
}

export function useScheduledJobs(): UseScheduledJobsHook {
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
    const [job_config, set_job_config] = useState<Job_Config_State>({
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
            void agent_service.load_agents_into_store();
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

    const reset_form = () => {
        set_job_config({ name: '', agent_id: '', workflow_id: null, prompt: '', cron_expr: '0 * * * *', budget_usd: 0.10, max_failures: 3 });
        set_is_creating(false);
        set_editing_job_id(null);
        set_target_search('');
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
            reset_form();
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

    return {
        jobs,
        workflows,
        is_loading,
        expanded_job,
        runs_map,
        confirm_delete,
        is_creating,
        editing_job_id,
        job_type,
        target_search,
        job_config,
        filtered_agents,
        filtered_workflows,
        set_is_creating,
        set_editing_job_id,
        set_job_type,
        set_target_search,
        set_job_config,
        set_confirm_delete,
        toggle_expand,
        toggle_enable,
        handle_edit,
        delete_job,
        handle_confirm_delete,
        handle_job_submit,
        reset_form
    };
}

// Metadata: [useScheduledJobs]
