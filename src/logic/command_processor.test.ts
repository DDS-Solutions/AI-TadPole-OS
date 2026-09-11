/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / command_processor.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { process_command, initialize_command_processor, reset_command_processor } from './command_processor';
import { agent_api_service } from '../services/agent';
import { agent_service } from '../services/agent_service';
import { system_api_service } from '../services/system_api_service';
import { event_bus } from '../services/event_bus';
import { use_agent_store } from '../stores/agent_store';
import { command_registry } from './commands/registry';
import type { Agent } from '../types';

// Mock dependencies
vi.mock('../services/event_bus', () => ({
    event_bus: {
        emit: vi.fn(),
        emit_log: vi.fn(),
        subscribe_traces: vi.fn(() => () => {}),
        clear_history: vi.fn(),
    }
}));

vi.mock('../services/agent', () => ({
    agent_api_service: {
        send_command: vi.fn().mockResolvedValue({ success: true }),
        pause_agent: vi.fn().mockResolvedValue(true),
        resume_agent: vi.fn().mockResolvedValue(true),
    }
}));

vi.mock('../services/agent_service', () => ({
    agent_service: {
        pause_agent: vi.fn().mockResolvedValue(true),
        resume_agent: vi.fn().mockResolvedValue(true),
        delete_agent: vi.fn().mockResolvedValue(true),
        update_agent: vi.fn().mockResolvedValue(true),
    }
}));

vi.mock('../services/system_api_service', () => ({
    system_api_service: {
        engine: {
            execute_local_cmd: vi.fn().mockResolvedValue({ success: true }),
            get_engine_status: vi.fn().mockResolvedValue({ features: [] }),
            deploy_engine: vi.fn().mockResolvedValue({ status: 'ok', output: 'Deployed successfully' }),
        }
    }
}));

vi.mock('../stores/workspace_store', () => ({
    use_workspace_store: {
        getState: vi.fn().mockReturnValue({
            clusters: [{ id: 'c1', name: 'Engineering', alpha_id: '2', department: 'Eng', theme: 'cyan', collaborators: ['1', '2'] }],
            generate_proposal: vi.fn().mockResolvedValue({ reasoning: 'Optimize token usage' })
        })
    }
}));

vi.mock('../stores/agent_store', () => ({
    use_agent_store: {
        getState: vi.fn().mockReturnValue({
            agents: [
                { id: '1', name: 'CEO', status: 'idle', tokens_used: 0, model: 'gpt-4', model_config: { provider: 'openai', modelId: 'gpt-4' } },
                { id: '2', name: 'Tadpole_Alpha', status: 'idle', tokens_used: 0, model: 'gpt-4', model_config: { provider: 'openai', modelId: 'gpt-4' } }
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

const default_model_config = { provider: 'openai', modelId: 'gpt-4' };

describe('process_command', () => {
    const mock_agents: Agent[] = [
        { id: '1', name: 'CEO', status: 'idle', theme_color: '#000', voice_id: 'v1', tokens_used: 0, model: 'gpt-4', model_config: default_model_config },
        { id: '2', name: 'Tadpole_Alpha', status: 'idle', theme_color: '#fff', voice_id: 'v2', tokens_used: 0, model: 'gpt-4', model_config: default_model_config }
    ];

    beforeEach(() => {
        vi.useFakeTimers();
        vi.clearAllMocks();
        reset_command_processor();
        initialize_command_processor();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it('should split commands correctly and preserve quoted strings', async () => {
        const text = '/send CEO "hello world"';
        await process_command(text, mock_agents);
        vi.runAllTimers();
        
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
        vi.runAllTimers();
        
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
        vi.runAllTimers();
        
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
        vi.runAllTimers();
        
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

    it('should resolve agent using ranked matching prioritizing exact match', async () => {
        const overlapping_agents: Agent[] = [
            { id: '1', name: 'Agent-Alpha-Prime', status: 'idle', theme_color: '#000', voice_id: 'v1', tokens_used: 0, model: 'gpt-4', model_config: default_model_config },
            { id: '2', name: 'Agent-Alpha', status: 'idle', theme_color: '#fff', voice_id: 'v2', tokens_used: 0, model: 'gpt-4', model_config: default_model_config }
        ];
        
        const text = '@Agent-Alpha hello';
        await process_command(text, overlapping_agents);
        vi.runAllTimers();

        // Should resolve to Agent-Alpha (exact match) rather than Agent-Alpha-Prime (first-in-list substring match)
        expect(agent_api_service.send_command).toHaveBeenCalledWith(
            '2',
            'hello',
            expect.anything(),
            expect.anything(),
            undefined,
            undefined,
            undefined,
            undefined,
            false
        );
    });

    it('should enforce input length validation guard', async () => {
        const long_text = 'a'.repeat(5000);
        const result = await process_command(long_text, mock_agents);
        
        expect(result).toEqual({ should_clear_logs: false });
        expect(agent_api_service.send_command).not.toHaveBeenCalled();
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'error',
            text: expect.stringContaining('exceeds maximum limit')
        }));
    });

    it('should sanitize control characters from the command text', async () => {
        const text = '@CEO \u0000\u0007hello\r\n';
        await process_command(text, mock_agents);
        vi.runAllTimers();

        expect(agent_api_service.send_command).toHaveBeenCalledWith(
            '1',
            'hello',
            expect.anything(),
            expect.anything(),
            undefined,
            undefined,
            undefined,
            undefined,
            false
        );
    });

    it('should handle unhandled rejection in command handlers gracefully via error boundaries', async () => {
        vi.spyOn(command_registry, 'execute').mockRejectedValueOnce(new Error('Registry crashed'));

        const text = '/clear';
        const result = await process_command(text, mock_agents);

        expect(result).toEqual({ should_clear_logs: false });
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'error',
            text: expect.stringContaining('Command execution failed: Registry crashed')
        }));
    });

    it('should time out if command execution exceeds the threshold', async () => {
        vi.spyOn(command_registry, 'execute').mockImplementationOnce(() => {
            return new Promise(() => {
                // Never resolve to simulate a hang
            });
        });

        const text = '/clear';
        const promise = process_command(text, mock_agents);
        
        // Advance timers by 6 seconds to trigger the 5-second timeout
        await vi.advanceTimersByTimeAsync(6000);
        
        const result = await promise;
        expect(result).toEqual({ should_clear_logs: false });
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'error',
            text: expect.stringContaining('Command execution failed: Command execution timed out')
        }));
    });

    it('should sanitize C1 control characters in input', async () => {
        const text = '@CEO \u0080\u009Fhello';
        await process_command(text, mock_agents);
        vi.runAllTimers();

        expect(agent_api_service.send_command).toHaveBeenCalledWith(
            '1',
            'hello',
            expect.anything(),
            expect.anything(),
            undefined,
            undefined,
            undefined,
            undefined,
            false
        );
    });

    it('should fail if command processor is not explicitly initialized', async () => {
        reset_command_processor(); // Clear registration and flag
        
        const text = '/clear';
        const result = await process_command(text, mock_agents);

        expect(result).toEqual({ should_clear_logs: false });
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'error',
            text: expect.stringContaining('Unknown command: /clear')
        }));
    });

    it('should route next chat turn to newly activated model when active_model_slot changes in store', async () => {
        const { use_agent_store } = await import('../stores/agent_store');
        
        const test_agent: Agent = {
            id: 'agent-elon',
            name: 'Elon',
            status: 'idle',
            model: 'gemini-1.5-pro',
            model_2: 'gpt-4o-2024-08-06',
            model_3: 'claude-3-5-sonnet-20241022',
            active_model_slot: 1,
            model_config2: { modelId: 'gpt-4o-2024-08-06', provider: 'openai' },
            model_config3: { modelId: 'claude-3-5-sonnet-20241022', provider: 'anthropic' }
        } as any;

        (use_agent_store.getState as any).mockReturnValue({
            agents: [test_agent]
        });

        // 1. Initial chat turn on Slot 1
        await process_command('@Elon Hello from slot 1', [test_agent]);
        vi.runAllTimers();

        expect(agent_api_service.send_command).toHaveBeenLastCalledWith(
            'agent-elon',
            'Hello from slot 1',
            'gemini-1.5-pro',
            'google',
            undefined,
            undefined,
            undefined,
            undefined,
            false
        );

        // 2. User activates Slot 2 (e.g. clicking green dot on card)
        test_agent.active_model_slot = 2;

        // 3. Next chat turn MUST route to Slot 2 model (gpt-4o-2024-08-06)
        await process_command('@Elon Hello from slot 2', [test_agent]);
        vi.runAllTimers();

        expect(agent_api_service.send_command).toHaveBeenLastCalledWith(
            'agent-elon',
            'Hello from slot 2',
            'gpt-4o-2024-08-06',
            'openai',
            undefined,
            undefined,
            undefined,
            undefined,
            false
        );
    });

    it('should preserve empty quoted strings as distinct arguments', async () => {
        const text = '/send CEO ""';
        await process_command(text, mock_agents);
        vi.runAllTimers();

        // Empty message should trigger usage warning
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'error',
            text: expect.stringContaining('Usage: /send')
        }));
    });

    it('should preserve words with contractions and apostrophes as intact tokens', async () => {
        const text = "@CEO don't touch it's broken";
        await process_command(text, mock_agents);
        vi.runAllTimers();

        expect(agent_api_service.send_command).toHaveBeenCalledWith(
            '1',
            "don't touch it's broken",
            expect.anything(),
            expect.anything(),
            undefined,
            undefined,
            undefined,
            undefined,
            false
        );
    });

    it('should normalize line breaks to spaces in multiline input', async () => {
        const text = '@CEO line1\r\nline2\tline3';
        await process_command(text, mock_agents);
        vi.runAllTimers();

        expect(agent_api_service.send_command).toHaveBeenCalledWith(
            '1',
            'line1 line2 line3',
            expect.anything(),
            expect.anything(),
            undefined,
            undefined,
            undefined,
            undefined,
            false
        );
    });

    it('should print available commands and targeting syntax on /help', async () => {
        const result = await process_command('/help', mock_agents);
        expect(result).toEqual({ should_clear_logs: false });
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'info',
            text: expect.stringContaining('Available Slash Commands:')
        }));
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            text: expect.stringContaining('Targeting Syntax:')
        }));
    });

    it('should report status including active, idle, offline, and degraded agents on /status', async () => {
        const diverse_agents: Agent[] = [
            { id: '1', name: 'A1', status: 'active', tokens_used: 1000, model: 'gpt-4' } as Agent,
            { id: '2', name: 'A2', status: 'speaking', tokens_used: 500, model: 'gpt-4' } as Agent,
            { id: '3', name: 'A3', status: 'idle', tokens_used: 0, model: 'gpt-4' } as Agent,
            { id: '4', name: 'A4', status: 'offline', tokens_used: 0, model: 'gpt-4' } as Agent,
            { id: '5', name: 'A5', status: 'suspended', tokens_used: 0, model: 'gpt-4' } as Agent,
            { id: '6', name: 'A6', status: 'failed', tokens_used: 0, model: 'gpt-4' } as Agent,
        ];
        const result = await process_command('/status', diverse_agents);
        expect(result).toEqual({ should_clear_logs: false });
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'success',
            text: expect.stringContaining('2 active · 1 idle · 1 offline · 2 degraded/suspended')
        }));
    });

    it('should require confirm argument for /deploy', async () => {
        const result = await process_command('/deploy', mock_agents);
        expect(result).toEqual({ should_clear_logs: false });
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'warning',
            text: expect.stringContaining('Type "/deploy confirm" to proceed')
        }));
        expect(system_api_service.engine.deploy_engine).not.toHaveBeenCalled();
    });

    it('should trigger deploy_engine on /deploy confirm', async () => {
        const result = await process_command('/deploy confirm', mock_agents);
        expect(result).toEqual({ should_clear_logs: false });
        expect(system_api_service.engine.deploy_engine).toHaveBeenCalled();
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'success',
            text: expect.stringContaining('Deployment successful')
        }));
    });

    it('should display agent configuration on /config', async () => {
        const result = await process_command('/config CEO', mock_agents);
        expect(result).toEqual({ should_clear_logs: false });
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'info',
            text: expect.stringContaining('Config for CEO:')
        }));
    });

    it('should pause and resume agents via agent_service', async () => {
        await process_command('/pause CEO', mock_agents);
        expect(agent_service.pause_agent).toHaveBeenCalledWith('1');

        await process_command('/resume CEO', mock_agents);
        expect(agent_service.resume_agent).toHaveBeenCalledWith('1');
    });

    it('should strictly validate slot input on /switch', async () => {
        await process_command('/switch CEO 2', mock_agents);
        expect(agent_service.update_agent).toHaveBeenCalledWith('1', { active_model_slot: 2 });

        await process_command('/switch CEO 4', mock_agents);
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'error',
            text: 'Invalid slot. Use 1, 2, or 3.'
        }));

        await process_command('/switch CEO 1abc', mock_agents);
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'error',
            text: 'Invalid slot. Use 1, 2, or 3.'
        }));
    });

    it('should require confirm before terminating agent on /kill', async () => {
        await process_command('/kill CEO', mock_agents);
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'warning',
            text: expect.stringContaining('Type "/kill CEO confirm" to proceed')
        }));
        expect(agent_service.delete_agent).not.toHaveBeenCalled();

        await process_command('/kill CEO confirm', mock_agents);
        expect(agent_service.delete_agent).toHaveBeenCalledWith('1');
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'warning',
            text: expect.stringContaining('Agent CEO terminated and removed from swarm')
        }));
    });

    it('should reject empty directive message for @agent and #cluster', async () => {
        await process_command('@CEO', mock_agents);
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'error',
            text: expect.stringContaining('Missing directive for agent @CEO')
        }));

        await process_command('#Engineering', mock_agents);
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'error',
            text: expect.stringContaining('Missing directive for cluster #Engineering')
        }));
    });

    it('should emit error if cluster alpha node is offline or missing', async () => {
        const text = '#Engineering hello';
        // Pass agent list without the alpha agent (id: 2)
        const partial_agents = [mock_agents[0]];
        (use_agent_store.getState as any).mockReturnValueOnce({ agents: partial_agents });

        await process_command(text, partial_agents);
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'error',
            text: expect.stringContaining('Alpha node for cluster "Engineering"')
        }));
    });

    it('should resolve agent by ID case-insensitively', async () => {
        const id_agents: Agent[] = [
            { id: 'agent-custom-id', name: 'Custom Bot', status: 'idle', model: 'gpt-4', model_config: default_model_config } as Agent
        ];
        await process_command('@AGENT-CUSTOM-ID hello', id_agents);
        vi.runAllTimers();

        expect(agent_api_service.send_command).toHaveBeenCalledWith(
            'agent-custom-id',
            'hello',
            expect.anything(),
            expect.anything(),
            undefined,
            undefined,
            undefined,
            undefined,
            false
        );
    });

    it('should report cluster inventory and trigger proposals on /swarm commands', async () => {
        await process_command('/swarm status', mock_agents);
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'info',
            text: expect.stringContaining('Mission Cluster Inventory:')
        }));

        await process_command('/swarm optimize', mock_agents);
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'info',
            text: expect.stringContaining('[Optimization Proposal for Engineering]: Optimize token usage')
        }));
    });
});
