/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the command processing and routing logic.** 
 * Validates the parsing of chat commands and ensures they are correctly routed to the appropriate agent, cluster, or swarm scope based on prefixes (@, #, etc.).
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Regex parsing failure for complex quoted strings or incorrect scope resolution for overlapping cluster/agent names.
 * - **Telemetry Link**: Search `[command_processor.test]` in tracing logs.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { process_command } from './command_processor';
import { agent_api_service } from '../services/agent_api_service';
import type { Agent } from '../types';

// Mock dependencies
vi.mock('../services/event_bus', () => ({
    event_bus: {
        emit: vi.fn(),
        emit_log: vi.fn(),
    }
}));

vi.mock('../services/agent_api_service', () => ({
    agent_api_service: {
        send_command: vi.fn().mockResolvedValue({ success: true }),
        pause_agent: vi.fn().mockResolvedValue(true),
        resume_agent: vi.fn().mockResolvedValue(true),
    }
}));

vi.mock('../services/system_api_service', () => ({
    system_api_service: {
        execute_local_cmd: vi.fn().mockResolvedValue({ success: true }),
        get_engine_status: vi.fn().mockResolvedValue({ features: [] }),
    }
}));

vi.mock('../stores/workspace_store', () => ({
    use_workspace_store: {
        getState: vi.fn().mockReturnValue({
            clusters: [{ id: 'c1', name: 'Engineering', alpha_id: '2', department: 'Eng' }]
        })
    }
}));

vi.mock('../stores/agent_store', () => ({
    use_agent_store: {
        getState: vi.fn().mockReturnValue({
            agents: [
                { id: '1', name: 'CEO', status: 'idle', tokens_used: 0, model: 'gpt-4', model_config: {} },
                { id: '2', name: 'Tadpole_Alpha', status: 'idle', tokens_used: 0, model: 'gpt-4', model_config: {} }
            ]
        })
    }
}));

vi.mock('../stores/sovereign_store', () => {
    const mock_store = {
        getState: vi.fn().mockReturnValue({
            add_message: vi.fn(),
        }),
        setState: vi.fn(),
        subscribe: vi.fn(),
    };
    const use_store = vi.fn().mockImplementation(() => mock_store.getState());
    return {
        use_sovereign_store: Object.assign(use_store, mock_store)
    };
});

describe('process_command', () => {
    const mock_agents: Agent[] = [
        { id: '1', name: 'CEO', status: 'idle', theme_color: '#000', voice_id: 'v1', tokens_used: 0, model: 'gpt-4', model_config: {} },
        { id: '2', name: 'Tadpole_Alpha', status: 'idle', theme_color: '#fff', voice_id: 'v2', tokens_used: 0, model: 'gpt-4', model_config: {} }
    ];

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should split commands correctly and preserve quoted strings', async () => {
        const text = '/send CEO "hello world"';
        await process_command(text, mock_agents);
        
        expect(agent_api_service.send_command).toHaveBeenCalledWith(
            '1', 
            'hello world',
            expect.anything(),
            expect.anything(),
            undefined,
            undefined,
            undefined,
            undefined,
            false
        );
    });

    it('should route prefix-less commands to target_node in agent scope', async () => {
        const text = 'status check';
        await process_command(text, mock_agents, false, 'agent', 'Tadpole_Alpha');
        
        expect(agent_api_service.send_command).toHaveBeenCalledWith(
            '2', 
            'status check',
            expect.anything(),
            expect.anything(),
            undefined,
            undefined,
            undefined,
            undefined,
            false
        );
    });

    it('should allow @mention override in any scope', async () => {
        const text = '@CEO wake up';
        await process_command(text, mock_agents, false, 'agent', 'Tadpole_Alpha');
        
        expect(agent_api_service.send_command).toHaveBeenCalledWith(
            '1', 
            'wake up',
            expect.anything(),
            expect.anything(),
            undefined,
            undefined,
            undefined,
            undefined,
            false
        );
    });

    it('should handle cluster targeting via # prefix by routing to the Alpha node', async () => {
        const text = '#Engineering sync';
        await process_command(text, mock_agents, false);
        
        expect(agent_api_service.send_command).toHaveBeenCalledWith(
            '2', 
            'sync',
            expect.anything(),
            expect.anything(),
            'c1', 
            'Eng', 
            undefined,
            undefined,
            false
        );
    });
});
