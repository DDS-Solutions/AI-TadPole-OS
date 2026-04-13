/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the base System and Infrastructure API client.** 
 * Verifies core engine pulses, deployment signaling, and cross-sector oversight (Quotas, Audit Trail, Nodes). 
 * Mocks `api_request` to isolate system-level orchestration from network side-effects and backend engine latency.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Failure to propagate backend-wide maintenance signals or incorrect parsing of environment-level metadata during an infrastructure handshake.
 * - **Telemetry Link**: Search `[system_api_service.test]` in tracing logs.
 */


/**
 * @file system_api_service.test.ts
 * @description Suite for the Tadpole OS System and Infrastructure API layer.
 * @module Services/system_api_service
 * @testedBehavior
 * - Core Health: Engine pulse and connectivity verification.
 * - Engine Orchestration: Deployment, power-cycling, and global kill-switch signaling.
 * - Neural Comms: Voice synthesis (TTS) and transcription orchestration.
 * - Managed Services: CRUD for continuity jobs, oversight ledgers, and swarm infrastructure.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks api_request to isolate system-level orchestration from network side-effects.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { system_api_service } from './system_api_service';
import { api_request } from './base_api_service';

vi.mock('./base_api_service', () => ({
    api_request: vi.fn(),
    DEPLOY_TIMEOUT: 60000,
}));

describe('system_api_service', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('check_health', () => {
        it('should return true if api_request succeeds', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            const res = await system_api_service.check_health();
            expect(res).toBe(true);
            expect(api_request).toHaveBeenCalledWith('/v1/engine/health', { method: 'GET', timeout: 5000 });
        });

        it('should return false if api_request fails', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('fail'));
            const res = await system_api_service.check_health();
            expect(res).toBe(false);
        });
    });

    it('deploy_engine', async () => {
        vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });
        await system_api_service.deploy_engine('target1');
        expect(api_request).toHaveBeenCalledWith('/v1/engine/deploy?target=target1', expect.objectContaining({ method: 'POST' }));
    });

    it('speak', async () => {
        const mock_blob = new Blob(['audio']);
        vi.mocked(api_request).mockResolvedValueOnce(mock_blob);
        const res = await system_api_service.speak('hello', 'voice1', 'engine1');
        expect(res).toBe(mock_blob);
        expect(api_request).toHaveBeenCalledWith('/v1/engine/speak', expect.objectContaining({
            method: 'POST',
            body: JSON.stringify({ text: 'hello', voice: 'voice1', engine: 'engine1' }),
            response_type: 'blob'
        }));
    });

    it('kill_agents', async () => {
        vi.mocked(api_request).mockResolvedValueOnce({});
        await system_api_service.kill_agents();
        expect(api_request).toHaveBeenCalledWith('/v1/engine/kill', { method: 'POST' });
    });

    it('shutdown_engine', async () => {
        vi.mocked(api_request).mockResolvedValueOnce({});
        await system_api_service.shutdown_engine();
        expect(api_request).toHaveBeenCalledWith('/v1/engine/shutdown', { method: 'POST' });
    });

    it('transcribe', async () => {
        const mock_blob = new Blob(['audio']);
        vi.mocked(api_request).mockResolvedValueOnce({ text: 'hello' });
        const res = await system_api_service.transcribe(mock_blob);
        expect(res).toBe('hello');
        expect(api_request).toHaveBeenCalledWith('/v1/engine/transcribe', expect.objectContaining({
            method: 'POST',
        }));
    });
    
    it('transcribe with no text', async () => {
        const mock_blob = new Blob(['audio']);
        vi.mocked(api_request).mockResolvedValueOnce({});
        const res = await system_api_service.transcribe(mock_blob);
        expect(res).toBe('');
    });

    describe('test_provider', () => {
        it('returns success if request passes', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok', latency: 100 });
            const res = await system_api_service.test_provider({ id: 'P', name: 'N', protocol: 'U' });
            expect(res).toEqual({ status: 'ok', latency: 100 });
        });

        it('returns timeout error string if TIMEOUT is thrown', async () => {
            vi.mocked(api_request).mockRejectedValueOnce('TIMEOUT');
            const res = await system_api_service.test_provider({ id: 'P', name: 'N', protocol: 'U' });
            expect(res).toEqual({ status: 'error', message: 'Handshake timeout: The provider endpoint is unresponsive.' });
        });

        it('returns generic error string if regular error is thrown', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('Bad Gateway'));
            const res = await system_api_service.test_provider({ id: 'P', name: 'N', protocol: 'U' });
            expect(res).toEqual({ status: 'error', message: 'Bad Gateway' });
        });
    });

    describe('Continuity Jobs', () => {
        it('get_scheduled_jobs handles array', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([{ id: '1' }]);
            const res = await system_api_service.get_scheduled_jobs();
            expect(res).toEqual([{ id: '1' }]);
        });

        it('get_scheduled_jobs handles envelope', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ jobs: [{ id: '2' }] });
            const res = await system_api_service.get_scheduled_jobs();
            expect(res).toEqual([{ id: '2' }]);
        });

        it('create_scheduled_job', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ id: 'new' });
            await system_api_service.create_scheduled_job({ name: 'job1' });
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/jobs', expect.objectContaining({ method: 'POST' }));
        });

        it('update_scheduled_job', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ id: 'updated' });
            await system_api_service.update_scheduled_job('j1', { name: 'job1' });
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/jobs/j1', expect.objectContaining({ method: 'PUT' }));
        });

        it('delete_scheduled_job', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await system_api_service.delete_scheduled_job('j1');
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/jobs/j1', expect.objectContaining({ method: 'DELETE' }));
        });

        it('get_scheduled_job_runs', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ runs: [{ id: 'r1' }] });
            const res = await system_api_service.get_scheduled_job_runs('j1');
            expect(res).toEqual([{ id: 'r1' }]);
            
            vi.mocked(api_request).mockResolvedValueOnce([{ id: 'r2' }]);
            const res_2 = await system_api_service.get_scheduled_job_runs('j1');
            expect(res_2).toEqual([{ id: 'r2' }]);
        });
    });

    describe('Oversight', () => {
        it('get_pending_oversight', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ data: [{ id: '1' }] });
            const res = await system_api_service.get_pending_oversight();
            expect(res).toEqual([{ id: '1' }]);
            
            vi.mocked(api_request).mockResolvedValueOnce([{ id: '2' }]);
            const res_2 = await system_api_service.get_pending_oversight();
            expect(res_2).toEqual([{ id: '2' }]);
        });

        it('get_oversight_ledger', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ data: [{ id: '1' }] });
            const res = await system_api_service.get_oversight_ledger();
            expect(res).toEqual([{ id: '1' }]);
        });

        it('decide_oversight', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await system_api_service.decide_oversight('1', 'approved');
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/1/decide', expect.objectContaining({ method: 'POST', body: '{"decision":"approved"}' }));
        });
    });

    describe('Quotas', () => {
        it('get_security_quotas', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ total_budget: 100 });
            const res = await system_api_service.get_security_quotas();
            expect(res).toEqual({ total_budget: 100 });
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/quotas', { method: 'GET' });
        });

        it('update_security_quota', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });
            await system_api_service.update_security_quota('e1', 50);
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/quotas/e1', expect.objectContaining({ method: 'PUT', body: '{"budget_usd":50}' }));
        });

        it('get_mission_quotas', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ quotas: [] });
            await system_api_service.get_mission_quotas();
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/missions/quotas', { method: 'GET' });
        });

        it('update_mission_quota', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });
            await system_api_service.update_mission_quota('c1', 10);
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/missions/c1/quota', expect.objectContaining({ method: 'PUT', body: '{"budget_usd":10}' }));
        });
    });

    describe('Other operations', () => {
        it('get_nodes', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await system_api_service.get_nodes();
            expect(api_request).toHaveBeenCalledWith('/v1/infra/nodes', { method: 'GET' });
        });
        
        it('discover_nodes', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({status: 'ok', discovered: []});
            await system_api_service.discover_nodes();
            expect(api_request).toHaveBeenCalledWith('/v1/infra/nodes/discover', { method: 'POST' });
        });
        
        it('get_benchmarks', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await system_api_service.get_benchmarks();
            expect(api_request).toHaveBeenCalledWith('/v1/benchmarks', { method: 'GET' });
        });
        
        it('run_benchmark', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({id: 'b1'});
            await system_api_service.run_benchmark('t1');
            expect(api_request).toHaveBeenCalledWith('/v1/benchmarks/run/t1', { method: 'POST' });
        });

        it('get_knowledge_docs', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await system_api_service.get_knowledge_docs();
            expect(api_request).toHaveBeenCalledWith('/v1/docs/knowledge', { method: 'GET' });
        });

        it('get_knowledge_doc', async () => {
            vi.mocked(api_request).mockResolvedValueOnce('# Doc');
            await system_api_service.get_knowledge_doc('cat', 'name');
            expect(api_request).toHaveBeenCalledWith('/v1/docs/knowledge/cat/name', expect.objectContaining({ method: 'GET' }));
        });

        it('get_operations_manual', async () => {
            vi.mocked(api_request).mockResolvedValueOnce('# Ops');
            await system_api_service.get_operations_manual();
            expect(api_request).toHaveBeenCalledWith('/v1/docs/operations-manual', expect.objectContaining({ method: 'GET' }));
        });

        it('get_providers', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await system_api_service.get_providers();
            expect(api_request).toHaveBeenCalledWith('/v1/infra/providers', { method: 'GET' });
        });

        it('update_provider', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({status: 'ok'});
            await system_api_service.update_provider('p1', {k: 'v'});
            expect(api_request).toHaveBeenCalledWith('/v1/infra/providers/p1', expect.objectContaining({ method: 'PUT' }));
        });
        
        it('get_models', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await system_api_service.get_models();
            expect(api_request).toHaveBeenCalledWith('/v1/infra/models', { method: 'GET' });
        });

        it('update_model', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({status: 'ok'});
            await system_api_service.update_model('m1', {k: 'v'});
            expect(api_request).toHaveBeenCalledWith('/v1/infra/models/m1', expect.objectContaining({ method: 'PUT' }));
        });

        it('get_audit_trail', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({data: [], total: 0});
            await system_api_service.get_audit_trail(2, 25);
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/audit-trail?page=2&per_page=25', { method: 'GET' });
        });

        it('get_agent_health', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({agents: []});
            await system_api_service.get_agent_health();
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/health', { method: 'GET' });
        });

        it('list_continuity_workflows', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await system_api_service.list_continuity_workflows();
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/workflows', { method: 'GET' });
        });

        it('create_continuity_workflows', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await system_api_service.create_continuity_workflows({name: 'w1'});
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/workflows', expect.objectContaining({ method: 'POST' }));
        });

        it('add_continuity_workflows_step', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await system_api_service.add_continuity_workflows_step('w1', {prompt: 'p1'});
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/workflows/w1/steps', expect.objectContaining({ method: 'POST' }));
        });

        it('delete_continuity_workflows', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await system_api_service.delete_continuity_workflows('w1');
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/workflows/w1', expect.objectContaining({ method: 'DELETE' }));
        });
    });
});

