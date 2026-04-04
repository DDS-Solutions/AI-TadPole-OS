/**
 * @file agent_api_service.test.ts
 * @description Unit tests for the Agent API service layer.
 * @module Services/agent_api_service
 * @testedBehavior
 * - Agent Retrieval: Direct array vs HATEOAS envelope unwrapping.
 * - Agent Mutation: Create/Update body mapping and model config transforms.
 * - Remote Orchestration: command sending with API key injection and model limit handling.
 * - Persistent Memory: CRUD operations for agent-specific memory rows.
 * @aiContext
 * - Mocks base_api_service (apiRequest) to prevent network side-effects.
 * - Mocks providerStore to simulate model limits and vault states.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { agent_api_service } from './agent_api_service';
import { apiRequest } from './base_api_service';
import { use_provider_store } from '../stores/provider_store';
import { use_vault_store } from '../stores/vault_store';
import { use_model_store } from '../stores/model_store';
import { event_bus } from './event_bus';
import { PROVIDERS } from '../constants';

vi.mock('./base_api_service', () => ({
    apiRequest: vi.fn(),
}));

vi.mock('../stores/provider_store', () => ({
    use_provider_store: {
        getState: vi.fn(),
    },
}));

vi.mock('../stores/vault_store', () => ({
    use_vault_store: {
        getState: vi.fn(),
    },
}));

vi.mock('../stores/model_store', () => ({
    use_model_store: {
        getState: vi.fn(),
    },
}));

vi.mock('./event_bus', () => ({
    event_bus: {
        emit: vi.fn(),
    },
}));

describe('agent_api_service', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('get_agents', () => {
        it('should return agents from a direct array response', async () => {
            const mockAgents = [{ id: '1', name: 'Agent 1' }];
            vi.mocked(apiRequest).mockResolvedValueOnce(mockAgents);

            const result = await agent_api_service.get_agents();
            expect(result).toEqual(mockAgents);
        });

        it('should return agents from a HATEOAS data envelope', async () => {
            const mockAgents = [{ id: '2', name: 'Agent 2' }];
            vi.mocked(apiRequest).mockResolvedValueOnce({ data: mockAgents });

            const result = await agent_api_service.get_agents();
            expect(result).toEqual(mockAgents);
        });

        it('should return an empty array for undefined envelope data', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ data: null });
            const result = await agent_api_service.get_agents();
            expect(result).toEqual([]);
        });

        it('should return an empty array if response is null/invalid', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce(null);
            const result = await agent_api_service.get_agents();
            expect(result).toEqual([]);
        });
    });

    describe('update_agent', () => {
        it('should send a PUT request with the correct body', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            const config = {
                name: 'Updated Name',
                modelConfig2: { modelId: 'model-2' } as any,
                modelConfig3: { modelId: 'model-3' } as any,
            };

            const result = await agent_api_service.update_agent('agent-1', config);
            expect(result).toBe(true);
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1', expect.objectContaining({
                method: 'PUT',
                body: expect.stringContaining('"name":"Updated Name"')
            }));
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1', expect.objectContaining({
                body: expect.stringContaining('"model2":"model-2"')
            }));
        });
    });

    describe('create_agent', () => {
        it('should send a POST request with the new agent mapping', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            const newAgent = {
                id: 'new-agent',
                name: 'Test Agent',
                role: 'Tester',
                department: 'QA',
                model: 'model-1',
                budget_usd: 10,
                theme_color: '#fff',
            } as any;

            const result = await agent_api_service.create_agent(newAgent);
            expect(result).toBe(true);
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents', expect.objectContaining({
                method: 'POST',
                body: expect.stringContaining('"name":"Test Agent"')
            }));
        });
    });

    describe('pause_agent and resume_agent', () => {
        it('should send POST to pause', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await agent_api_service.pause_agent('agent-1');
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/pause', { method: 'POST' });
        });

        it('should send POST to resume', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await agent_api_service.resume_agent('agent-1');
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/resume', { method: 'POST' });
        });
    });

    describe('send_command', () => {
        const mockGetApiKey = vi.fn();
        const mockVaultState = {
            getApiKey: mockGetApiKey,
            isLocked: false
        };
        const mockModelState = {
            models: [{ name: 'test-model', rpm: 10, tpm: 1000, rpd: 100, tpd: 10000 }],
        };
        const mockProviderState = {
            baseUrls: { 'openai': 'https://api.openai.com/v1' },
        };

        beforeEach(() => {
            vi.mocked(use_vault_store.getState).mockReturnValue(mockVaultState as any);
            vi.mocked(use_model_store.getState).mockReturnValue(mockModelState as any);
            vi.mocked(use_provider_store.getState).mockReturnValue(mockProviderState as any);
        });

        it('should send command with API key and model limits if provider has a key', async () => {
            mockGetApiKey.mockResolvedValueOnce('secret-key');
            vi.mocked(apiRequest).mockResolvedValueOnce({});

            await agent_api_service.send_command('agent-1', 'Hello', 'test-model', 'openai');

            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                method: 'POST',
                body: expect.stringContaining('"apiKey":"secret-key"')
            }));
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                body: expect.stringContaining('"rpm":10')
            }));
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                body: expect.stringContaining('"baseUrl":"https://api.openai.com/v1"')
            }));
        });

        it('should emit a security info event if no key is present for a remote provider', async () => {
            mockGetApiKey.mockResolvedValueOnce(null);
            vi.mocked(apiRequest).mockResolvedValueOnce({});

            await agent_api_service.send_command('agent-1', 'Hello', 'test-model', 'openai');

            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                text: expect.stringContaining('🔒 Neural Security: No Key for OPENAI for AGENT-1.'),
                severity: 'warning'
            }));
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                method: 'POST'
            }));
        });

        it('should ignore missing key for local providers and not emit event', async () => {
            mockGetApiKey.mockResolvedValueOnce(null);
            vi.mocked(apiRequest).mockResolvedValueOnce({});

            await agent_api_service.send_command('agent-1', 'Hello', 'test-model', PROVIDERS.OLLAMA);

            expect(event_bus.emit_log).not.toHaveBeenCalled();
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                method: 'POST'
            }));
        });

        it('should include X-Request-Id header if requestId is provided', async () => {
            mockGetApiKey.mockResolvedValueOnce('secret-key');
            vi.mocked(apiRequest).mockResolvedValueOnce({});

            await agent_api_service.send_command('agent-1', 'Hello', 'test-model', 'openai', undefined, undefined, undefined, undefined, undefined, undefined, 'req-123');

            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                headers: { 'X-Request-Id': 'req-123' }
            }));
        });
    });

    describe('Memory operations', () => {
        it('get_agent_memory', async () => {
            const mockRes = { status: 'ok', entries: [] };
            vi.mocked(apiRequest).mockResolvedValueOnce(mockRes);
            const res = await agent_api_service.get_agent_memory('agent-1');
            expect(res).toEqual(mockRes);
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/memories', { method: 'GET' });
        });

        it('delete_agent_memory', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ status: 'ok' });
            await agent_api_service.delete_agent_memory('agent-1', 'row-1');
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/memories/row-1', { method: 'DELETE' });
        });

        it('save_agent_memory', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ status: 'ok', id: 'new-row' });
            await agent_api_service.save_agent_memory('agent-1', 'new memory text');
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/memories', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({ text: 'new memory text' })
            }));
        });
    });
});

