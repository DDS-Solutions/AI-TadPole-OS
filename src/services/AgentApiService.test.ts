/**
 * @file AgentApiService.test.ts
 * @description Unit tests for the Agent API service layer.
 * @module Services/AgentApiService
 * @testedBehavior
 * - Agent Retrieval: Direct array vs HATEOAS envelope unwrapping.
 * - Agent Mutation: Create/Update body mapping and model config transforms.
 * - Remote Orchestration: command sending with API key injection and model limit handling.
 * - Persistent Memory: CRUD operations for agent-specific memory rows.
 * @aiContext
 * - Mocks BaseApiService (apiRequest) to prevent network side-effects.
 * - Mocks providerStore to simulate model limits and vault states.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AgentApiService } from './AgentApiService';
import { apiRequest } from './BaseApiService';
import { useProviderStore } from '../stores/providerStore';
import { useVaultStore } from '../stores/vaultStore';
import { useModelStore } from '../stores/modelStore';
import { EventBus } from './eventBus';
import { PROVIDERS } from '../constants';

vi.mock('./BaseApiService', () => ({
    apiRequest: vi.fn(),
}));

vi.mock('../stores/providerStore', () => ({
    useProviderStore: {
        getState: vi.fn(),
    },
}));

vi.mock('../stores/vaultStore', () => ({
    useVaultStore: {
        getState: vi.fn(),
    },
}));

vi.mock('../stores/modelStore', () => ({
    useModelStore: {
        getState: vi.fn(),
    },
}));

vi.mock('./eventBus', () => ({
    EventBus: {
        emit: vi.fn(),
    },
}));

describe('AgentApiService', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('getAgents', () => {
        it('should return agents from a direct array response', async () => {
            const mockAgents = [{ id: '1', name: 'Agent 1' }];
            vi.mocked(apiRequest).mockResolvedValueOnce(mockAgents);

            const result = await AgentApiService.getAgents();
            expect(result).toEqual(mockAgents);
        });

        it('should return agents from a HATEOAS data envelope', async () => {
            const mockAgents = [{ id: '2', name: 'Agent 2' }];
            vi.mocked(apiRequest).mockResolvedValueOnce({ data: mockAgents });

            const result = await AgentApiService.getAgents();
            expect(result).toEqual(mockAgents);
        });

        it('should return an empty array for undefined envelope data', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ data: null });
            const result = await AgentApiService.getAgents();
            expect(result).toEqual([]);
        });

        it('should return an empty array if response is null/invalid', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce(null);
            const result = await AgentApiService.getAgents();
            expect(result).toEqual([]);
        });
    });

    describe('updateAgent', () => {
        it('should send a PUT request with the correct body', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            const config = {
                name: 'Updated Name',
                modelConfig2: { modelId: 'model-2' } as any,
                modelConfig3: { modelId: 'model-3' } as any,
            };

            const result = await AgentApiService.updateAgent('agent-1', config);
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

    describe('createAgent', () => {
        it('should send a POST request with the new agent mapping', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            const newAgent = {
                id: 'new-agent',
                name: 'Test Agent',
                role: 'Tester',
                department: 'QA',
                model: 'model-1',
                budgetUsd: 10,
                themeColor: '#fff',
            } as any;

            const result = await AgentApiService.createAgent(newAgent);
            expect(result).toBe(true);
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents', expect.objectContaining({
                method: 'POST',
                body: expect.stringContaining('"name":"Test Agent"')
            }));
        });
    });

    describe('pauseAgent and resumeAgent', () => {
        it('should send POST to pause', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await AgentApiService.pauseAgent('agent-1');
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/pause', { method: 'POST' });
        });

        it('should send POST to resume', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({});
            await AgentApiService.resumeAgent('agent-1');
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/resume', { method: 'POST' });
        });
    });

    describe('sendCommand', () => {
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
            vi.mocked(useVaultStore.getState).mockReturnValue(mockVaultState as any);
            vi.mocked(useModelStore.getState).mockReturnValue(mockModelState as any);
            vi.mocked(useProviderStore.getState).mockReturnValue(mockProviderState as any);
        });

        it('should send command with API key and model limits if provider has a key', async () => {
            mockGetApiKey.mockResolvedValueOnce('secret-key');
            vi.mocked(apiRequest).mockResolvedValueOnce({});

            await AgentApiService.sendCommand('agent-1', 'Hello', 'test-model', 'openai');

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

            await AgentApiService.sendCommand('agent-1', 'Hello', 'test-model', 'openai');

            expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({
                text: expect.stringContaining('Using Server Environment for OPENAI.')
            }));
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                method: 'POST'
            }));
        });

        it('should ignore missing key for local providers and not emit event', async () => {
            mockGetApiKey.mockResolvedValueOnce(null);
            vi.mocked(apiRequest).mockResolvedValueOnce({});

            await AgentApiService.sendCommand('agent-1', 'Hello', 'test-model', PROVIDERS.OLLAMA);

            expect(EventBus.emit).not.toHaveBeenCalled();
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                method: 'POST'
            }));
        });

        it('should include X-Request-Id header if requestId is provided', async () => {
            mockGetApiKey.mockResolvedValueOnce('secret-key');
            vi.mocked(apiRequest).mockResolvedValueOnce({});

            await AgentApiService.sendCommand('agent-1', 'Hello', 'test-model', 'openai', undefined, undefined, undefined, undefined, undefined, undefined, 'req-123');

            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                headers: { 'X-Request-Id': 'req-123' }
            }));
        });
    });

    describe('Memory operations', () => {
        it('getAgentMemory', async () => {
            const mockRes = { status: 'ok', entries: [] };
            vi.mocked(apiRequest).mockResolvedValueOnce(mockRes);
            const res = await AgentApiService.getAgentMemory('agent-1');
            expect(res).toEqual(mockRes);
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/memories', { method: 'GET' });
        });

        it('deleteAgentMemory', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ status: 'ok' });
            await AgentApiService.deleteAgentMemory('agent-1', 'row-1');
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/memories/row-1', { method: 'DELETE' });
        });

        it('saveAgentMemory', async () => {
            vi.mocked(apiRequest).mockResolvedValueOnce({ status: 'ok', id: 'new-row' });
            await AgentApiService.saveAgentMemory('agent-1', 'new memory text');
            expect(apiRequest).toHaveBeenCalledWith('/v1/agents/agent-1/memories', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({ text: 'new memory text' })
            }));
        });
    });
});
