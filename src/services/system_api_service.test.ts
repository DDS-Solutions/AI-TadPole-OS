/**
 * @file system_api_service.test.ts
 * @description Suite for core system management, infrastructure, and engine control.
 * @module Services/system_api_service
 * @testedBehavior
 * - Engine Governance: Health checks, deployment, and emergency shutdown (kill switch).
 * - Multi-modal AI: Transcription and TTS synthesis orchestration.
 * - Infrastructure: Provider management, node discovery, and benchmarking.
 * - Continuity: Scheduling and management of recurring system jobs.
 * @aiContext
 * - Mocks apiRequest and DEPLOY_TIMEOUT constants.
 * - Tests both flat array and enveloped JSON response patterns used across the engine API.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { system_api_service } from './system_api_service';
import { apiRequest } from './base_api_service';

vi.mock('./base_api_service', () => ({
    apiRequest: vi.fn(),
    DEPLOY_TIMEOUT: 60000,
}));

describe('system_api_service', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('check_health', () => {
        it('should return true if apiRequest succeeds', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            const res = await system_api_service.check_health();
            expect(res).toBe(true);
            expect(apiRequest).toHaveBeenCalledWith('/v1/engine/health', { method: 'GET', timeout: 5000 });
        });

        it('should return false if apiRequest fails', async () => {
            vi.mocked(apiRequest).mockRejectedValueOnce(new Error('fail'));
            const res = await system_api_service.check_health();
            expect(res).toBe(false);
        });
    });

    it('deploy_engine', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({ status: 'ok' });
        await system_api_service.deploy_engine('target1');
        expect(apiRequest).toHaveBeenCalledWith('/v1/engine/deploy?target=target1', expect.objectContaining({ method: 'POST' }));
    });

    it('speak', async () => {
        const mockBlob = new Blob(['audio']);
        vi.mocked(apiRequest).mockResolvedValueOnce(mockBlob);
        const res = await system_api_service.speak('hello', 'voice1', 'engine1');
        expect(res).toBe(mockBlob);
        expect(apiRequest).toHaveBeenCalledWith('/v1/engine/speak', expect.objectContaining({
            method: 'POST',
            body: JSON.stringify({ text: 'hello', voice: 'voice1', engine: 'engine1' }),
            responseType: 'blob'
        }));
    });

    it('kill_agents', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        await system_api_service.kill_agents();
        expect(apiRequest).toHaveBeenCalledWith('/v1/engine/kill', { method: 'POST' });
    });

    it('shutdown_engine', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        await system_api_service.shutdown_engine();
        expect(apiRequest).toHaveBeenCalledWith('/v1/engine/shutdown', { method: 'POST' });
    });

    it('transcribe', async () => {
        const mockBlob = new Blob(['audio']);
        vi.mocked(apiRequest).mockResolvedValueOnce({ text: 'hello' });
        const res = await system_api_service.transcribe(mockBlob);
        expect(res).toBe('hello');
        expect(apiRequest).toHaveBeenCalledWith('/v1/engine/transcribe', expect.objectContaining({
            method: 'POST',
        }));
    });
    
    it('transcribe with no text', async () => {
        const mockBlob = new Blob(['audio']);
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        const res = await system_api_service.transcribe(mockBlob);
        expect(res).toBe('');
    });

    describe('test_provider', () => {
        it('returns success if request passes', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ status: 'ok', latency: 100 });
            const res = await (system_api_service as any).test_provider('P', 'K', 'U');
            expect(res).toEqual({ status: 'ok', latency: 100 });
        });

        it('returns timeout error string if TIMEOUT is thrown', async () => {
            vi.mocked(apiRequest).mockRejectedValueOnce('TIMEOUT');
            const res = await (system_api_service as any).test_provider('P', 'K', 'U');
            expect(res).toEqual({ status: 'error', message: 'Handshake timeout: The provider endpoint is unresponsive.' });
        });

        it('returns generic error string if regular error is thrown', async () => {
            vi.mocked(apiRequest).mockRejectedValueOnce(new Error('Bad Gateway'));
            const res = await (system_api_service as any).test_provider('P', 'K', 'U');
            expect(res).toEqual({ status: 'error', message: 'Bad Gateway' });
        });
    });

    describe('Continuity Jobs', () => {
        it('get_scheduled_jobs handles array', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce([{ id: '1' }]);
            const res = await system_api_service.get_scheduled_jobs();
            expect(res).toEqual([{ id: '1' }]);
        });

        it('get_scheduled_jobs handles envelope', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ jobs: [{ id: '2' }] });
            const res = await system_api_service.get_scheduled_jobs();
            expect(res).toEqual([{ id: '2' }]);
        });

        it('create_scheduled_job', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ id: 'new' });
            await system_api_service.create_scheduled_job({ name: 'job1' });
            expect(apiRequest).toHaveBeenCalledWith('/v1/continuity/jobs', expect.objectContaining({ method: 'POST' }));
        });

        it('update_scheduled_job', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ id: 'updated' });
            await system_api_service.update_scheduled_job('j1', { name: 'job1' });
            expect(apiRequest).toHaveBeenCalledWith('/v1/continuity/jobs/j1', expect.objectContaining({ method: 'PUT' }));
        });

        it('delete_scheduled_job', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await system_api_service.delete_scheduled_job('j1');
            expect(apiRequest).toHaveBeenCalledWith('/v1/continuity/jobs/j1', expect.objectContaining({ method: 'DELETE' }));
        });

        it('get_scheduled_job_runs', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ runs: [{ id: 'r1' }] });
            const res = await system_api_service.get_scheduled_job_runs('j1');
            expect(res).toEqual([{ id: 'r1' }]);
            
            vi.mocked(apiRequest).mockResolvedValueOnce([{ id: 'r2' }]);
            const res2 = await system_api_service.get_scheduled_job_runs('j1');
            expect(res2).toEqual([{ id: 'r2' }]);
        });
    });

    describe('Oversight', () => {
        it('get_pending_oversight', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ data: [{ id: '1' }] });
            const res = await system_api_service.get_pending_oversight();
            expect(res).toEqual([{ id: '1' }]);
            
            vi.mocked(apiRequest).mockResolvedValueOnce([{ id: '2' }]);
            const res2 = await system_api_service.get_pending_oversight();
            expect(res2).toEqual([{ id: '2' }]);
        });

        it('get_oversight_ledger', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ data: [{ id: '1' }] });
            const res = await system_api_service.get_oversight_ledger();
            expect(res).toEqual([{ id: '1' }]);
        });

        it('decide_oversight', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await system_api_service.decide_oversight('1', 'approved');
            expect(apiRequest).toHaveBeenCalledWith('/v1/oversight/1/decide', expect.objectContaining({ method: 'POST', body: '{"decision":"approved"}' }));
        });
    });

    describe('Other operations', () => {
        it('get_nodes', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce([]);
            await system_api_service.get_nodes();
            expect(apiRequest).toHaveBeenCalledWith('/v1/infra/nodes', { method: 'GET' });
        });
        
        it('discover_nodes', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({status: 'ok', discovered: []});
            await system_api_service.discover_nodes();
            expect(apiRequest).toHaveBeenCalledWith('/v1/infra/nodes/discover', { method: 'POST' });
        });
        
        it('get_benchmarks', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce([]);
            await system_api_service.get_benchmarks();
            expect(apiRequest).toHaveBeenCalledWith('/v1/benchmarks', { method: 'GET' });
        });
        
        it('run_benchmark', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({id: 'b1'});
            await system_api_service.run_benchmark('t1');
            expect(apiRequest).toHaveBeenCalledWith('/v1/benchmarks/run/t1', { method: 'POST' });
        });

        it('get_knowledge_docs', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce([]);
            await system_api_service.get_knowledge_docs();
            expect(apiRequest).toHaveBeenCalledWith('/v1/docs/knowledge', { method: 'GET' });
        });

        it('get_knowledge_doc', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce('# Doc');
            await system_api_service.get_knowledge_doc('cat', 'name');
            expect(apiRequest).toHaveBeenCalledWith('/v1/docs/knowledge/cat/name', expect.objectContaining({ method: 'GET' }));
        });

        it('get_operations_manual', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce('# Ops');
            await system_api_service.get_operations_manual();
            expect(apiRequest).toHaveBeenCalledWith('/v1/docs/operations-manual', expect.objectContaining({ method: 'GET' }));
        });

        it('get_providers', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce([]);
            await system_api_service.get_providers();
            expect(apiRequest).toHaveBeenCalledWith('/v1/infra/providers', { method: 'GET' });
        });

        it('update_provider', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({status: 'ok'});
            await system_api_service.update_provider('p1', {k: 'v'});
            expect(apiRequest).toHaveBeenCalledWith('/v1/infra/providers/p1', expect.objectContaining({ method: 'PUT' }));
        });
        
        it('get_models', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce([]);
            await system_api_service.get_models();
            expect(apiRequest).toHaveBeenCalledWith('/v1/infra/models', { method: 'GET' });
        });

        it('update_model', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({status: 'ok'});
            await system_api_service.update_model('m1', {k: 'v'});
            expect(apiRequest).toHaveBeenCalledWith('/v1/infra/models/m1', expect.objectContaining({ method: 'PUT' }));
        });

        it('get_security_quotas', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await system_api_service.get_security_quotas();
            expect(apiRequest).toHaveBeenCalledWith('/v1/oversight/security/quotas', { method: 'GET' });
        });

        it('get_audit_trail', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({data: [], total: 0});
            await system_api_service.get_audit_trail(2, 25);
            expect(apiRequest).toHaveBeenCalledWith('/v1/oversight/security/audit-trail?page=2&per_page=25', { method: 'GET' });
        });

        it('get_agent_health', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({agents: []});
            await system_api_service.get_agent_health();
            expect(apiRequest).toHaveBeenCalledWith('/v1/oversight/security/health', { method: 'GET' });
        });

        it('list_continuity_workflows', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce([]);
            await system_api_service.list_continuity_workflows();
            expect(apiRequest).toHaveBeenCalledWith('/v1/continuity/workflows', { method: 'GET' });
        });

        it('create_continuity_workflows', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await system_api_service.create_continuity_workflows({name: 'w1'});
            expect(apiRequest).toHaveBeenCalledWith('/v1/continuity/workflows', expect.objectContaining({ method: 'POST' }));
        });

        it('add_continuity_workflows_step', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await system_api_service.add_continuity_workflows_step('w1', {prompt: 'p1'});
            expect(apiRequest).toHaveBeenCalledWith('/v1/continuity/workflows/w1/steps', expect.objectContaining({ method: 'POST' }));
        });

        it('delete_continuity_workflows', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await system_api_service.delete_continuity_workflows('w1');
            expect(apiRequest).toHaveBeenCalledWith('/v1/continuity/workflows/w1', expect.objectContaining({ method: 'DELETE' }));
        });
    });
});
