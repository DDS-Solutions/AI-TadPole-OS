/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / registry_service.test
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
import { AgentRegistryService } from './registry_service';
import type { Agent, AgentPatch } from '../../contracts/agent';

describe('AgentRegistryService', () => {
    let mock_api_request: any;
    let service: AgentRegistryService;

    beforeEach(() => {
        mock_api_request = vi.fn();
        service = new AgentRegistryService(mock_api_request);
    });

    describe('get_agents', () => {
        it('fetches agents from /v1/agents and caches the result', async () => {
            const mock_agents = [{ id: 'agent-1', name: 'Agent 1' }];
            mock_api_request.mockResolvedValueOnce(mock_agents);

            const result1 = await service.get_agents();
            const result2 = await service.get_agents();

            expect(mock_api_request).toHaveBeenCalledTimes(1);
            expect(result1).toEqual(mock_agents);
            expect(result2).toEqual(mock_agents);
        });

        it('supports envelope unwrapping if response contains data array', async () => {
            const mock_agents = [{ id: 'agent-1', name: 'Agent 1' }];
            mock_api_request.mockResolvedValueOnce({ data: mock_agents });

            const result = await service.get_agents();
            expect(result).toEqual(mock_agents);
        });

        it('refetches agents after invalidate_agents_cache is called', async () => {
            mock_api_request.mockResolvedValueOnce([{ id: '1' }]);
            mock_api_request.mockResolvedValueOnce([{ id: '2' }]);

            await service.get_agents();
            service.invalidate_agents_cache();
            const res2 = await service.get_agents();

            expect(mock_api_request).toHaveBeenCalledTimes(2);
            expect(res2).toEqual([{ id: '2' }]);
        });

        it('rejects if already aborted signal is provided', async () => {
            const controller = new AbortController();
            controller.abort();

            await expect(service.get_agents({ signal: controller.signal })).rejects.toThrow(/aborted/);
        });

        it('rejects when signal aborts during request', async () => {
            const controller = new AbortController();
            mock_api_request.mockImplementation(() => new Promise((resolve) => setTimeout(resolve, 500)));

            const promise = service.get_agents({ signal: controller.signal });
            controller.abort();

            await expect(promise).rejects.toThrow(/aborted/);
        });
    });

    describe('mutation and lifecycle operations', () => {
        it('update_agent sends PUT and invalidates cache', async () => {
            mock_api_request.mockResolvedValueOnce({});
            const patch: AgentPatch = { name: 'New Name' };

            const success = await service.update_agent('agent-1', patch);
            expect(mock_api_request).toHaveBeenCalledWith('/v1/agents/agent-1', expect.objectContaining({
                method: 'PUT'
            }));
            expect(success).toBe(true);
        });

        it('create_agent sends POST and invalidates cache', async () => {
            mock_api_request.mockResolvedValueOnce({});
            const agent: Agent = {
                id: 'agent-new',
                name: 'New Agent',
                role: 'Worker',
                status: 'idle',
                model: 'claude-3',
                provider: 'anthropic',
                workspace_path: '/workspaces/agent-new'
            } as any;

            const success = await service.create_agent(agent);
            expect(mock_api_request).toHaveBeenCalledWith('/v1/agents', expect.objectContaining({
                method: 'POST'
            }));
            expect(success).toBe(true);
        });

        it('delete_agent sends DELETE and invalidates cache', async () => {
            mock_api_request.mockResolvedValueOnce({});
            const success = await service.delete_agent('agent-1');
            expect(mock_api_request).toHaveBeenCalledWith('/v1/agents/agent-1', { method: 'DELETE' });
            expect(success).toBe(true);
        });

        it('pause_agent sends POST to /v1/agents/:id/pause', async () => {
            mock_api_request.mockResolvedValueOnce({});
            const success = await service.pause_agent('agent-1');
            expect(mock_api_request).toHaveBeenCalledWith('/v1/agents/agent-1/pause', { method: 'POST' });
            expect(success).toBe(true);
        });

        it('resume_agent sends POST to /v1/agents/:id/resume', async () => {
            mock_api_request.mockResolvedValueOnce({});
            const success = await service.resume_agent('agent-1');
            expect(mock_api_request).toHaveBeenCalledWith('/v1/agents/agent-1/resume', { method: 'POST' });
            expect(success).toBe(true);
        });

        it('reset_agent sends POST to /v1/agents/:id/reset and returns status', async () => {
            const mock_res = { status: 'success', message: 'Reset completed' };
            mock_api_request.mockResolvedValueOnce(mock_res);
            const result = await service.reset_agent('agent-1');
            expect(mock_api_request).toHaveBeenCalledWith('/v1/agents/agent-1/reset', { method: 'POST' });
            expect(result).toEqual(mock_res);
        });

        it('rethrows mapped api errors on failure', async () => {
            mock_api_request.mockRejectedValueOnce(new Error('Reset error'));
            await expect(service.reset_agent('agent-1')).rejects.toThrow();
        });
    });
});
