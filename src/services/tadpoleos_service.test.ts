/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Comprehensive verification of the primary Tadpole OS service aggregator.** 
 * Verifies the orchestration between specialized API clients (Agents, Mission, System) and the global state stores. 
 * Mocks sub-services (`agent_api`, `mission_api`, `system_api`) to verify correct domain delegation and unified interface signatures.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Service initialization deadlock if one of the underlying API clients fails to hydrate or failure to clear stale cache on logout.
 * - **Telemetry Link**: Search `[tadpoleos_service.test]` in tracing logs.
 */


/**
 * @file tadpoleos_service.test.ts
 * @description Suite for the Tadpole OS Service Facade.
 * @module Services/tadpole_os_service
 * @testedBehavior
 * - Domain Delegation: Verification that the facade correctly routes calls to Agent, Mission, and System sub-services.
 * - Unified Interface: Validation of the high-level API signatures used by the frontend components.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks sub-services (agent_api, mission_api, system_api) to verify delegation logic.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { describe, it, expect, vi } from 'vitest';
import { tadpole_os_service } from './tadpoleos_service';
import { agent_api_service } from './agent_api_service';
import { mission_api_service } from './mission_api_service';
import { system_api_service } from './system_api_service';

vi.mock('./agent_api_service');
vi.mock('./mission_api_service');
vi.mock('./system_api_service');

describe('tadpole_os_service Facade', () => {
    it('delegates Agent domain calls correctly', async () => {
        const mock_agents = [{ id: '1' }] as any;
        (agent_api_service.get_agents as any).mockResolvedValue(mock_agents);
        
        const result = await tadpole_os_service.get_agents();
        expect(agent_api_service.get_agents).toHaveBeenCalled();
        expect(result).toBe(mock_agents);
    });

    it('delegates Mission domain calls correctly', async () => {
        const mock_skills = [{ name: 'test' }] as any;
        (mission_api_service.get_unified_skills as any).mockResolvedValue(mock_skills);

        const result = await tadpole_os_service.get_unified_skills();
        expect(mission_api_service.get_unified_skills).toHaveBeenCalled();
        expect(result).toBe(mock_skills);
    });

    it('delegates System domain calls correctly', async () => {
        (system_api_service.check_health as any).mockResolvedValue({ status: 'ok' });

        const result = await tadpole_os_service.check_health();
        expect(system_api_service.check_health).toHaveBeenCalled();
        expect(result).toEqual({ status: 'ok' });
    });

    it('delegates execute_mcp_tool correctly', async () => {
        (mission_api_service.execute_mcp_tool as any).mockResolvedValue({ output: 'success' });

        const result = await tadpole_os_service.execute_mcp_tool('tool-1', {});
        expect(mission_api_service.execute_mcp_tool).toHaveBeenCalledWith('tool-1', {});
        expect(result).toEqual({ output: 'success' });
    });

    it('delegates get_audit_trail correctly', async () => {
        (system_api_service.get_audit_trail as any).mockResolvedValue([]);

        await tadpole_os_service.get_audit_trail();
        expect(system_api_service.get_audit_trail).toHaveBeenCalled();
    });

    it('delegates scheduled job operations correctly', async () => {
        (system_api_service.get_scheduled_jobs as any).mockResolvedValue([]);
        await tadpole_os_service.get_scheduled_jobs();
        expect(system_api_service.get_scheduled_jobs).toHaveBeenCalled();

        (system_api_service.create_scheduled_job as any).mockResolvedValue({ success: true });
        await tadpole_os_service.create_scheduled_job({} as any);
        expect(system_api_service.create_scheduled_job).toHaveBeenCalled();
    });

    it('delegates benchmark operations correctly', async () => {
        (system_api_service.run_benchmark as any).mockResolvedValue({ id: 'bench-1' });
        await tadpole_os_service.run_benchmark('llm');
        expect(system_api_service.run_benchmark).toHaveBeenCalledWith('llm');
    });

    it('delegates agent memory operations correctly', async () => {
        (agent_api_service.save_agent_memory as any).mockResolvedValue({ success: true });
        await tadpole_os_service.save_agent_memory('agent-1', 'content');
        expect(agent_api_service.save_agent_memory).toHaveBeenCalledWith('agent-1', 'content');
    });

    it('verifies all proxy methods are correctly mapped', async () => {
        const services = { agent_api_service, mission_api_service, system_api_service };
        
        // Define which methods belong to which sub-service for verification
        const mapping: Record<string, keyof typeof services> = {};
        
        // Populate mapping based on current file structure
        Object.keys(agent_api_service).forEach(k => mapping[k] = 'agent_api_service');
        Object.keys(mission_api_service).forEach(k => mapping[k] = 'mission_api_service');
        Object.keys(system_api_service).forEach(k => mapping[k] = 'system_api_service');

        const keys = Object.keys(tadpole_os_service) as (keyof typeof tadpole_os_service)[];
        
        for (const key of keys) {
            const method = tadpole_os_service[key];
            if (typeof method !== 'function') continue;
            if (key === 'request' || key === 'get_headers' || key === 'resolve_provider') continue;

            const target_service_name = mapping[key as string];
            if (!target_service_name) continue;

            const target_service = services[target_service_name];
            (target_service[key as keyof typeof target_service] as any).mockClear();
            (target_service[key as keyof typeof target_service] as any).mockResolvedValue({ ok: true });

            await (method as any)();
            expect(target_service[key as keyof typeof target_service]).toHaveBeenCalled();
        }
    });
});


// Metadata: [tadpoleos_service_test]

// Metadata: [tadpoleos_service_test]
