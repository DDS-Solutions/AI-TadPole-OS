/**
 * @file mission_api_service.test.ts
 * @description Suite for the Mission and Skill management API service.
 * @module Services/mission_api_service
 * @testedBehavior
 * - Skill Management: CRUD operations for agent skills and MCP tools.
 * - Workflow Orchestration: Deployment and lifecycle management of multi-step workflows.
 * - Discovery: Fetching of available platform skills and MCP integration points.
 * @aiContext
 * - Mocks apiRequest from base_api_service to isolate backend network calls.
 * - Validates JSON payload structures for PUT/POST requests to ensure contract compliance.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mission_api_service } from './mission_api_service';
import { apiRequest } from './base_api_service';

vi.mock('./base_api_service', () => ({
    apiRequest: vi.fn(),
}));

describe('mission_api_service', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('sync_mission', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        const mission = { id: 'm1' } as any;
        const res = await mission_api_service.sync_mission('agent-1', mission);
        expect(res).toBe(true);
        expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/mission', {
            method: 'POST',
            body: JSON.stringify(mission)
        });
    });

    it('get_skill_manifests', async () => {
        const mockSkills = [{ name: 'skill1' }];
        vi.mocked(apiRequest).mockResolvedValueOnce(mockSkills);
        const res = await mission_api_service.get_skill_manifests();
        expect(res).toEqual(mockSkills);
        expect(apiRequest).toHaveBeenCalledWith('/v1/skills/manifests', { method: 'GET' });
    });

    it('get_unified_skills', async () => {
        const mockSkills = { skills: [], workflows: [], manifests: [] };
        vi.mocked(apiRequest).mockResolvedValueOnce(mockSkills);
        const res = await mission_api_service.get_unified_skills();
        expect(res).toEqual(mockSkills);
        expect(apiRequest).toHaveBeenCalledWith('/v1/skills', { method: 'GET' });
    });

    it('save_skill_script', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        const manifest = { name: 's1', script: '...' };
        await mission_api_service.save_skill_script('s1', manifest);
        expect(apiRequest).toHaveBeenCalledWith('/v1/skills/scripts/s1', {
            method: 'PUT',
            body: JSON.stringify(manifest)
        });
    });

    it('delete_skill_script', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        await mission_api_service.delete_skill_script('s1');
        expect(apiRequest).toHaveBeenCalledWith('/v1/skills/scripts/s1', { method: 'DELETE' });
    });

    it('save_workflow', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        const manifest = { name: 'w1' };
        await mission_api_service.save_workflow('w1', manifest);
        expect(apiRequest).toHaveBeenCalledWith('/v1/skills/workflows/w1', {
            method: 'PUT',
            body: JSON.stringify(manifest)
        });
    });

    it('delete_workflow', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        await mission_api_service.delete_workflow('w1');
        expect(apiRequest).toHaveBeenCalledWith('/v1/skills/workflows/w1', { method: 'DELETE' });
    });

    it('get_mcp_tools', async () => {
        const mockTools = [{ name: 'tool1' }];
        vi.mocked(apiRequest).mockResolvedValueOnce(mockTools);
        const res = await mission_api_service.get_mcp_tools();
        expect(res).toEqual(mockTools);
        expect(apiRequest).toHaveBeenCalledWith('/v1/skills/mcp-tools', { method: 'GET' });
    });

    it('execute_mcp_tool', async () => {
        const mockRes = { result: 'success' };
        vi.mocked(apiRequest).mockResolvedValueOnce(mockRes);
        const res = await mission_api_service.execute_mcp_tool('tool1', { arg: 1 });
        expect(res).toEqual(mockRes);
        expect(apiRequest).toHaveBeenCalledWith('/v1/skills/mcp-tools/tool1/execute', {
            method: 'POST',
            body: JSON.stringify({ arg: 1 })
        });
    });
});
