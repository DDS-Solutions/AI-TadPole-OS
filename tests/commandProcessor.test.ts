/**
 * @docs ARCHITECTURE:Quality:Verification
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[commandProcessor_test]` in observability traces.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { process_command } from '../src/logic/command_processor';
import { event_bus } from '../src/services/event_bus';
import type { Agent } from '../src/types';

// Mock API Services
vi.mock('../src/services/agent_api_service', () => ({
    agent_api_service: {
        pause_agent: vi.fn().mockResolvedValue(true),
        resume_agent: vi.fn().mockResolvedValue(true),
        send_command: vi.fn().mockResolvedValue(true),
        update_agent: vi.fn().mockResolvedValue(true),
    }
}));

vi.mock('../src/services/system_api_service', () => ({
    system_api_service: {
        deploy_engine: vi.fn().mockResolvedValue({ output: 'Mock success' }),
    }
}));

// Mock Zustand store
vi.mock('../src/stores/workspace_store', () => ({
    use_workspace_store: {
        getState: vi.fn().mockReturnValue({
            clusters: [
                { id: 'c1', name: 'Alpha Cluster', theme: 'cyan', alpha_id: 'a1', collaborators: ['a1', 'a2'], objective: 'Test Objective' }
            ],
            active_proposals: {},
            generate_proposal: vi.fn(),
        })
    }
}));

describe('command_processor', () => {
    const mock_agents: Agent[] = [
        { id: 'a1', name: 'Tadpole', status: 'idle', tokens_used: 1000, role: 'Coordinator' } as Agent,
        { id: 'a2', name: 'Optimizer', status: 'active', tokens_used: 500, role: 'Specialist' } as Agent,
    ];

    beforeEach(() => {
        vi.clearAllMocks();
        event_bus.destroy();
    });

    it('processes /help command', async () => {
        const result = await process_command('/help', mock_agents);
        expect(result.should_clear_logs).toBe(false);
        const history = event_bus.get_history();
        expect(history[0].text).toContain('Available Commands');
    });

    it('processes /clear command', async () => {
        event_bus.emit_log({ source: 'System', text: 'test', severity: 'info' });
        const result = await process_command('/clear', mock_agents);
        expect(result.should_clear_logs).toBe(true);
        expect(event_bus.get_history().length).toBe(0);
    });

    it('processes /status command', async () => {
        await process_command('/status', mock_agents);
        const history = event_bus.get_history();
        expect(history[0].text).toContain('Swarm Status');
        expect(history[0].text).toContain('1 active');
        expect(history[0].text).toContain('1.5k');
    });

    it('processes /pause command for valid agent', async () => {
        await process_command('/pause Tadpole', mock_agents);
        const { agent_api_service } = await import('../src/services/agent_api_service');
        expect(agent_api_service.pause_agent).toHaveBeenCalledWith('a1');
        const history = event_bus.get_history();
        expect(history[0].text).toContain('Tadpole paused');
    });

    it('processes /swarm status command', async () => {
        await process_command('/swarm status', mock_agents);
        const history = event_bus.get_history();
        expect(history[0].text).toContain('Mission Cluster Inventory');
        expect(history[0].text).toContain('Alpha Cluster');
        expect(history[0].text).toContain('Test Objective');
    });

    it('reports error for unknown command', async () => {
        await process_command('/invalid', mock_agents);
        const history = event_bus.get_history();
        expect(history[0].text).toContain('Unknown command');
        expect(history[0].severity).toBe('error');
    });

    it('processes /send command with message', async () => {
        await process_command('/send Tadpole hello world', mock_agents);
        const { agent_api_service } = await import('../src/services/agent_api_service');
        // Note: active_model_slot logic in send_command helper
        expect(agent_api_service.send_command).toHaveBeenCalledWith('a1', 'hello world', expect.anything(), expect.anything(), undefined, undefined, undefined, undefined, false);
        const history = event_bus.get_history();
        expect(history[0].text).toBe('→ Tadpole: hello world');
    });

    it('resolves agent by ID for /config command', async () => {
        await process_command('/config a1', mock_agents);
        const history = event_bus.get_history();
        expect(history[0].text).toContain('Config for Tadpole');
    });

    it('resolves agent by partial name match', async () => {
        await process_command('/pause Optim', mock_agents);
        const { agent_api_service } = await import('../src/services/agent_api_service');
        expect(agent_api_service.pause_agent).toHaveBeenCalledWith('a2');
    });

    it('emits error for unknown agent', async () => {
        await process_command('/pause NonExistent', mock_agents);
        const history = event_bus.get_history();
        expect(history[0].severity).toBe('error');
        expect(history[0].text).toContain('not found');
    });

    it('emits error when agent name is missing', async () => {
        await process_command('/pause', mock_agents);
        const history = event_bus.get_history();
        expect(history[0].severity).toBe('error');
        expect(history[0].text).toContain('Missing agent name');
    });
});

// Metadata: [commandProcessor_test]
