/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / continuity_api.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { continuity_api } from './continuity_api';
import { api_request } from '../base_api_service';

vi.mock('../base_api_service', () => ({
    api_request: vi.fn()
}));

describe('continuity_api', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('get_scheduled_jobs', () => {
        it('handles array format', async () => {
            const mock_jobs = [{ id: '1', name: 'job1' }];
            vi.mocked(api_request).mockResolvedValueOnce(mock_jobs);

            const result = await continuity_api.get_scheduled_jobs();
            expect(result).toEqual(mock_jobs);
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/jobs', expect.objectContaining({ method: 'GET' }));
        });

        it('handles envelope format', async () => {
            const mock_envelope = { jobs: [{ id: '2', name: 'job2' }] };
            vi.mocked(api_request).mockResolvedValueOnce(mock_envelope);

            const result = await continuity_api.get_scheduled_jobs();
            expect(result).toEqual(mock_envelope.jobs);
        });

        it('returns empty array when api returns null or undefined', async () => {
            vi.mocked(api_request).mockResolvedValueOnce(null);
            const result_null = await continuity_api.get_scheduled_jobs();
            expect(result_null).toEqual([]);

            vi.mocked(api_request).mockResolvedValueOnce(undefined);
            const result_undef = await continuity_api.get_scheduled_jobs();
            expect(result_undef).toEqual([]);
        });

        it('propagates abort signal and timeout option', async () => {
            const controller = new AbortController();
            vi.mocked(api_request).mockResolvedValueOnce([]);

            await continuity_api.get_scheduled_jobs({ signal: controller.signal, timeout: 4000 });
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/jobs', expect.objectContaining({
                signal: controller.signal,
                timeout: 4000
            }));
        });
    });

    describe('create_scheduled_job', () => {
        it('calls POST and passes request body', async () => {
            const mock_job = { name: 'new_job' };
            vi.mocked(api_request).mockResolvedValueOnce({ id: 'j-1', ...mock_job });

            const result = await continuity_api.create_scheduled_job(mock_job);
            expect(result.id).toBe('j-1');
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/jobs', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify(mock_job)
            }));
        });
    });

    describe('update_scheduled_job', () => {
        it('calls PUT on safe id and body', async () => {
            const mock_job = { name: 'updated_name' };
            vi.mocked(api_request).mockResolvedValueOnce({ id: 'j-2', ...mock_job });

            const result = await continuity_api.update_scheduled_job('j-2', mock_job);
            expect(result.id).toBe('j-2');
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/jobs/j-2', expect.objectContaining({
                method: 'PUT',
                body: JSON.stringify(mock_job)
            }));
        });

        it('blocks traversal ids and invalid characters', async () => {
            await expect(continuity_api.update_scheduled_job('../workflows', {}))
                .rejects.toThrow('Invalid identifier: ../workflows');
            await expect(continuity_api.update_scheduled_job('job-1\u200b', {}))
                .rejects.toThrow('Invalid identifier: job-1\u200b');
            await expect(continuity_api.update_scheduled_job('job-1 2', {}))
                .rejects.toThrow('Invalid identifier: job-1 2');
            await expect(continuity_api.update_scheduled_job('job#1', {}))
                .rejects.toThrow('Invalid identifier: job#1');
            await expect(continuity_api.update_scheduled_job('a'.repeat(65), {}))
                .rejects.toThrow('Invalid identifier:');
            expect(api_request).not.toHaveBeenCalled();
        });
    });

    describe('delete_scheduled_job', () => {
        it('calls DELETE on safe id', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});

            await continuity_api.delete_scheduled_job('j-3');
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/jobs/j-3', expect.objectContaining({
                method: 'DELETE'
            }));
        });

        it('blocks traversal ids and invalid characters', async () => {
            await expect(continuity_api.delete_scheduled_job('j-3/../../admin'))
                .rejects.toThrow('Invalid identifier: j-3/../../admin');
            await expect(continuity_api.delete_scheduled_job('j-3*'))
                .rejects.toThrow('Invalid identifier: j-3*');
            expect(api_request).not.toHaveBeenCalled();
        });
    });

    describe('get_scheduled_job_runs', () => {
        it('handles array format', async () => {
            const mock_runs = [{ id: 'run-1', status: 'success' }];
            vi.mocked(api_request).mockResolvedValueOnce(mock_runs);

            const result = await continuity_api.get_scheduled_job_runs('j-1');
            expect(result).toEqual(mock_runs);
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/jobs/j-1/runs', expect.objectContaining({ method: 'GET' }));
        });

        it('handles envelope format', async () => {
            const mock_runs = [{ id: 'run-2', status: 'failure' }];
            vi.mocked(api_request).mockResolvedValueOnce({ runs: mock_runs });

            const result = await continuity_api.get_scheduled_job_runs('j-1');
            expect(result).toEqual(mock_runs);
        });

        it('returns empty array when api returns null or undefined', async () => {
            vi.mocked(api_request).mockResolvedValueOnce(null);
            const result = await continuity_api.get_scheduled_job_runs('j-1');
            expect(result).toEqual([]);
        });

        it('blocks traversal ids and invalid characters', async () => {
            await expect(continuity_api.get_scheduled_job_runs('..\\wf'))
                .rejects.toThrow('Invalid identifier: ..\\wf');
            await expect(continuity_api.get_scheduled_job_runs('wf name'))
                .rejects.toThrow('Invalid identifier: wf name');
        });
    });

    describe('list_continuity_workflows', () => {
        it('handles array/envelope response', async () => {
            const mock_wf = [{ name: 'wf-1' }];
            vi.mocked(api_request).mockResolvedValueOnce(mock_wf);

            const result = await continuity_api.list_continuity_workflows();
            expect(result).toEqual(mock_wf);
        });

        it('handles null response safely', async () => {
            vi.mocked(api_request).mockResolvedValueOnce(null);

            const result = await continuity_api.list_continuity_workflows();
            expect(result).toEqual([]);
        });
    });

    describe('add_continuity_workflows_step', () => {
        it('calls POST and appends step', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ id: 'step-1' });

            const result = await continuity_api.add_continuity_workflows_step('w-1', { prompt: 'run cmd' });
            expect(result).toEqual({ id: 'step-1' });
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/workflows/w-1/steps', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({ prompt: 'run cmd' })
            }));
        });

        it('blocks traversal ids and invalid characters', async () => {
            await expect(continuity_api.add_continuity_workflows_step('w-1/../w-2', {}))
                .rejects.toThrow('Invalid identifier: w-1/../w-2');
            await expect(continuity_api.add_continuity_workflows_step('w-1"quote', {}))
                .rejects.toThrow('Invalid identifier: w-1"quote');
        });
    });

    describe('delete_continuity_workflows', () => {
        it('calls DELETE', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});

            await continuity_api.delete_continuity_workflows('w-1');
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/workflows/w-1', expect.objectContaining({
                method: 'DELETE'
            }));
        });

        it('blocks traversal ids and invalid characters', async () => {
            await expect(continuity_api.delete_continuity_workflows('w-1/../../admin'))
                .rejects.toThrow('Invalid identifier: w-1/../../admin');
            await expect(continuity_api.delete_continuity_workflows('w-1\\admin'))
                .rejects.toThrow('Invalid identifier: w-1\\admin');
        });
    });
});
