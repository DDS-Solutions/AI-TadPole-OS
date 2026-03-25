import { describe, it, expect, vi } from 'vitest';
import { TadpoleOSService } from './tadpoleosService';
import { AgentApiService } from './AgentApiService';
import { MissionApiService } from './MissionApiService';
import { SystemApiService } from './SystemApiService';

vi.mock('./AgentApiService');
vi.mock('./MissionApiService');
vi.mock('./SystemApiService');

describe('TadpoleOSService Facade', () => {
    it('delegates Agent domain calls correctly', async () => {
        const mockAgents = [{ id: '1' }] as any;
        (AgentApiService.getAgents as any).mockResolvedValue(mockAgents);
        
        const result = await TadpoleOSService.getAgents();
        expect(AgentApiService.getAgents).toHaveBeenCalled();
        expect(result).toBe(mockAgents);
    });

    it('delegates Mission domain calls correctly', async () => {
        const mockSkills = [{ name: 'test' }] as any;
        (MissionApiService.getUnifiedSkills as any).mockResolvedValue(mockSkills);

        const result = await TadpoleOSService.getUnifiedSkills();
        expect(MissionApiService.getUnifiedSkills).toHaveBeenCalled();
        expect(result).toBe(mockSkills);
    });

    it('delegates System domain calls correctly', async () => {
        (SystemApiService.checkHealth as any).mockResolvedValue({ status: 'ok' });

        const result = await TadpoleOSService.checkHealth();
        expect(SystemApiService.checkHealth).toHaveBeenCalled();
        expect(result).toEqual({ status: 'ok' });
    });

    it('delegates executeMcpTool correctly', async () => {
        (MissionApiService.executeMcpTool as any).mockResolvedValue({ output: 'success' });

        const result = await TadpoleOSService.executeMcpTool('agent-1', 'tool-1', {});
        expect(MissionApiService.executeMcpTool).toHaveBeenCalledWith('agent-1', 'tool-1', {});
        expect(result).toEqual({ output: 'success' });
    });

    it('delegates getAuditTrail correctly', async () => {
        (SystemApiService.getAuditTrail as any).mockResolvedValue([]);

        await TadpoleOSService.getAuditTrail();
        expect(SystemApiService.getAuditTrail).toHaveBeenCalled();
    });
});
