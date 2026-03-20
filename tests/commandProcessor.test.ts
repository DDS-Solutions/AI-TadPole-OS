import { describe, it, expect, beforeEach, vi } from 'vitest';
import { processCommand } from '../src/logic/commandProcessor';
import { EventBus } from '../src/services/eventBus';
import type { Agent } from '../src/types';

// Mock API Services
vi.mock('../src/services/AgentApiService', () => ({
    AgentApiService: {
        pauseAgent: vi.fn().mockResolvedValue(true),
        resumeAgent: vi.fn().mockResolvedValue(true),
        sendCommand: vi.fn().mockResolvedValue(true),
        updateAgent: vi.fn().mockResolvedValue(true),
    }
}));

vi.mock('../src/services/SystemApiService', () => ({
    SystemApiService: {
        deployEngine: vi.fn().mockResolvedValue({ output: 'Mock success' }),
    }
}));

// Mock Zustand store
vi.mock('../src/stores/workspaceStore', () => ({
    useWorkspaceStore: {
        getState: vi.fn().mockReturnValue({
            clusters: [
                { id: 'c1', name: 'Alpha Cluster', theme: 'cyan', alphaId: 'a1', collaborators: ['a1', 'a2'], objective: 'Test Objective' }
            ],
            activeProposals: {},
            generateProposal: vi.fn(),
        })
    }
}));

describe('commandProcessor', () => {
    const mockAgents: Agent[] = [
        { id: 'a1', name: 'Tadpole', status: 'idle', tokensUsed: 1000, role: 'Coordinator' } as Agent,
        { id: 'a2', name: 'Optimizer', status: 'active', tokensUsed: 500, role: 'Specialist' } as Agent,
    ];

    beforeEach(() => {
        vi.clearAllMocks();
        EventBus.destroy();
    });

    it('processes /help command', async () => {
        const result = await processCommand('/help', mockAgents);
        expect(result.shouldClearLogs).toBe(false);
        const history = EventBus.getHistory();
        expect(history[0].text).toContain('Available Commands');
    });

    it('processes /clear command', async () => {
        EventBus.emit({ source: 'System', text: 'test', severity: 'info' });
        const result = await processCommand('/clear', mockAgents);
        expect(result.shouldClearLogs).toBe(true);
        expect(EventBus.getHistory().length).toBe(0);
    });

    it('processes /status command', async () => {
        await processCommand('/status', mockAgents);
        const history = EventBus.getHistory();
        expect(history[0].text).toContain('Swarm Status');
        expect(history[0].text).toContain('1 active');
        expect(history[0].text).toContain('1.5k');
    });

    it('processes /pause command for valid agent', async () => {
        await processCommand('/pause Tadpole', mockAgents);
        const { AgentApiService } = await import('../src/services/AgentApiService');
        expect(AgentApiService.pauseAgent).toHaveBeenCalledWith('a1');
        const history = EventBus.getHistory();
        expect(history[0].text).toContain('Tadpole paused');
    });

    it('processes /swarm status command', async () => {
        await processCommand('/swarm status', mockAgents);
        const history = EventBus.getHistory();
        expect(history[0].text).toContain('Mission Cluster Inventory');
        expect(history[0].text).toContain('Alpha Cluster');
        expect(history[0].text).toContain('Test Objective');
    });

    it('reports error for unknown command', async () => {
        await processCommand('/invalid', mockAgents);
        const history = EventBus.getHistory();
        expect(history[0].text).toContain('Unknown command');
        expect(history[0].severity).toBe('error');
    });

    it('processes /send command with message', async () => {
        await processCommand('/send Tadpole hello world', mockAgents);
        const { AgentApiService } = await import('../src/services/AgentApiService');
        expect(AgentApiService.sendCommand).toHaveBeenCalledWith('a1', 'hello world', 'gemini-1.5-flash', 'google', undefined, undefined, undefined, undefined, undefined);
        const history = EventBus.getHistory();
        expect(history[0].text).toBe('→ Tadpole: hello world');
    });

    it('resolves agent by ID for /config command', async () => {
        await processCommand('/config a1', mockAgents);
        const history = EventBus.getHistory();
        expect(history[0].text).toContain('Config for Tadpole');
    });

    it('resolves agent by partial name match', async () => {
        await processCommand('/pause Optim', mockAgents);
        const { AgentApiService } = await import('../src/services/AgentApiService');
        expect(AgentApiService.pauseAgent).toHaveBeenCalledWith('a2');
    });

    it('emits error for unknown agent', async () => {
        await processCommand('/pause NonExistent', mockAgents);
        const history = EventBus.getHistory();
        expect(history[0].severity).toBe('error');
        expect(history[0].text).toContain('not found');
    });

    it('emits error when agent name is missing', async () => {
        await processCommand('/pause', mockAgents);
        const history = EventBus.getHistory();
        expect(history[0].severity).toBe('error');
        expect(history[0].text).toContain('Missing agent name');
    });
});


