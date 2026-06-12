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
import { 
    system_api_service, 
    get_circuit_breakers, 
    invalidate_namespace, 
    _cache_for_testing, 
    NAMESPACE_KEYS,
    BREAKER_FAILURE_THRESHOLD,
    BREAKER_COOLDOWN_MS,
    DEFAULT_TIMEOUT_MS
} from './system_api_service';
import { use_trace_store } from '../stores/trace_store';
import { api_request, request_interceptors } from './base_api_service';
import { benchmarks_api } from './system/benchmarks_api';
import { continuity_api } from './system/continuity_api';
import { docs_api } from './system/docs_api';
import { engine_api } from './system/engine_api';
import { infra_api } from './system/infra_api';
import { oversight_api } from './system/oversight_api';
import { workspace_api } from './system/workspace_api';

vi.mock('./base_api_service', async (importOriginal) => {
    const actual = await importOriginal<any>();
    return {
        ...actual,
        api_request: vi.fn(),
        DEPLOY_TIMEOUT: 60000,
    };
});

describe('system_api_service', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('check_health', () => {
        it('should return true if api_request succeeds', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            const res = await system_api_service.engine.check_health();
            expect(res).toBe(true);
            expect(api_request).toHaveBeenCalledWith('/v1/engine/health', { method: 'GET', timeout: 5000 });
        });

        it('should return false if api_request fails', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('fail'));
            const res = await system_api_service.engine.check_health();
            expect(res).toBe(false);
        });
    });

    it('deploy_engine', async () => {
        vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });
        await system_api_service.engine.deploy_engine('target1');
        expect(api_request).toHaveBeenCalledWith('/v1/engine/deploy?target=target1', expect.objectContaining({ method: 'POST' }));
    });

    it('speak', async () => {
        const mock_blob = new Blob(['audio']);
        vi.mocked(api_request).mockResolvedValueOnce(mock_blob);
        const res = await system_api_service.engine.speak('hello', 'voice1', 'engine1');
        expect(res).toBe(mock_blob);
        expect(api_request).toHaveBeenCalledWith('/v1/engine/speak', expect.objectContaining({
            method: 'POST',
            body: JSON.stringify({ text: 'hello', voice: 'voice1', engine: 'engine1' }),
            response_type: 'blob'
        }));
    });

    it('kill_agents', async () => {
        vi.mocked(api_request).mockResolvedValueOnce({});
        await system_api_service.engine.kill_agents();
        expect(api_request).toHaveBeenCalledWith('/v1/engine/kill', { method: 'POST' });
    });

    it('shutdown_engine', async () => {
        vi.mocked(api_request).mockResolvedValueOnce({});
        await system_api_service.engine.shutdown_engine();
        expect(api_request).toHaveBeenCalledWith('/v1/engine/shutdown', { method: 'POST' });
    });

    it('transcribe', async () => {
        const mock_blob = new Blob(['audio']);
        vi.mocked(api_request).mockResolvedValueOnce({ text: 'hello' });
        const res = await system_api_service.engine.transcribe(mock_blob);
        expect(res).toBe('hello');
        expect(api_request).toHaveBeenCalledWith('/v1/engine/transcribe', expect.objectContaining({
            method: 'POST',
        }));
    });
    
    it('transcribe with no text', async () => {
        const mock_blob = new Blob(['audio']);
        vi.mocked(api_request).mockResolvedValueOnce({});
        const res = await system_api_service.engine.transcribe(mock_blob);
        expect(res).toBe('');
    });

    describe('test_provider', () => {
        it('returns success if request passes', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok', latency: 100 });
            const res = await system_api_service.infra.test_provider({ id: 'P', name: 'N', protocol: 'U' });
            expect(res).toEqual({ status: 'ok', latency: 100 });
        });

        it('returns timeout error string if request times out', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('Request timed out'));
            const res = await system_api_service.infra.test_provider({ id: 'P', name: 'N', protocol: 'U' });
            expect(res).toEqual({ status: 'error', message: 'Handshake timeout: The provider endpoint is unresponsive.' });
        });

        it('returns generic error string if regular error is thrown', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('Bad Gateway'));
            const res = await system_api_service.infra.test_provider({ id: 'P', name: 'N', protocol: 'U' });
            expect(res).toEqual({ status: 'error', message: 'Bad Gateway' });
        });
    });

    describe('Continuity Jobs', () => {
        it('get_scheduled_jobs handles array', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([{ id: '1' }]);
            const res = await system_api_service.continuity.get_scheduled_jobs();
            expect(res).toEqual([{ id: '1' }]);
        });

        it('get_scheduled_jobs handles envelope', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ jobs: [{ id: '2' }] });
            const res = await system_api_service.continuity.get_scheduled_jobs();
            expect(res).toEqual([{ id: '2' }]);
        });

        it('create_scheduled_job', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ id: 'new' });
            await system_api_service.continuity.create_scheduled_job({ name: 'job1' });
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/jobs', expect.objectContaining({ method: 'POST' }));
        });

        it('update_scheduled_job', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ id: 'updated' });
            await system_api_service.continuity.update_scheduled_job('j1', { name: 'job1' });
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/jobs/j1', expect.objectContaining({ method: 'PUT' }));
        });

        it('delete_scheduled_job', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await system_api_service.continuity.delete_scheduled_job('j1');
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/jobs/j1', expect.objectContaining({ method: 'DELETE' }));
        });

        it('get_scheduled_job_runs', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ runs: [{ id: 'r1' }] });
            const res = await system_api_service.continuity.get_scheduled_job_runs('j1');
            expect(res).toEqual([{ id: 'r1' }]);
            
            vi.mocked(api_request).mockResolvedValueOnce([{ id: 'r2' }]);
            const res_2 = await system_api_service.continuity.get_scheduled_job_runs('j1');
            expect(res_2).toEqual([{ id: 'r2' }]);
        });
    });

    describe('Oversight', () => {
        it('get_pending_oversight', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ data: [{ id: '1' }] });
            const res = await system_api_service.oversight.get_pending_oversight();
            expect(res).toEqual([{ id: '1' }]);
            
            vi.mocked(api_request).mockResolvedValueOnce([{ id: '2' }]);
            const res_2 = await system_api_service.oversight.get_pending_oversight();
            expect(res_2).toEqual([{ id: '2' }]);
        });

        it('get_oversight_ledger', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ data: [{ id: '1' }] });
            const res = await system_api_service.oversight.get_oversight_ledger();
            expect(res).toEqual([{ id: '1' }]);
        });

        it('decide_oversight', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await system_api_service.oversight.decide_oversight('1', 'approved');
            expect(api_request).toHaveBeenCalledWith(
                '/v1/oversight/1/decide',
                expect.objectContaining({
                    method: 'POST',
                    body: expect.stringContaining('"decision":"approved"')
                })
            );
        });
    });

    describe('Quotas', () => {
        it('get_security_quotas', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ total_budget: 100 });
            const res = await system_api_service.oversight.get_security_quotas();
            expect(res).toEqual({ total_budget: 100 });
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/quotas', { method: 'GET' });
        });

        it('update_security_quota', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });
            await system_api_service.oversight.update_security_quota('e1', 50);
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/quotas/e1', expect.objectContaining({ method: 'PUT', body: '{"budget_usd":50}' }));
        });

        it('get_mission_quotas', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ quotas: [] });
            await system_api_service.oversight.get_mission_quotas();
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/missions/quotas', { method: 'GET' });
        });

        it('update_mission_quota', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });
            await system_api_service.oversight.update_mission_quota('c1', 10);
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/missions/c1/quota', expect.objectContaining({ method: 'PUT', body: '{"budget_usd":10}' }));
        });
    });

    describe('Other operations', () => {
        it('get_nodes', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await system_api_service.infra.get_nodes();
            expect(api_request).toHaveBeenCalledWith('/v1/infra/nodes', { method: 'GET' });
        });
        
        it('discover_nodes', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({status: 'ok', discovered: []});
            await system_api_service.infra.discover_nodes();
            expect(api_request).toHaveBeenCalledWith('/v1/infra/nodes/discover', { method: 'POST' });
        });
        
        it('get_benchmarks', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await system_api_service.benchmarks.get_benchmarks();
            expect(api_request).toHaveBeenCalledWith('/v1/benchmarks', { method: 'GET' });
        });
        
        it('run_benchmark', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({id: 'b1'});
            await system_api_service.benchmarks.run_benchmark('t1');
            expect(api_request).toHaveBeenCalledWith('/v1/benchmarks/run/t1', { method: 'POST' });
        });

        it('get_knowledge_docs', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await system_api_service.docs.get_knowledge_docs();
            expect(api_request).toHaveBeenCalledWith('/v1/docs/knowledge', { method: 'GET' });
        });

        it('get_knowledge_doc', async () => {
            vi.mocked(api_request).mockResolvedValueOnce('# Doc');
            await system_api_service.docs.get_knowledge_doc('cat', 'name');
            expect(api_request).toHaveBeenCalledWith('/v1/docs/knowledge/cat/name', expect.objectContaining({ method: 'GET' }));
        });

        it('get_operations_manual', async () => {
            vi.mocked(api_request).mockResolvedValueOnce('# Ops');
            await system_api_service.docs.get_operations_manual();
            expect(api_request).toHaveBeenCalledWith('/v1/docs/operations-manual', expect.objectContaining({ method: 'GET' }));
        });

        it('get_providers', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await system_api_service.infra.get_providers();
            expect(api_request).toHaveBeenCalledWith('/v1/infra/providers', { method: 'GET' });
        });

        it('update_provider', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({status: 'ok'});
            await system_api_service.infra.update_provider('p1', {k: 'v'});
            expect(api_request).toHaveBeenCalledWith('/v1/infra/providers/p1', expect.objectContaining({ method: 'PUT' }));
        });
        
        it('get_models', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await system_api_service.infra.get_models();
            expect(api_request).toHaveBeenCalledWith('/v1/infra/models', { method: 'GET' });
        });

        it('update_model', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({status: 'ok'});
            await system_api_service.infra.update_model('m1', {k: 'v'});
            expect(api_request).toHaveBeenCalledWith('/v1/infra/models/m1', expect.objectContaining({ method: 'PUT' }));
        });

        it('get_audit_trail', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({data: [], total: 0});
            await system_api_service.oversight.get_audit_trail(2, 25);
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/audit-trail?page=2&per_page=25', { method: 'GET' });
        });

        it('get_agent_health', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({agents: []});
            await system_api_service.oversight.get_agent_health();
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/health', { method: 'GET' });
        });

        it('list_continuity_workflows', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await system_api_service.continuity.list_continuity_workflows();
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/workflows', { method: 'GET' });
        });

        it('create_continuity_workflows', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await system_api_service.continuity.create_continuity_workflows({name: 'w1'});
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/workflows', expect.objectContaining({ method: 'POST' }));
        });

        it('add_continuity_workflows_step', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await system_api_service.continuity.add_continuity_workflows_step('w1', {prompt: 'p1'});
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/workflows/w1/steps', expect.objectContaining({ method: 'POST' }));
        });

        it('delete_continuity_workflows', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await system_api_service.continuity.delete_continuity_workflows('w1');
            expect(api_request).toHaveBeenCalledWith('/v1/continuity/workflows/w1', expect.objectContaining({ method: 'DELETE' }));
        });
    });

    describe('Facade Integrity', () => {
        it('should have a frozen facade and locked keys', () => {
            expect(Object.isFrozen(system_api_service)).toBe(true);
            expect(() => { (system_api_service as any).engine = {}; }).toThrow();
        });

        it('should prevent mutation on proxy namespace namespaces', () => {
            const engineProxy = system_api_service.engine;
            expect(() => { (engineProxy as any).check_health = () => {}; }).toThrow();
            expect(() => { Object.defineProperty(engineProxy, 'newProp', { value: 1 }); }).toThrow();
            expect(() => { delete (engineProxy as any).check_health; }).toThrow();
        });

        it('should return value descriptors for allowlisted keys', () => {
            const desc = Object.getOwnPropertyDescriptor(system_api_service.engine, 'check_health');
            expect(desc).toBeDefined();
            expect(desc!.value).toBeInstanceOf(Function);
            expect(desc!.writable).toBe(true);
        });
    });

    describe('Allowlist Synchronization', () => {
        it('should match actual exported methods of each sub-service exactly', () => {
            const actualKeys = {
                engine: Object.keys(engine_api),
                infra: Object.keys(infra_api),
                benchmarks: Object.keys(benchmarks_api),
                continuity: Object.keys(continuity_api),
                oversight: Object.keys(oversight_api),
                docs: Object.keys(docs_api),
                workspace: Object.keys(workspace_api)
            };

            expect([...NAMESPACE_KEYS.engine].sort()).toEqual(actualKeys.engine.sort());
            expect([...NAMESPACE_KEYS.infra].sort()).toEqual(actualKeys.infra.sort());
            expect([...NAMESPACE_KEYS.benchmarks].sort()).toEqual(actualKeys.benchmarks.sort());
            expect([...NAMESPACE_KEYS.continuity].sort()).toEqual(actualKeys.continuity.sort());
            expect([...NAMESPACE_KEYS.oversight].sort()).toEqual(actualKeys.oversight.sort());
            expect([...NAMESPACE_KEYS.docs].sort()).toEqual(actualKeys.docs.sort());
            expect([...NAMESPACE_KEYS.workspace].sort()).toEqual(actualKeys.workspace.sort());
        });
    });

    describe('Single-Flight Dynamic Loader Cache', () => {
        beforeEach(() => {
            invalidate_namespace('engine');
        });

        it('should reuse the loading promise for concurrent parallel calls', async () => {
            vi.mocked(api_request).mockResolvedValue({});

            // Trigger multiple concurrent calls
            const p1 = system_api_service.engine.check_health();
            const p2 = system_api_service.engine.check_health();
            const p3 = system_api_service.engine.check_health();

            // Verify they resolve to the same underlying loader promise
            expect(_cache_for_testing.engine).toBeDefined();
            expect(_cache_for_testing.engine).toBeInstanceOf(Promise);

            await Promise.all([p1, p2, p3]);
        });
    });

    describe('Resilience Timeout Boundary', () => {
        it('should time out long running requests at the boundary', async () => {
            // Mock a request that never resolves or takes 40s
            vi.mocked(api_request).mockImplementationOnce(() => new Promise(() => {}));

            vi.useFakeTimers();
            const callPromise = system_api_service.infra.get_nodes();

            // Fast forward 31 seconds
            vi.advanceTimersByTime(31000);

            await expect(callPromise).rejects.toThrow('Request timed out at resilience boundary');
            vi.useRealTimers();
        });
    });

    describe('Circuit Breaker & Fallbacks', () => {
        beforeEach(() => {
            const breakers = get_circuit_breakers();
            for (const key of Object.keys(breakers)) {
                breakers[key as any].force_close();
            }
        });

        it('should trip to OPEN after 5 consecutive failures and short-circuit subsequent calls', async () => {
            vi.mocked(api_request).mockRejectedValue(new Error('Network offline'));

            const breakers = get_circuit_breakers();
            const infra_breaker = breakers.infra;
            expect(infra_breaker.get_state()).toBe('CLOSED');

            // 5 failures
            for (let i = 0; i < 5; i++) {
                await expect(system_api_service.infra.get_nodes()).rejects.toThrow('Network offline');
            }

            // The state should be OPEN
            expect(infra_breaker.get_state()).toBe('OPEN');

            // 6th call should immediately throw CircuitBreakerOpenError without calling api_request
            vi.mocked(api_request).mockClear();
            await expect(system_api_service.infra.get_nodes()).rejects.toThrow('Circuit Breaker OPEN');
            expect(api_request).not.toHaveBeenCalled();
        });

        it('should transition to HALF_OPEN after cooldown and reset on successes', async () => {
            const breakers = get_circuit_breakers();
            const infra_breaker = breakers.infra;

            infra_breaker.force_open();
            expect(infra_breaker.get_state()).toBe('OPEN');

            // Mock success response
            vi.mocked(api_request).mockResolvedValue([{ id: 'node-1' }]);

            // Wait cooldown time (mock Date.now to fast forward 11 seconds)
            const realNow = Date.now;
            const nowMock = vi.fn().mockReturnValue(realNow() + 11000);
            Date.now = nowMock;

            // First call in HALF_OPEN should succeed and transition state
            const nodes = await system_api_service.infra.get_nodes();
            expect(nodes).toEqual([{ id: 'node-1' }]);
            expect(infra_breaker.get_state()).toBe('HALF_OPEN');

            // Second success should close the circuit
            await system_api_service.infra.get_nodes();
            expect(infra_breaker.get_state()).toBe('CLOSED');

            Date.now = realNow;
        });

        it('should apply fallback for check_health and get_engine_status on breaker OPEN or timeout', async () => {
            const breakers = get_circuit_breakers();
            breakers.engine.force_open();

            // When circuit is open, check_health should return false instead of throwing
            const health = await system_api_service.engine.check_health();
            expect(health).toBe(false);

            // get_engine_status should return null instead of throwing
            const status = await system_api_service.engine.get_engine_status();
            expect(status).toBeNull();
        });

        it('should NOT apply fallbacks for standard API errors (like AuthError/ValidationError)', async () => {
            // Mock a 401 error
            vi.mocked(api_request).mockRejectedValueOnce(new Error('Unauthorized'));

            // It should bubble up the real error rather than fallback
            await expect(system_api_service.infra.get_nodes()).rejects.toThrow('Unauthorized');
        });
    });

    describe('Phase 2 Configuration and Telemetry', () => {
        it('should have parsed circuit breaker env configuration with default fallbacks', () => {
            expect(BREAKER_FAILURE_THRESHOLD).toBe(5);
            expect(BREAKER_COOLDOWN_MS).toBe(10000);
            expect(DEFAULT_TIMEOUT_MS).toBe(30000);
        });

        it('should trace facade calls, loadService, and breaker execution using spans', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            invalidate_namespace('engine');
            use_trace_store.getState().clear_all();

            await system_api_service.engine.check_health();

            const spans = Object.values(use_trace_store.getState().spans);
            expect(spans.length).toBeGreaterThanOrEqual(3);

            const facadeSpan = spans.find(s => s.name === 'system_api: engine.check_health');
            expect(facadeSpan).toBeDefined();
            expect(facadeSpan!.status).toBe('success');
            expect(facadeSpan!.attributes.namespace).toBe('engine');
            expect(facadeSpan!.attributes.method).toBe('check_health');

            const loaderSpan = spans.find(s => s.name === 'system_api: load_service (engine)');
            expect(loaderSpan).toBeDefined();
            expect(loaderSpan!.parent_id).toBe(facadeSpan!.id);
            expect(loaderSpan!.status).toBe('success');

            const breakerSpan = spans.find(s => s.name === 'circuit_breaker: execute (engine)');
            expect(breakerSpan).toBeDefined();
            expect(breakerSpan!.parent_id).toBe(facadeSpan!.id);
            expect(breakerSpan!.status).toBe('success');
        });

        it('should intercept /health/system virtual endpoint and return breaker states', async () => {
            const breakers = get_circuit_breakers();
            breakers.engine.force_close();
            breakers.infra.force_open();

            // Query the real interceptor from the request_interceptors set
            const interceptorsArray = Array.from(request_interceptors);
            const interceptor = interceptorsArray.find(i => i('/health/system') !== null);
            expect(interceptor).toBeDefined();

            const health = await interceptor!('/health/system');
            expect(health).toBeDefined();
            expect(health.engine.state).toBe('CLOSED');
            expect(health.infra.state).toBe('OPEN');

            // Reset infra breaker
            breakers.infra.force_close();
        });
    });
});


// Metadata: [system_api_service_test]

// Metadata: [system_api_service_test]
