/**
 * @file SystemApiService.test.ts
 * @description Suite for core system management, infrastructure, and engine control.
 * @module Services/SystemApiService
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
import { SystemApiService } from './SystemApiService';
import { apiRequest } from './BaseApiService';

vi.mock('./BaseApiService', () => ({
    apiRequest: vi.fn(),
    DEPLOY_TIMEOUT: 60000,
}));

describe('SystemApiService', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('checkHealth', () => {
        it('should return true if apiRequest succeeds', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            const res = await SystemApiService.checkHealth();
            expect(res).toBe(true);
            expect(apiRequest).toHaveBeenCalledWith('/v1/engine/health', { method: 'GET', timeout: 5000 });
        });

        it('should return false if apiRequest fails', async () => {
            vi.mocked(apiRequest).mockRejectedValueOnce(new Error('fail'));
            const res = await SystemApiService.checkHealth();
            expect(res).toBe(false);
        });
    });

    it('deployEngine', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({ status: 'ok' });
        await SystemApiService.deployEngine('target1');
        expect(apiRequest).toHaveBeenCalledWith('/v1/engine/deploy?target=target1', expect.objectContaining({ method: 'POST' }));
    });

    it('speak', async () => {
        const mockBlob = new Blob(['audio']);
        vi.mocked(apiRequest).mockResolvedValueOnce(mockBlob);
        const res = await SystemApiService.speak('hello', 'voice1', 'engine1');
        expect(res).toBe(mockBlob);
        expect(apiRequest).toHaveBeenCalledWith('/v1/engine/speak', expect.objectContaining({
            method: 'POST',
            body: JSON.stringify({ text: 'hello', voice: 'voice1', engine: 'engine1' }),
            responseType: 'blob'
        }));
    });

    it('killAgents', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        await SystemApiService.killAgents();
        expect(apiRequest).toHaveBeenCalledWith('/v1/engine/kill', { method: 'POST' });
    });

    it('shutdownEngine', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        await SystemApiService.shutdownEngine();
        expect(apiRequest).toHaveBeenCalledWith('/v1/engine/shutdown', { method: 'POST' });
    });

    it('transcribe', async () => {
        const mockBlob = new Blob(['audio']);
        vi.mocked(apiRequest).mockResolvedValueOnce({ text: 'hello' });
        const res = await SystemApiService.transcribe(mockBlob);
        expect(res).toBe('hello');
        expect(apiRequest).toHaveBeenCalledWith('/v1/engine/transcribe', expect.objectContaining({
            method: 'POST',
        }));
    });
    
    it('transcribe with no text', async () => {
        const mockBlob = new Blob(['audio']);
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        const res = await SystemApiService.transcribe(mockBlob);
        expect(res).toBe('');
    });

    describe('testProvider', () => {
        it('returns success if request passes', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ status: 'ok', latency: 100 });
            const res = await (SystemApiService as any).testProvider('P', 'K', 'U');
            expect(res).toEqual({ status: 'ok', latency: 100 });
        });

        it('returns timeout error string if TIMEOUT is thrown', async () => {
            vi.mocked(apiRequest).mockRejectedValueOnce('TIMEOUT');
            const res = await (SystemApiService as any).testProvider('P', 'K', 'U');
            expect(res).toEqual({ status: 'error', message: 'Handshake timeout: The provider endpoint is unresponsive.' });
        });

        it('returns generic error string if regular error is thrown', async () => {
            vi.mocked(apiRequest).mockRejectedValueOnce(new Error('Bad Gateway'));
            const res = await (SystemApiService as any).testProvider('P', 'K', 'U');
            expect(res).toEqual({ status: 'error', message: 'Bad Gateway' });
        });
    });

    describe('Continuity Jobs', () => {
        it('getScheduledJobs handles array', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce([{ id: '1' }]);
            const res = await SystemApiService.getScheduledJobs();
            expect(res).toEqual([{ id: '1' }]);
        });

        it('getScheduledJobs handles envelope', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ jobs: [{ id: '2' }] });
            const res = await SystemApiService.getScheduledJobs();
            expect(res).toEqual([{ id: '2' }]);
        });

        it('createScheduledJob', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ id: 'new' });
            await SystemApiService.createScheduledJob({ name: 'job1' });
            expect(apiRequest).toHaveBeenCalledWith('/v1/continuity/jobs', expect.objectContaining({ method: 'POST' }));
        });

        it('updateScheduledJob', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ id: 'updated' });
            await SystemApiService.updateScheduledJob('j1', { name: 'job1' });
            expect(apiRequest).toHaveBeenCalledWith('/v1/continuity/jobs/j1', expect.objectContaining({ method: 'PUT' }));
        });

        it('deleteScheduledJob', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await SystemApiService.deleteScheduledJob('j1');
            expect(apiRequest).toHaveBeenCalledWith('/v1/continuity/jobs/j1', expect.objectContaining({ method: 'DELETE' }));
        });

        it('getScheduledJobRuns', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ runs: [{ id: 'r1' }] });
            const res = await SystemApiService.getScheduledJobRuns('j1');
            expect(res).toEqual([{ id: 'r1' }]);
            
            vi.mocked(apiRequest).mockResolvedValueOnce([{ id: 'r2' }]);
            const res2 = await SystemApiService.getScheduledJobRuns('j1');
            expect(res2).toEqual([{ id: 'r2' }]);
        });
    });

    describe('Oversight', () => {
        it('getPendingOversight', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ data: [{ id: '1' }] });
            const res = await SystemApiService.getPendingOversight();
            expect(res).toEqual([{ id: '1' }]);
            
            vi.mocked(apiRequest).mockResolvedValueOnce([{ id: '2' }]);
            const res2 = await SystemApiService.getPendingOversight();
            expect(res2).toEqual([{ id: '2' }]);
        });

        it('getOversightLedger', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ data: [{ id: '1' }] });
            const res = await SystemApiService.getOversightLedger();
            expect(res).toEqual([{ id: '1' }]);
        });

        it('decideOversight', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await SystemApiService.decideOversight('1', 'approved');
            expect(apiRequest).toHaveBeenCalledWith('/v1/oversight/1/decide', expect.objectContaining({ method: 'POST', body: '{"decision":"approved"}' }));
        });
    });

    describe('Other operations', () => {
        it('getNodes', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce([]);
            await SystemApiService.getNodes();
            expect(apiRequest).toHaveBeenCalledWith('/v1/infra/nodes', { method: 'GET' });
        });
        
        it('discoverNodes', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({status: 'ok', discovered: []});
            await SystemApiService.discoverNodes();
            expect(apiRequest).toHaveBeenCalledWith('/v1/infra/nodes/discover', { method: 'POST' });
        });
        
        it('getBenchmarks', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce([]);
            await SystemApiService.getBenchmarks();
            expect(apiRequest).toHaveBeenCalledWith('/v1/benchmarks', { method: 'GET' });
        });
        
        it('runBenchmark', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({id: 'b1'});
            await SystemApiService.runBenchmark('t1');
            expect(apiRequest).toHaveBeenCalledWith('/v1/benchmarks/run/t1', { method: 'POST' });
        });

        it('getKnowledgeDocs', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce([]);
            await SystemApiService.getKnowledgeDocs();
            expect(apiRequest).toHaveBeenCalledWith('/v1/docs/knowledge', { method: 'GET' });
        });

        it('getKnowledgeDoc', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce('# Doc');
            await SystemApiService.getKnowledgeDoc('cat', 'name');
            expect(apiRequest).toHaveBeenCalledWith('/v1/docs/knowledge/cat/name', expect.objectContaining({ method: 'GET' }));
        });

        it('getOperationsManual', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce('# Ops');
            await SystemApiService.getOperationsManual();
            expect(apiRequest).toHaveBeenCalledWith('/v1/docs/operations-manual', expect.objectContaining({ method: 'GET' }));
        });

        it('getProviders', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce([]);
            await SystemApiService.getProviders();
            expect(apiRequest).toHaveBeenCalledWith('/v1/infra/providers', { method: 'GET' });
        });

        it('updateProvider', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({status: 'ok'});
            await SystemApiService.updateProvider('p1', {k: 'v'});
            expect(apiRequest).toHaveBeenCalledWith('/v1/infra/providers/p1', expect.objectContaining({ method: 'PUT' }));
        });
        
        it('getModels', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce([]);
            await SystemApiService.getModels();
            expect(apiRequest).toHaveBeenCalledWith('/v1/infra/models', { method: 'GET' });
        });

        it('updateModel', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({status: 'ok'});
            await SystemApiService.updateModel('m1', {k: 'v'});
            expect(apiRequest).toHaveBeenCalledWith('/v1/infra/models/m1', expect.objectContaining({ method: 'PUT' }));
        });

        it('getSecurityQuotas', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await SystemApiService.getSecurityQuotas();
            expect(apiRequest).toHaveBeenCalledWith('/v1/oversight/security/quotas', { method: 'GET' });
        });

        it('getAuditTrail', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({data: [], total: 0});
            await SystemApiService.getAuditTrail(2, 25);
            expect(apiRequest).toHaveBeenCalledWith('/v1/oversight/security/audit-trail?page=2&per_page=25', { method: 'GET' });
        });

        it('getAgentHealth', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({agents: []});
            await SystemApiService.getAgentHealth();
            expect(apiRequest).toHaveBeenCalledWith('/v1/oversight/security/health', { method: 'GET' });
        });

        it('listContinuityWorkflows', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce([]);
            await SystemApiService.listContinuityWorkflows();
            expect(apiRequest).toHaveBeenCalledWith('/v1/continuity/workflows', { method: 'GET' });
        });

        it('createContinuityWorkflow', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await SystemApiService.createContinuityWorkflow({name: 'w1'});
            expect(apiRequest).toHaveBeenCalledWith('/v1/continuity/workflows', expect.objectContaining({ method: 'POST' }));
        });

        it('addContinuityWorkflowStep', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await SystemApiService.addContinuityWorkflowStep('w1', {prompt: 'p1'});
            expect(apiRequest).toHaveBeenCalledWith('/v1/continuity/workflows/w1/steps', expect.objectContaining({ method: 'POST' }));
        });

        it('deleteContinuityWorkflow', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await SystemApiService.deleteContinuityWorkflow('w1');
            expect(apiRequest).toHaveBeenCalledWith('/v1/continuity/workflows/w1', expect.objectContaining({ method: 'DELETE' }));
        });
    });
});
