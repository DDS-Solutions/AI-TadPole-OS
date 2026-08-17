/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the high-level Agent Orchestration and Loading logic.** 
 * Verifies the population of the global agent cache, normalization of raw backend payloads, and field-level updates with model-to-provider auto-resolution. 
 * Mocks `agent_api_service` to isolate high-level service logic from network I/O and REST-specific side-effects.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Duplicate agent IDs in the cache after a refresh or failure to hydrate agent metadata during a network partition.
 * - **Telemetry Link**: Search `[agent_service.test]` in tracing logs.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { normalize_agent, persist_agent_update, agent_service, type Raw_Agent } from './agent_service';
import { agent_api_service } from './agent';
import { system_api_service } from './system_api_service';

vi.mock('./agent', () => ({
    agent_api_service: {
        get_agents: vi.fn(),
        update_agent: vi.fn(),
    }
}));

vi.mock('./system_api_service', () => ({
    system_api_service: {
        engine: {
            check_health: vi.fn(),
        }
    }
}));

describe('agent_service', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('normalize_agent', () => {
        it('normalizes a full raw agent correctly', () => {
            const raw_agent: Raw_Agent = {
                id: 'agent-1',
                name: 'Test Agent',
                role: 'Analyst',
                department: 'Research',
                status: 'working',
                currentTask: 'Investigating regression', // camelCase Wire DTO
                tokensUsed: 500,
                tokenUsage: {
                    inputTokens: 300,
                    outputTokens: 200,
                    totalTokens: 500
                },
                skills: ['search', 'read'],
                workflows: ['workflow-1'],
                budgetUsd: 10.0,
                costUsd: 1.5,
                failureCount: 2,
                lastFailureAt: '2023-01-01T00:00:00Z',
                model: 'gpt-4o',
                voiceId: 'alloy',
                voiceEngine: 'openai',
                connectorConfigs: [{ type: 'fs', uri: '/tmp/workspace' }],
                metadata: { stt_engine: 'whisper', team: 'red' },
                currentReasoningTurn: 2,
                reasoningDepth: 4
            } as any;

            const normalized_agent = normalize_agent(raw_agent, '/tmp/workspace');

            expect(normalized_agent.id).toBe('agent-1');
            expect(normalized_agent.name).toBe('Test Agent');
            expect(normalized_agent.status).toBe('active'); // 'working' -> 'active'
            expect(normalized_agent.current_task).toBe('Investigating regression');
            expect(normalized_agent.tokens_used).toBe(500);
            expect(normalized_agent.input_tokens).toBe(300);
            expect(normalized_agent.output_tokens).toBe(200);
            expect(normalized_agent.skills).toEqual(['search', 'read']);
            expect(normalized_agent.workflows).toEqual(['workflow-1']);
            expect(normalized_agent.department).toBe('Research');
            expect(normalized_agent.failure_count).toBe(2);
            expect(normalized_agent.workspace_path).toBe('/tmp/workspace');
            expect(normalized_agent.voice_id).toBe('alloy');
            expect(normalized_agent.voice_engine).toBe('openai');
            expect(normalized_agent.stt_engine).toBe('whisper');
            expect(normalized_agent.connector_configs).toEqual([{ type: 'fs', uri: '/tmp/workspace' }]);
            expect(normalized_agent.metadata).toEqual({ stt_engine: 'whisper', team: 'red' });
            expect(normalized_agent.current_reasoning_turn).toBe(2);
            expect(normalized_agent.reasoning_depth).toBe(4);
        });

        it('prefers backend current_task and only falls back when an active payload omits it', () => {
            const existing_agent = {
                id: 'agent-1',
                name: 'Existing',
                role: 'Analyst',
                department: 'Research',
                status: 'active',
                tokens_used: 0,
                current_task: 'Existing task',
                model: 'gpt-4o',
                category: 'user'
            } as any;

            const normalized_from_backend = normalize_agent({
                id: 'agent-1',
                name: 'Existing',
                status: 'active',
                currentTask: 'Fresh task from backend'
            } as any, undefined, existing_agent);

            const normalized_fallback = normalize_agent({
                id: 'agent-1',
                name: 'Existing',
                status: 'active'
            } as any, undefined, existing_agent);

            expect(normalized_from_backend.current_task).toBe('Fresh task from backend');
            expect(normalized_fallback.current_task).toBe('Existing task');
        });

        it('handles missing fields with defaults', () => {
            const raw_agent: Raw_Agent = { id: '2', name: '' } as any;
            const normalized_agent = normalize_agent(raw_agent);

            expect(normalized_agent.name).toBe('Unnamed Agent');
            expect(normalized_agent.role).toBe('AI Agent');
            expect(normalized_agent.department).toBe('Operations');
            expect(normalized_agent.skills).toEqual([]);
            expect(normalized_agent.tokens_used).toBe(0);
        });

        it('prefers metadata for role and department if primary fields missing', () => {
            const raw_agent: Raw_Agent = { 
                id: '3', 
                name: 'MetaAgent',
                metadata: { role: 'Architect', department: 'Executive' }
            } as any;
            const normalized_agent = normalize_agent(raw_agent);
            expect(normalized_agent.role).toBe('Architect');
            expect(normalized_agent.department).toBe('Executive');
        });

        it('supports legacy snake_case aliases for backend fields', () => {
            const raw_agent = {
                id: '4',
                name: 'LegacyAgent',
                tokens_used: 25,
                model_config: { provider: 'local', modelId: 'llama' },
                workspace_path: '/legacy/workspace',
                current_task: 'Legacy task',
                mcp_tools: '["filesystem","search"]',
                theme_color: '#00ff99',
                budget_usd: 5,
                cost_usd: 1,
                requires_oversight: true,
                model_2: 'claude',
                model_3: 'gemini',
                active_model_slot: 2,
                failure_count: 1,
                last_failure_at: '2026-01-01T00:00:00Z',
                created_at: '2025-01-01T00:00:00Z',
                last_pulse: '2026-01-02T00:00:00Z',
                connector_configs: [{ type: 'fs', uri: '/legacy/workspace' }],
                voice_id: 'alloy',
                voice_engine: 'openai',
                stt_engine: 'whisper',
                input_tokens: 10,
                output_tokens: 15,
                current_reasoning_turn: 3,
                reasoning_depth: 7,
            } as any;

            const normalized_agent = normalize_agent(raw_agent);

            expect(normalized_agent.tokens_used).toBe(25);
            expect(normalized_agent.model_config).toEqual({ provider: 'local', modelId: 'llama' });
            expect(normalized_agent.workspace_path).toBe('/legacy/workspace');
            expect(normalized_agent.current_task).toBe('Legacy task');
            expect(normalized_agent.mcp_tools).toEqual(['filesystem', 'search']);
            expect(normalized_agent.theme_color).toBe('#00ff99');
            expect(normalized_agent.budget_usd).toBe(5);
            expect(normalized_agent.cost_usd).toBe(1);
            expect(normalized_agent.requires_oversight).toBe(true);
            expect(normalized_agent.model_2).toBe('claude');
            expect(normalized_agent.model_3).toBe('gemini');
            expect(normalized_agent.active_model_slot).toBe(2);
            expect(normalized_agent.failure_count).toBe(1);
            expect(normalized_agent.last_failure_at).toBe('2026-01-01T00:00:00Z');
            expect(normalized_agent.created_at).toBe('2025-01-01T00:00:00Z');
            expect(normalized_agent.last_pulse).toBe('2026-01-02T00:00:00Z');
            expect(normalized_agent.connector_configs).toEqual([{ type: 'fs', uri: '/legacy/workspace' }]);
            expect(normalized_agent.voice_id).toBe('alloy');
            expect(normalized_agent.voice_engine).toBe('openai');
            expect(normalized_agent.stt_engine).toBe('whisper');
            expect(normalized_agent.input_tokens).toBe(10);
            expect(normalized_agent.output_tokens).toBe(15);
            expect(normalized_agent.current_reasoning_turn).toBe(3);
            expect(normalized_agent.reasoning_depth).toBe(7);
        });

        it('preserves explicit DTO fields over metadata fallbacks', () => {
            const raw_agent = {
                id: '5',
                name: 'ExplicitAgent',
                role: 'Operator',
                department: 'Operations',
                sttEngine: 'native',
                metadata: {
                    role: 'Architect',
                    department: 'Executive',
                    stt_engine: 'whisper'
                }
            } as any;

            const normalized_agent = normalize_agent(raw_agent);

            expect(normalized_agent.role).toBe('Operator');
            expect(normalized_agent.department).toBe('Operations');
            expect(normalized_agent.stt_engine).toBe('native');
        });
    });

    describe('persist_agent_update', () => {
        it('calls agent_api_service.update_agent with correctly mapped camelCase payload', async () => {
            await persist_agent_update('agent-1', {
                name: 'New Name',
                skills: ['skill-1'],
                budget_usd: 20,
                current_task: 'Handling follow-up'
            });

            expect(agent_api_service.update_agent).toHaveBeenCalledWith('agent-1', {
                name: 'New Name',
                skills: ['skill-1'],
                budget_usd: 20,
                current_task: 'Handling follow-up'
            });
        });

        it('resolves provider automatically for common models if missing', async () => {
            await persist_agent_update('agent-1', {
                model: 'claude-3-sonnet'
            });

            expect(agent_api_service.update_agent).toHaveBeenCalledWith('agent-1', {
                model: 'claude-3-sonnet'
            });
        });

        it('uses model_config provided explicitly', async () => {
            await persist_agent_update('agent-1', {
                model_config: {
                    modelId: 'custom-model',
                    provider: 'ollama',
                    temperature: 0.5,
                    systemPrompt: 'hi',
                    reasoningDepth: 6,
                    actThreshold: 0.82
                }
            });

            expect(agent_api_service.update_agent).toHaveBeenCalledWith('agent-1', {
                model_config: {
                    modelId: 'custom-model',
                    provider: 'ollama',
                    temperature: 0.5,
                    systemPrompt: 'hi',
                    reasoningDepth: 6,
                    actThreshold: 0.82
                }
            });
        });

        it('forwards voice, metadata, and connector updates in camelCase', async () => {
            await persist_agent_update('agent-1', {
                voice_id: 'verse',
                voice_engine: 'openai',
                metadata: { stt_engine: 'groq' },
                connector_configs: [{ type: 'fs', uri: 'D:/TadpoleOS-Dev' }]
            });

            expect(agent_api_service.update_agent).toHaveBeenCalledWith('agent-1', {
                voice_id: 'verse',
                voice_engine: 'openai',
                metadata: { stt_engine: 'groq' },
                connector_configs: [{ type: 'fs', uri: 'D:/TadpoleOS-Dev' }]
            });
        });

        it('triggers load_agents_into_store and logs warning on update failure', async () => {
            const loadSpy = vi.spyOn(agent_service, 'load_agents_into_store').mockResolvedValue(undefined);
            vi.mocked(agent_api_service.update_agent).mockRejectedValue(new Error('Update failed'));

            await persist_agent_update('agent-1', {
                name: 'New Name'
            });

            expect(agent_api_service.update_agent).toHaveBeenCalledWith('agent-1', {
                name: 'New Name'
            });
            expect(loadSpy).toHaveBeenCalled();
        });
    });

    describe('dispose', () => {
        it('cleans up telemetry and tab sync subscriptions', () => {
            const mock_unsubscribe_socket = vi.fn();
            const mock_unsubscribe_tab = vi.fn();

            const telemetry_spy = vi.spyOn(agent_service as any, 'init_telemetry').mockReturnValue(mock_unsubscribe_socket);
            const sync_spy = vi.spyOn(agent_service as any, 'init_tab_sync').mockReturnValue(mock_unsubscribe_tab);

            const cleanup = agent_service.init();

            agent_service.dispose();

            expect(mock_unsubscribe_socket).toHaveBeenCalled();
            expect(mock_unsubscribe_tab).toHaveBeenCalled();

            cleanup();

            telemetry_spy.mockRestore();
            sync_spy.mockRestore();
        });
    });

    describe('fetch_agents_from_api', () => {
        it('fetches agents from api when health check succeeds', async () => {
            vi.mocked(system_api_service.engine.check_health).mockResolvedValue(true);
            
            const raw_agents: Raw_Agent[] = [{ id: 'api-agent', name: 'API Agent' } as any];
            vi.mocked(agent_api_service.get_agents).mockResolvedValue(raw_agents);

            const result = await agent_service.fetch_agents_from_api();

            expect(result).toEqual(raw_agents);
            expect(agent_api_service.get_agents).toHaveBeenCalled();
        });

        it('returns empty array when health check fails', async () => {
            vi.mocked(system_api_service.engine.check_health).mockResolvedValue(false);

            const result = await agent_service.fetch_agents_from_api();

            expect(result).toEqual([]);
            expect(agent_api_service.get_agents).not.toHaveBeenCalled();
        });
    });

    describe('store warning logic', () => {
        it('logs a recoverable warning when store is not set', () => {
            const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
            
            // Set store to null temporarily
            const previousStore = (agent_service as any).agent_store;
            (agent_service as any).agent_store = null;
            
            const store = (agent_service as any)._get_store_or_warn('test_action');
            expect(store).toBeNull();
            expect(warnSpy).toHaveBeenCalledWith(
                expect.stringContaining('[AgentService] Recoverable Setup Warning:')
            );
            
            // Restore store
            (agent_service as any).agent_store = previousStore;
            warnSpy.mockRestore();
        });
    });

    describe('_normalize_patch', () => {
        it('delegates to normalize_agent_dto only for SOCKET source when workspace_path is provided', () => {
            const patch = { name: 'Socket Patch' };
            const existing = { id: 'agent-1', name: 'Existing' } as any;
            
            const resultSocket = (agent_service as any)._normalize_patch(
                patch, 
                'SOCKET', 
                existing, 
                { workspace_path: '/workspaces/test' }
            );
            expect(resultSocket.name).toBe('Socket Patch');
            expect(resultSocket.workspace_path).toBe('/workspaces/test');

            const resultApi = (agent_service as any)._normalize_patch(
                patch, 
                'API', 
                existing, 
                { workspace_path: '/workspaces/test' }
            );
            expect(resultApi).toBe(patch); // Identity check
        });
    });
});

// Metadata: [agent_service_test]
