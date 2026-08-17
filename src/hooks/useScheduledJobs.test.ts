/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Assist Note
 * Regression coverage for the adjacent production module and its public contracts.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Contract, rendering, state transition, or error-handling regression.
 * - **Trace Scope**: Vitest assertions and test-local mocks.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useScheduledJobs } from './useScheduledJobs';
import { use_agent_store } from '../stores/agent_store';

const mock_get_jobs = vi.fn();
const mock_list_workflows = vi.fn();
const mock_create_job = vi.fn();
const mock_update_job = vi.fn();
const mock_delete_job = vi.fn();

vi.mock('../services/system_api_service', () => ({
    system_api_service: {
        continuity: {
            get_scheduled_jobs: () => mock_get_jobs(),
            list_continuity_workflows: () => mock_list_workflows(),
            create_scheduled_job: (p: any) => mock_create_job(p),
            update_scheduled_job: (id: string, p: any) => mock_update_job(id, p),
            delete_scheduled_job: (id: string) => mock_delete_job(id),
            get_scheduled_job_runs: vi.fn().mockResolvedValue([])
        }
    }
}));

vi.mock('../services/mission_api_service', () => ({
    mission_api_service: {
        get_unified_skills: vi.fn().mockResolvedValue({ workflows: [] })
    }
}));

describe('useScheduledJobs', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        mock_get_jobs.mockResolvedValue([
            { id: 'job-1', name: 'Hourly Scan', cron_expr: '0 * * * *', enabled: true, budget_usd: 0.1, max_failures: 3 }
        ]);
        mock_list_workflows.mockResolvedValue([]);

        use_agent_store.setState({
            agents: [
                { id: 'agent-1', name: 'Scout Agent', role: 'Research' } as any,
                { id: 'agent-2', name: 'Builder Agent', role: 'Engineering' } as any
            ]
        });
    });

    it('initializes and provides memoized filtered agents list', () => {
        const { result } = renderHook(() => useScheduledJobs());
        expect(result.current.filtered_agents.length).toBe(2);

        act(() => {
            result.current.set_target_search('scout');
        });

        expect(result.current.filtered_agents.length).toBe(1);
        expect(result.current.filtered_agents[0].name).toBe('Scout Agent');
    });

    it('manages form creation and editing state', () => {
        const { result } = renderHook(() => useScheduledJobs());

        act(() => {
            result.current.set_is_creating(true);
            result.current.set_job_config(prev => ({ ...prev, name: 'Nightly Audit' }));
        });

        expect(result.current.is_creating).toBe(true);
        expect(result.current.job_config.name).toBe('Nightly Audit');

        act(() => {
            result.current.reset_form();
        });

        expect(result.current.is_creating).toBe(false);
        expect(result.current.job_config.name).toBe('');
    });

    it('handles job edit population', () => {
        const { result } = renderHook(() => useScheduledJobs());

        act(() => {
            result.current.handle_edit({
                id: 'job-100',
                name: 'Data Ingestion',
                agent_id: 'agent-1',
                cron_expr: '0 0 * * *',
                budget_usd: 1.0,
                max_failures: 5
            } as any);
        });

        expect(result.current.editing_job_id).toBe('job-100');
        expect(result.current.job_config.name).toBe('Data Ingestion');
        expect(result.current.job_config.cron_expr).toBe('0 0 * * *');
    });

    it('handles delete job confirmation triggers', () => {
        const { result } = renderHook(() => useScheduledJobs());

        act(() => {
            result.current.delete_job('job-99', 'Old Job');
        });

        expect(result.current.confirm_delete).toEqual({ id: 'job-99', name: 'Old Job' });
    });
});
