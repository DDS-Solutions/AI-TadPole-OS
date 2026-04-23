/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Validation of the Agent Lifecycle API client.** 
 * Verifies the creation, configuration, and termination of individual neural nodes.
 * Mocks `base_api_service` (api_request) to isolate endpoint logic from network side-effects and backend latency.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Inconsistent agent state when the backend returns a 201 Created but the subsequent status poll fails.
 * - **Telemetry Link**: Search `[agent_api_service.test]` in tracing logs.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { agent_api_service } from './agent_api_service';
import { api_request } from './base_api_service';
import { use_provider_store } from '../stores/provider_store';
import { use_vault_store } from '../stores/vault_store';
import { use_model_store } from '../stores/model_store';


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
    });

    describe('update_agent', () => {
        it('should send a PUT request with the correct camelCase wire body', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            const config = {
                name: 'Updated Name',
                voice_id: 'alloy',
                budget_usd: 50,
                current_task: 'Testing recovery'
            };

            const result = await agent_api_service.update_agent('agent-1', config);
            expect(result).toBe(true);
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1', expect.objectContaining({
                method: 'PUT',
                body: expect.stringContaining('"name":"Updated Name"')
            }));
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1', expect.objectContaining({
                body: expect.stringContaining('"voiceId":"alloy"')
            }));
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1', expect.objectContaining({
                body: expect.stringContaining('"budgetUsd":50')
            }));
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1', expect.objectContaining({
                body: expect.stringContaining('"currentTask":"Testing recovery"')
            }));
        });
    });

    describe('create_agent', () => {
        it('should send a POST request with structural mapping', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            const new_agent = {
                id: 'new-agent',
                name: 'Test Agent',
                budget_usd: 10
            } as any;

            const result = await agent_api_service.create_agent(new_agent);
            expect(result).toBe(true);
            expect(api_request).toHaveBeenCalledWith('/v1/agents', expect.objectContaining({
                method: 'POST',
                body: expect.stringContaining('"name":"Test Agent"')
            }));
            expect(api_request).toHaveBeenCalledWith('/v1/agents', expect.objectContaining({
               body: expect.stringContaining('"budgetUsd":10')
            }));
        });
    });

    describe('send_command', () => {
        const mock_get_api_key = vi.fn();
        const mock_vault_state = {
            get_api_key: mock_get_api_key,
            is_locked: false
        };
        const mock_model_state = {
            models: [{ name: 'test-model', rpm: 10 }],
        };
        const mock_provider_state = {
            base_urls: { 'openai': 'https://api.openai.com/v1' },
        };

        beforeEach(() => {
            vi.mocked(use_vault_store.getState).mockReturnValue(mock_vault_state as any);
            vi.mocked(use_model_store.getState).mockReturnValue(mock_model_state as any);
            vi.mocked(use_provider_store.getState).mockReturnValue(mock_provider_state as any);
        });

        it('should send command with API key and rate limits', async () => {
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
        });
    });

    describe('save_role_blueprint', () => {
        it('stringifies capability arrays for the governance API', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});

            const result = await agent_api_service.save_role_blueprint({
                id: 'qa-lead',
                name: 'QA Lead',
                department: 'Quality Assurance',
                description: 'Regression gatekeeper',
                skills: ['audit', 'test'],
                workflows: ['release-review'],
                mcp_tools: ['list_files'],
                requires_oversight: true,
                model_id: 'gpt-4o',
                created_at: '2023-01-01'
            });

            expect(result).toBe(true);
            expect(api_request).toHaveBeenCalledWith('/v1/governance/blueprints', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({
                    id: 'qa-lead',
                    name: 'QA Lead',
                    department: 'Quality Assurance',
                    description: 'Regression gatekeeper',
                    skills: JSON.stringify(['audit', 'test']),
                    workflows: JSON.stringify(['release-review']),
                    mcp_tools: JSON.stringify(['list_files']),
                    requiresOversight: true,
                    modelId: 'gpt-4o',
                    createdAt: '2023-01-01'
                })
            }));
        });
    });
});

// Metadata: [agent_api_service_test]

// Metadata: [agent_api_service_test]
