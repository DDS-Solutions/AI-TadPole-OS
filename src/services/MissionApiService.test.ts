/**
 * @file MissionApiService.test.ts
 * @description Suite for the Mission and Capability management API service.
 * @module Services/MissionApiService
 * @testedBehavior
 * - Skill Management: CRUD operations for agent skills and MCP tools.
 * - Workflow Orchestration: Deployment and lifecycle management of multi-step workflows.
 * - Discovery: Fetching of available platform capabilities and MCP integration points.
 * @aiContext
 * - Mocks apiRequest from BaseApiService to isolate backend network calls.
 * - Validates JSON payload structures for PUT/POST requests to ensure contract compliance.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { MissionApiService } from './MissionApiService';
import { apiRequest } from './BaseApiService';

vi.mock('./BaseApiService', () => ({
    apiRequest: vi.fn(),
}));

describe('MissionApiService', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('syncMission', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        const mission = { id: 'm1' } as any;
        const res = await MissionApiService.syncMission('agent-1', mission);
        expect(res).toBe(true);
        expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/mission', {
            method: 'POST',
            body: JSON.stringify(mission)
        });
    });

    it('getSkills', async () => {
        const mockSkills = [{ name: 'skill1' }];
        vi.mocked(apiRequest).mockResolvedValueOnce(mockSkills);
        const res = await MissionApiService.getSkills();
        expect(res).toEqual(mockSkills);
        expect(apiRequest).toHaveBeenCalledWith('/v1/skills', { method: 'GET' });
    });

    it('getCapabilities', async () => {
        const mockCaps = { skills: [], workflows: [] };
        vi.mocked(apiRequest).mockResolvedValueOnce(mockCaps);
        const res = await MissionApiService.getCapabilities();
        expect(res).toEqual(mockCaps);
        expect(apiRequest).toHaveBeenCalledWith('/v1/capabilities', { method: 'GET' });
    });

    it('saveSkill', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        const manifest = { name: 's1' };
        await MissionApiService.saveSkill('s1', manifest);
        expect(apiRequest).toHaveBeenCalledWith('/v1/capabilities/skills/s1', {
            method: 'PUT',
            body: JSON.stringify(manifest)
        });
    });

    it('deleteSkill', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        await MissionApiService.deleteSkill('s1');
        expect(apiRequest).toHaveBeenCalledWith('/v1/capabilities/skills/s1', { method: 'DELETE' });
    });

    it('saveWorkflow', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        const manifest = { name: 'w1' };
        await MissionApiService.saveWorkflow('w1', manifest);
        expect(apiRequest).toHaveBeenCalledWith('/v1/capabilities/workflows/w1', {
            method: 'PUT',
            body: JSON.stringify(manifest)
        });
    });

    it('deleteWorkflow', async () => {
        vi.mocked(apiRequest).mockResolvedValueOnce({});
        await MissionApiService.deleteWorkflow('w1');
        expect(apiRequest).toHaveBeenCalledWith('/v1/capabilities/workflows/w1', { method: 'DELETE' });
    });

    it('getMcpTools', async () => {
        const mockTools = [{ name: 'tool1' }];
        vi.mocked(apiRequest).mockResolvedValueOnce(mockTools);
        const res = await MissionApiService.getMcpTools();
        expect(res).toEqual(mockTools);
        expect(apiRequest).toHaveBeenCalledWith('/v1/capabilities/mcp-tools', { method: 'GET' });
    });

    it('executeMcpTool', async () => {
        const mockRes = { result: 'success' };
        vi.mocked(apiRequest).mockResolvedValueOnce(mockRes);
        const res = await MissionApiService.executeMcpTool('tool1', { arg: 1 });
        expect(res).toEqual(mockRes);
        expect(apiRequest).toHaveBeenCalledWith('/v1/capabilities/mcp-tools/tool1/execute', {
            method: 'POST',
            body: JSON.stringify({ arg: 1 })
        });
    });
});
