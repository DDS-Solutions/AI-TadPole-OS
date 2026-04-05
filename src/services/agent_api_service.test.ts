/**
 * @docs ARCHITECTURE:Services
 * 
 * ### AI Assist Note
 * **Test Suite**: Validates the REST interface for agent registry mutations. 
 */

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
 * - Mocks base_api_service (api_request) to prevent network side-effects.
 * - Mocks provider_store, vault_store, and model_store.
 * - Refactored for 100% snake_case architectural parity.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { agent_api_service } from './agent_api_service';
import { api_request } from './base_api_service';
import { use_provider_store } from '../stores/provider_store';
import { use_vault_store } from '../stores/vault_store';
import { use_model_store } from '../stores/model_store';
import { event_bus } from './event_bus';
import { PROVIDERS } from '../constants';

vi.mock('./base_api_service', () => ({
    api_request: vi.fn(),
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
        emit_log: vi.fn(),
    },
}));

describe('agent_api_service', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('get_agents', () => {
        it('should return agents from a direct array response', async () => {
            const mock_agents = [{ id: '1', name: 'Agent 1' }];
            vi.mocked(api_request).mockResolvedValueOnce(mock_agents);

            const result = await agent_api_service.get_agents();
            expect(result).toEqual(mock_agents);
        });

        it('should return agents from a HATEOAS data envelope', async () => {
            const mock_agents = [{ id: '2', name: 'Agent 2' }];
            vi.mocked(api_request).mockResolvedValueOnce({ data: mock_agents });

            const result = await agent_api_service.get_agents();
            expect(result).toEqual(mock_agents);
        });

        it('should return an empty array for undefined envelope data', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ data: null });
            const result = await agent_api_service.get_agents();
            expect(result).toEqual([]);
        });

        it('should return an empty array if response is null/invalid', async () => {
            vi.mocked(api_request).mockResolvedValueOnce(null);
            const result = await agent_api_service.get_agents();
            expect(result).toEqual([]);
        });
    });

    describe('update_agent', () => {
        it('should send a PUT request with the correct body', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            const config = {
                name: 'Updated Name',
                model_config2: { model_id: 'model-2' } as any,
                model_config3: { model_id: 'model-3' } as any,
            };

            const result = await agent_api_service.update_agent('agent-1', config);
            expect(result).toBe(true);
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1', expect.objectContaining({
                method: 'PUT',
                body: expect.stringContaining('"name":"Updated Name"')
            }));
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1', expect.objectContaining({
                body: expect.stringContaining('"model2":"model-2"')
            }));
        });
    });

    describe('create_agent', () => {
        it('should send a POST request with the new agent mapping', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            const new_agent = {
                id: 'new-agent',
                name: 'Test Agent',
                role: 'Tester',
                department: 'QA',
                model: 'model-1',
                budget_usd: 10,
                theme_color: '#fff',
            } as any;

            const result = await agent_api_service.create_agent(new_agent);
            expect(result).toBe(true);
            expect(api_request).toHaveBeenCalledWith('/v1/agents', expect.objectContaining({
                method: 'POST',
                body: expect.stringContaining('"name":"Test Agent"')
            }));
        });
    });

    describe('pause_agent and resume_agent', () => {
        it('should send POST to pause', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await agent_api_service.pause_agent('agent-1');
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1/pause', { method: 'POST' });
        });

        it('should send POST to resume', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await agent_api_service.resume_agent('agent-1');
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1/resume', { method: 'POST' });
        });
    });

    describe('send_command', () => {
        const mock_get_api_key = vi.fn();
        const mock_vault_state = {
            get_api_key: mock_get_api_key,
            is_locked: false
        };
        const mock_model_state = {
            models: [{ name: 'test-model', rpm: 10, tpm: 1000, rpd: 100, tpd: 10000 }],
        };
        const mock_provider_state = {
            base_urls: { 'openai': 'https://api.openai.com/v1' },
        };

        beforeEach(() => {
            vi.mocked(use_vault_store.getState).mockReturnValue(mock_vault_state as any);
            vi.mocked(use_model_store.getState).mockReturnValue(mock_model_state as any);
            vi.mocked(use_provider_store.getState).mockReturnValue(mock_provider_state as any);
        });

        it('should send command with API key and model limits if provider has a key', async () => {
            mock_get_api_key.mockResolvedValueOnce('secret-key');
            vi.mocked(api_request).mockResolvedValueOnce({});

            await agent_api_service.send_command('agent-1', 'Hello', 'test-model', 'openai');

            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                method: 'POST',
                body: expect.stringContaining('"api_key":"secret-key"')
            }));
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                body: expect.stringContaining('"rpm":10')
            }));
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                body: expect.stringContaining('"base_url":"https://api.openai.com/v1"')
            }));
        });

        it('should emit a security info event if no key is present for a remote provider', async () => {
            mock_get_api_key.mockResolvedValueOnce(null);
            vi.mocked(api_request).mockResolvedValueOnce({});

            await agent_api_service.send_command('agent-1', 'Hello', 'test-model', 'openai');

            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                text: expect.stringContaining('🔒 Neural Security: No Key for OPENAI for AGENT-1.'),
                severity: 'warning'
            }));
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                method: 'POST'
            }));
        });

        it('should ignore missing key for local providers and not emit event', async () => {
            mock_get_api_key.mockResolvedValueOnce(null);
            vi.mocked(api_request).mockResolvedValueOnce({});

            await agent_api_service.send_command('agent-1', 'Hello', 'test-model', PROVIDERS.OLLAMA);

            expect(event_bus.emit_log).not.toHaveBeenCalled();
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                method: 'POST'
            }));
        });

        it('should include X-Request-Id header if request_id is provided', async () => {
            mock_get_api_key.mockResolvedValueOnce('secret-key');
            vi.mocked(api_request).mockResolvedValueOnce({});

            await agent_api_service.send_command('agent-1', 'Hello', 'test-model', 'openai', undefined, undefined, undefined, undefined, undefined, undefined, 'req-123');

            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                headers: { 'X-Request-Id': 'req-123' }
            }));
        });
    });

    describe('Memory operations', () => {
        it('get_agent_memory', async () => {
            const mock_res = { status: 'ok', entries: [] };
            vi.mocked(api_request).mockResolvedValueOnce(mock_res);
            const res = await agent_api_service.get_agent_memory('agent-1');
            expect(res).toEqual(mock_res);
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1/memories', { method: 'GET' });
        });

        it('delete_agent_memory', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });
            await agent_api_service.delete_agent_memory('agent-1', 'row-1');
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1/memories/row-1', { method: 'DELETE' });
        });

        it('save_agent_memory', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok', id: 'new-row' });
            await agent_api_service.save_agent_memory('agent-1', 'new memory text');
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1/memories', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({ text: 'new memory text' })
            }));
        });
    });
});

