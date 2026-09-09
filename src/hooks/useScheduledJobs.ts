/**
 * @docs ARCHITECTURE:UI-Hooks
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useScheduledJobs
 * - **Primary Entrypoints**: `useScheduledJobs`, `Job_Config_State`, `UseScheduledJobsHook`, `ScheduledJobsState`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useReducer, useEffect, useMemo, useCallback } from 'react';
import { system_api_service, type Scheduled_Job, type Scheduled_Job_Run, type Workflow_Entry } from '../services/system_api_service';
import { mission_api_service } from '../services/mission_api_service';
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
    set_confirm_delete: React.Dispatch<React.SetStateAction<{ id: string; name: string } | null>>;
    
    toggle_expand: (job_id: string) => void;
    toggle_enable: (job: Scheduled_Job) => Promise<void>;
    handle_edit: (job: Scheduled_Job) => void;
    delete_job: (id: string, name: string) => void;
    handle_confirm_delete: () => Promise<void>;
    handle_job_submit: (e: React.FormEvent) => Promise<void>;
    reset_form: () => void;
}

export interface ScheduledJobsState {
    jobs: Scheduled_Job[];
    workflows: (Workflow_Entry & { type: 'continuity' | 'passive' })[];
    is_loading: boolean;
    expanded_job: string | null;
    runs_map: Record<string, Scheduled_Job_Run[]>;
    confirm_delete: { id: string; name: string } | null;
    is_creating: boolean;
    editing_job_id: string | null;
    job_type: 'agent' | 'workflow';
    target_search: string;
    job_config: Job_Config_State;
}

const initial_job_config: Job_Config_State = {
    name: '',
    agent_id: '',
    workflow_id: null,
    prompt: '',
    cron_expr: '0 * * * *',
    budget_usd: 0.10,
    max_failures: 3
};

const initial_state: ScheduledJobsState = {
    jobs: [],
    workflows: [],
    is_loading: true,
    expanded_job: null,
    runs_map: {},
    confirm_delete: null,
    is_creating: false,
    editing_job_id: null,
    job_type: 'agent',
    target_search: '',
    job_config: initial_job_config
};

type Action =
    | { type: 'SET_LOADING'; value: boolean }
    | { type: 'SET_JOBS'; value: Scheduled_Job[] }
    | { type: 'SET_WORKFLOWS'; value: (Workflow_Entry & { type: 'continuity' | 'passive' })[] }
    | { type: 'SET_EXPANDED_JOB'; value: string | null }
    | { type: 'SET_IS_CREATING'; value: boolean }
    | { type: 'SET_EDITING_JOB_ID'; value: string | null }
    | { type: 'SET_JOB_TYPE'; value: 'agent' | 'workflow' }
    | { type: 'SET_TARGET_SEARCH'; value: string }
    | { type: 'SET_JOB_CONFIG'; value: React.SetStateAction<Job_Config_State> }
    | { type: 'SET_CONFIRM_DELETE'; value: React.SetStateAction<{ id: string; name: string } | null> }
    | { type: 'SET_RUNS'; job_id: string; runs: Scheduled_Job_Run[] }
    | { type: 'OPEN_EDIT_FORM'; job: Scheduled_Job }
    | { type: 'RESET_FORM' };

function scheduled_jobs_reducer(state: ScheduledJobsState, action: Action): ScheduledJobsState {
    switch (action.type) {
        case 'SET_LOADING':
            return { ...state, is_loading: action.value };
        case 'SET_JOBS':
            return { ...state, jobs: action.value };
        case 'SET_WORKFLOWS':
            return { ...state, workflows: action.value };
        case 'SET_EXPANDED_JOB':
            return { ...state, expanded_job: action.value };
        case 'SET_IS_CREATING':
            return { ...state, is_creating: action.value };
        case 'SET_EDITING_JOB_ID':
            return { ...state, editing_job_id: action.value };
        case 'SET_JOB_TYPE':
            return { ...state, job_type: action.value };
        case 'SET_TARGET_SEARCH':
            return { ...state, target_search: action.value };
        case 'SET_JOB_CONFIG':
            return {
                ...state,
                job_config: typeof action.value === 'function' ? action.value(state.job_config) : action.value
            };
        case 'SET_CONFIRM_DELETE':
            return {
                ...state,
                confirm_delete: typeof action.value === 'function' ? action.value(state.confirm_delete) : action.value
            };
        case 'SET_RUNS':
            return {
                ...state,
                runs_map: { ...state.runs_map, [action.job_id]: action.runs }
            };
        case 'OPEN_EDIT_FORM': {
            const type = action.job.workflow_id ? 'workflow' : 'agent';
            return {
                ...state,
                editing_job_id: action.job.id,
                job_type: type,
                is_creating: true,
                job_config: {
                    name: action.job.name,
                    agent_id: action.job.agent_id || '',
                    workflow_id: action.job.workflow_id || null,
                    prompt: action.job.prompt || '',
                    cron_expr: action.job.cron_expr,
                    budget_usd: action.job.budget_usd,
                    max_failures: action.job.max_failures
                }
            };
        }
        case 'RESET_FORM':
            return {
                ...state,
                job_config: initial_job_config,
                is_creating: false,
                editing_job_id: null,
                target_search: ''
            };
        default:
            return state;
    }
}

export function useScheduledJobs(): UseScheduledJobsHook {
    const [state, dispatch] = useReducer(scheduled_jobs_reducer, initial_state);
    const agents = use_agent_store(s => s.agents);

    // Strictly-typed setters with stable references
    const set_is_creating = useCallback((value: boolean) => dispatch({ type: 'SET_IS_CREATING', value }), []);
    const set_editing_job_id = useCallback((value: string | null) => dispatch({ type: 'SET_EDITING_JOB_ID', value }), []);
    const set_job_type = useCallback((value: 'agent' | 'workflow') => dispatch({ type: 'SET_JOB_TYPE', value }), []);
    const set_target_search = useCallback((value: string) => dispatch({ type: 'SET_TARGET_SEARCH', value }), []);
    const set_confirm_delete = useCallback((value: React.SetStateAction<{ id: string; name: string } | null>) => {
        dispatch({ type: 'SET_CONFIRM_DELETE', value });
    }, []);
    const set_job_config = useCallback((value: React.SetStateAction<Job_Config_State>) => {
        dispatch({ type: 'SET_JOB_CONFIG', value });
    }, []);

    // Filtered and Sorted Agent List (memoized)
    const filtered_agents = useMemo(() => {
        if (!Array.isArray(agents)) return [];
        return [...agents]
            .filter(a => 
                a.name.toLowerCase().includes(state.target_search.toLowerCase()) || 
                a.role.toLowerCase().includes(state.target_search.toLowerCase())
            )
            .sort((a, b) => a.name.localeCompare(b.name));
    }, [agents, state.target_search]);

    // Filtered and Sorted Workflow List (memoized)
    const filtered_workflows = useMemo(() => {
        if (!Array.isArray(state.workflows)) return [];
        return [...state.workflows]
            .filter(w => w.name.toLowerCase().includes(state.target_search.toLowerCase()))
            .sort((a, b) => a.name.localeCompare(b.name));
    }, [state.workflows, state.target_search]);

    const fetch_workflows = useCallback(async () => {
        try {
            const [continuity_data, skills_data] = await Promise.all([
                system_api_service.continuity.list_continuity_workflows(),
                mission_api_service.get_unified_skills()
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

            dispatch({
                type: 'SET_WORKFLOWS',
                value: [...continuity_wfs, ...passive_wfs]
            });
        } catch (error: unknown) {
            console.error('Failed to fetch workflows:', error);
        }
    }, []);

    const fetch_jobs = useCallback(async () => {
        try {
            const data = await system_api_service.continuity.get_scheduled_jobs();
            dispatch({ type: 'SET_JOBS', value: data });
        } catch (error: unknown) {
            console.error('Failed to fetch scheduled jobs:', error);
        } finally {
            dispatch({ type: 'SET_LOADING', value: false });
        }
    }, []);

    const fetch_runs = useCallback(async (job_id: string) => {
        try {
            const data = await system_api_service.continuity.get_scheduled_job_runs(job_id);
            dispatch({ type: 'SET_RUNS', job_id, runs: data });
        } catch (error: unknown) {
            console.error('Failed to fetch runs:', error);
        }
    }, []);

    useEffect(() => {
        agent_service.load_agents_into_store();
        fetch_workflows();
        fetch_jobs();
    }, [fetch_jobs, fetch_workflows]);

    const toggle_expand = useCallback((job_id: string) => {
        if (state.expanded_job === job_id) {
            dispatch({ type: 'SET_EXPANDED_JOB', value: null });
        } else {
            dispatch({ type: 'SET_EXPANDED_JOB', value: job_id });
            if (!state.runs_map[job_id]) {
                fetch_runs(job_id);
            }
        }
    }, [state.expanded_job, state.runs_map, fetch_runs]);

    const toggle_enable = useCallback(async (job: Scheduled_Job) => {
        try {
            await system_api_service.continuity.update_scheduled_job(job.id, { enabled: !job.enabled });
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

    const handle_edit = useCallback((job: Scheduled_Job) => {
        dispatch({ type: 'OPEN_EDIT_FORM', job });
    }, []);

    const delete_job = useCallback((id: string, name: string) => {
        dispatch({
            type: 'SET_CONFIRM_DELETE',
            value: { id, name }
        });
    }, []);

    const handle_confirm_delete = useCallback(async () => {
        if (!state.confirm_delete) return;
        const { id, name } = state.confirm_delete;
        
        try {
            await system_api_service.continuity.delete_scheduled_job(id);
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('scheduled_jobs.event_terminated', { name: name }),
                severity: 'info'
            });
            dispatch({ type: 'SET_CONFIRM_DELETE', value: null });
            fetch_jobs();
        } catch (error: unknown) {
            console.error('Failed to delete job:', error);
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('scheduled_jobs.event_deletion_failed', { error: error instanceof Error ? error.message : i18n.t('common.unknown_error') }),
                severity: 'error'
            });
            dispatch({ type: 'SET_CONFIRM_DELETE', value: null });
        }
    }, [state.confirm_delete, fetch_jobs]);

    const reset_form = useCallback(() => {
        dispatch({ type: 'RESET_FORM' });
    }, []);

    const handle_job_submit = useCallback(async (e: React.FormEvent) => {
        e.preventDefault();
        // Explicit payload mapping to prevent contract bleed
        const payload: Job_Config_State = {
            name: state.job_config.name,
            agent_id: state.job_config.agent_id,
            workflow_id: state.job_config.workflow_id,
            prompt: state.job_config.prompt,
            cron_expr: state.job_config.cron_expr,
            budget_usd: state.job_config.budget_usd,
            max_failures: state.job_config.max_failures
        };

        try {
            if (state.editing_job_id) {
                await system_api_service.continuity.update_scheduled_job(state.editing_job_id, payload);
                event_bus.emit_log({
                    source: 'System',
                    text: i18n.t('scheduled_jobs.event_updated', { name: payload.name }),
                    severity: 'success'
                });
            } else {
                await system_api_service.continuity.create_scheduled_job(payload);
                event_bus.emit_log({
                    source: 'System',
                    text: i18n.t('scheduled_jobs.event_created', { name: payload.name }),
                    severity: 'success'
                });
            }
            reset_form();
            fetch_jobs();
        } catch (error: unknown) { 
            console.error('Failed to handle job:', error);
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('scheduled_jobs.event_failed_action', { action: state.editing_job_id ? 'update' : 'create' }),
                severity: 'error'
            });
        }
    }, [state.editing_job_id, state.job_config, reset_form, fetch_jobs]);

    return {
        jobs: state.jobs,
        workflows: state.workflows,
        is_loading: state.is_loading,
        expanded_job: state.expanded_job,
        runs_map: state.runs_map,
        confirm_delete: state.confirm_delete,
        is_creating: state.is_creating,
        editing_job_id: state.editing_job_id,
        job_type: state.job_type,
        target_search: state.target_search,
        job_config: state.job_config,
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
