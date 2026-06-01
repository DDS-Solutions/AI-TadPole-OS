/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[continuity_api]` in observability traces.
 */

import { api_request } from '../base_api_service';
import type { Scheduled_Job, Scheduled_Job_Run, Workflow_Entry, Workflow_Step } from '../system_api_types';

export const continuity_api = {
    get_scheduled_jobs: async (): Promise<Scheduled_Job[]> => {
        const res = await api_request<{ jobs: Scheduled_Job[] } | Scheduled_Job[]>('/v1/continuity/jobs', { method: 'GET' });
        return Array.isArray(res) ? res : (res.jobs || []);
    },

    create_scheduled_job: async (job: Partial<Scheduled_Job>): Promise<Scheduled_Job> => {
        return api_request<Scheduled_Job>('/v1/continuity/jobs', {
            method: 'POST',
            body: JSON.stringify(job)
        });
    },

    update_scheduled_job: async (id: string, job: Partial<Scheduled_Job>): Promise<Scheduled_Job> => {
        return api_request<Scheduled_Job>(`/v1/continuity/jobs/${id}`, {
            method: 'PUT',
            body: JSON.stringify(job)
        });
    },

    delete_scheduled_job: async (id: string): Promise<void> => {
        return api_request<void>(`/v1/continuity/jobs/${id}`, { method: 'DELETE' });
    },

    get_scheduled_job_runs: async (id: string): Promise<Scheduled_Job_Run[]> => {
        const res = await api_request<{ runs: Scheduled_Job_Run[] } | Scheduled_Job_Run[]>(`/v1/continuity/jobs/${id}/runs`, { method: 'GET' });
        return Array.isArray(res) ? res : (res.runs || []);
    },

    list_continuity_workflows: async (): Promise<Workflow_Entry[]> => {
        const res = await api_request<{ workflows: Workflow_Entry[] } | Workflow_Entry[]>('/v1/continuity/workflows', { method: 'GET' });
        return Array.isArray(res) ? res : (res.workflows || []);
    },

    create_continuity_workflows: async (data: { name: string; description?: string }): Promise<Workflow_Entry> => {
        return api_request<Workflow_Entry>('/v1/continuity/workflows', {
            method: 'POST',
            body: JSON.stringify(data)
        });
    },

    add_continuity_workflows_step: async (workflow_id: string, step: Partial<Workflow_Step>): Promise<Workflow_Step> => {
        return api_request<Workflow_Step>(`/v1/continuity/workflows/${workflow_id}/steps`, {
            method: 'POST',
            body: JSON.stringify(step)
        });
    },

    delete_continuity_workflows: async (workflow_id: string): Promise<void> => {
        return api_request(`/v1/continuity/workflows/${workflow_id}`, { method: 'DELETE' });
    }
};

// Metadata: [continuity_api]
