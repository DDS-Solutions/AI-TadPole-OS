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
        const mockAgents = [{ id: '1' }] as any;
        (agent_api_service.get_agents as any).mockResolvedValue(mockAgents);
        
        const result = await tadpole_os_service.get_agents();
        expect(agent_api_service.get_agents).toHaveBeenCalled();
        expect(result).toBe(mockAgents);
    });

    it('delegates Mission domain calls correctly', async () => {
        const mockSkills = [{ name: 'test' }] as any;
        (mission_api_service.get_unified_skills as any).mockResolvedValue(mockSkills);

        const result = await tadpole_os_service.get_unified_skills();
        expect(mission_api_service.get_unified_skills).toHaveBeenCalled();
        expect(result).toBe(mockSkills);
    });

    it('delegates System domain calls correctly', async () => {
        (system_api_service.check_health as any).mockResolvedValue({ status: 'ok' });

        const result = await tadpole_os_service.check_health();
        expect(system_api_service.check_health).toHaveBeenCalled();
        expect(result).toEqual({ status: 'ok' });
    });

    it('delegates execute_mcp_tool correctly', async () => {
        (mission_api_service.execute_mcp_tool as any).mockResolvedValue({ output: 'success' });

        const result = await tadpole_os_service.execute_mcp_tool('agent-1', 'tool-1', {});
        expect(mission_api_service.execute_mcp_tool).toHaveBeenCalledWith('agent-1', 'tool-1', {});
        expect(result).toEqual({ output: 'success' });
    });

    it('delegates get_audit_trail correctly', async () => {
        (system_api_service.get_audit_trail as any).mockResolvedValue([]);

        await tadpole_os_service.get_audit_trail();
        expect(system_api_service.get_audit_trail).toHaveBeenCalled();
    });
});

