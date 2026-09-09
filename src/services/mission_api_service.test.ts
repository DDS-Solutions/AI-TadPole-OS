/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / mission_api_service.test
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
import { mission_api_service } from './mission_api_service';
import { api_request } from './base_api_service';

vi.mock('./base_api_service', async (importOriginal) => {
    const actual = await importOriginal<any>();
    return {
        ...actual,
        api_request: vi.fn(),
    };
});

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
