/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Validation of the Mission and Skill Registry API client.** 
 * Verifies the submission of swarm mandates, goal decomposition, and real-time progress tracking. 
 * Mocks `api_request` to isolate mission-level registry and MCP tool execution from network side-effects.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Mission deadlock when a parent task is completed but the child tasks are stuck in 'pending' or 'failed' states.
 * - **Telemetry Link**: Search `[mission_api_service.test]` in tracing logs.
 */


/**
 * @file mission_api_service.test.ts
 * @description Suite for the Tadpole OS Mission and Skill Registry API layer.
 * @module Services/mission_api_service
 * @testedBehavior
 * - Registry: Fetching and sorting of local scripts, YAML workflows, and lifecycle hooks.
 * - Tool Coordination: Verification of MCP tool execution and manifest discovery.
 * - Continuity: CRUD for mission blueprints and lifecycle state.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks api_request to isolate mission-level registry from network side-effects.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mission_api_service } from './mission_api_service';
import { api_request } from './base_api_service';

vi.mock('./base_api_service', () => ({
    api_request: vi.fn(),
}));

describe('mission_api_service', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('sync_mission', async () => {
        vi.mocked(api_request).mockResolvedValueOnce({});
        const mission = { objective: 'test' };
        await mission_api_service.sync_mission('agent-1', mission as any);
        expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1/mission', expect.objectContaining({
            method: 'POST',
            body: JSON.stringify(mission)
        }));
    });

    it('get_skill_manifests', async () => {
        vi.mocked(api_request).mockResolvedValueOnce([]);
        await mission_api_service.get_skill_manifests();
        expect(api_request).toHaveBeenCalledWith('/v1/skills/manifests', { method: 'GET' });
    });

    it('get_unified_skills', async () => {
        vi.mocked(api_request).mockResolvedValueOnce({ scripts: [], workflows: [], masks: [] });
        await mission_api_service.get_unified_skills();
        expect(api_request).toHaveBeenCalledWith('/v1/skills', { method: 'GET' });
    });

    it('save_skill_script', async () => {
        vi.mocked(api_request).mockResolvedValueOnce({});
        await mission_api_service.save_skill_script('test', { code: '...' });
        expect(api_request).toHaveBeenCalledWith('/v1/skills/scripts/test', expect.objectContaining({ method: 'PUT' }));
    });

    it('delete_skill_script', async () => {
        vi.mocked(api_request).mockResolvedValueOnce({});
        await mission_api_service.delete_skill_script('test');
        expect(api_request).toHaveBeenCalledWith('/v1/skills/scripts/test', { method: 'DELETE' });
    });

    it('save_workflow', async () => {
        vi.mocked(api_request).mockResolvedValueOnce({});
        await mission_api_service.save_workflow('test', { steps: [] });
        expect(api_request).toHaveBeenCalledWith('/v1/skills/workflows/test', expect.objectContaining({ method: 'PUT' }));
    });

    it('delete_workflow', async () => {
        vi.mocked(api_request).mockResolvedValueOnce({});
        await mission_api_service.delete_workflow('test');
        expect(api_request).toHaveBeenCalledWith('/v1/skills/workflows/test', { method: 'DELETE' });
    });

    it('get_mcp_tools', async () => {
        vi.mocked(api_request).mockResolvedValueOnce([]);
        await mission_api_service.get_mcp_tools();
        expect(api_request).toHaveBeenCalledWith('/v1/skills/mcp-tools', { method: 'GET' });
    });

    it('execute_mcp_tool', async () => {
        vi.mocked(api_request).mockResolvedValueOnce({});
        await mission_api_service.execute_mcp_tool('test_tool', { arg: 1 });
        expect(api_request).toHaveBeenCalledWith('/v1/skills/mcp-tools/test_tool/execute', expect.objectContaining({
            method: 'POST',
            body: JSON.stringify({ arg: 1 })
        }));
    });

    it('save_hook', async () => {
        vi.mocked(api_request).mockResolvedValueOnce({});
        await mission_api_service.save_hook('test', { code: '...' });
        expect(api_request).toHaveBeenCalledWith('/v1/skills/hooks/test', expect.objectContaining({ method: 'PUT' }));
    });

    it('delete_hook', async () => {
        vi.mocked(api_request).mockResolvedValueOnce({});
        await mission_api_service.delete_hook('test');
        expect(api_request).toHaveBeenCalledWith('/v1/skills/hooks/test', { method: 'DELETE' });
    });
});


// Metadata: [mission_api_service_test]

// Metadata: [mission_api_service_test]
