/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / continuity_api
 * - **Primary Entrypoints**: `continuity_api`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { api_request } from '../base_api_service';
import type { Scheduled_Job, Scheduled_Job_Run, Workflow_Entry, Workflow_Step, Workflow_Step_Run, RequestOptions } from '../system_api_types';

const sanitize_id = (id: string): string => {
    if (!id || !/^[a-zA-Z0-9_.-]{1,64}$/.test(id)) {
        throw new Error(`Invalid identifier: ${id}`);
    }
    return encodeURIComponent(id);
};

export const continuity_api = {
    get_scheduled_jobs: async (options?: RequestOptions): Promise<Scheduled_Job[]> => {
        const res = await api_request<{ jobs: Scheduled_Job[] } | Scheduled_Job[]>('/v1/continuity/jobs', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
        if (!res) return [];
        return Array.isArray(res) ? res : (res.jobs || []);
    },

    create_scheduled_job: async (job: Partial<Scheduled_Job>, options?: RequestOptions): Promise<Scheduled_Job> => {
        return api_request<Scheduled_Job>('/v1/continuity/jobs', {
            method: 'POST',
            body: JSON.stringify(job),
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    update_scheduled_job: async (id: string, job: Partial<Scheduled_Job>, options?: RequestOptions): Promise<Scheduled_Job> => {
        const clean_id = sanitize_id(id);
        return api_request<Scheduled_Job>(`/v1/continuity/jobs/${clean_id}`, {
            method: 'PUT',
            body: JSON.stringify(job),
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    delete_scheduled_job: async (id: string, options?: RequestOptions): Promise<void> => {
        const clean_id = sanitize_id(id);
        return api_request<void>(`/v1/continuity/jobs/${clean_id}`, { 
            method: 'DELETE',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    get_scheduled_job_runs: async (id: string, options?: RequestOptions): Promise<Scheduled_Job_Run[]> => {
        const clean_id = sanitize_id(id);
        const res = await api_request<{ runs: Scheduled_Job_Run[] } | Scheduled_Job_Run[]>(`/v1/continuity/jobs/${clean_id}/runs`, { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
        if (!res) return [];
        return Array.isArray(res) ? res : (res.runs || []);
    },

    list_continuity_workflows: async (options?: RequestOptions): Promise<Workflow_Entry[]> => {
        const res = await api_request<{ workflows: Workflow_Entry[] } | Workflow_Entry[]>('/v1/continuity/workflows', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
        if (!res) return [];
        return Array.isArray(res) ? res : (res.workflows || []);
    },

    create_continuity_workflows: async (data: { name: string; description?: string }, options?: RequestOptions): Promise<Workflow_Entry> => {
        return api_request<Workflow_Entry>('/v1/continuity/workflows', {
            method: 'POST',
            body: JSON.stringify(data),
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    add_continuity_workflows_step: async (workflow_id: string, step: Partial<Workflow_Step>, options?: RequestOptions): Promise<Workflow_Step> => {
        const clean_workflow_id = sanitize_id(workflow_id);
        return api_request<Workflow_Step>(`/v1/continuity/workflows/${clean_workflow_id}/steps`, {
            method: 'POST',
            body: JSON.stringify(step),
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    delete_continuity_workflows: async (workflow_id: string, options?: RequestOptions): Promise<void> => {
        const clean_workflow_id = sanitize_id(workflow_id);
        return api_request<void>(`/v1/continuity/workflows/${clean_workflow_id}`, { 
            method: 'DELETE',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    trigger_scheduled_job: async (id: string, options?: RequestOptions): Promise<void> => {
        const clean_id = sanitize_id(id);
        return api_request<void>(`/v1/continuity/jobs/${clean_id}/run`, { 
            method: 'POST',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    get_workflow_run_steps: async (run_id: string, options?: RequestOptions): Promise<Workflow_Step_Run[]> => {
        const clean_run_id = sanitize_id(run_id);
        const res = await api_request<{ step_runs: Workflow_Step_Run[] } | Workflow_Step_Run[]>(`/v1/continuity/workflow-runs/${clean_run_id}/steps`, {
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
        if (!res) return [];
        return Array.isArray(res) ? res : (res.step_runs || []);
    }
};
