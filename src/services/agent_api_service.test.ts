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
import { api_request, ApiError, AuthError, RateLimitError, ValidationError, ServerError } from './base_api_service';
import { use_provider_store } from '../stores/provider_store';
import { use_vault_store } from '../stores/vault_store';
import { use_model_store } from '../stores/model_store';
import { use_trace_store } from '../stores/trace_store';
import { event_bus } from './event_bus';

vi.mock('./base_api_service', async () => {
    const actual = await vi.importActual<typeof import('./base_api_service')>('./base_api_service');
    return {
        ...actual,
        api_request: vi.fn(),
    };
});

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
        emit_trace: vi.fn(),
        subscribe_traces: vi.fn(() => () => {}),
    },
}));

vi.mock('../stores/settings_store', () => ({
    get_settings: vi.fn(() => ({
        tadpole_os_url: 'http://localhost:8000',
        tadpole_os_api_key: 'test-key-mocked-for-telemetry-scrubbing'
    })),
    use_settings_store: {
        getState: vi.fn(() => ({
            settings: {
                tadpole_os_url: 'http://localhost:8000',
                tadpole_os_api_key: 'test-key-mocked-for-telemetry-scrubbing'
            }
        })),
    }
}));

describe('agent_api_service', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        agent_api_service.invalidate_cache();
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

        it('should implement Promise memoization (concurrency cache) and reuse the in-flight request', async () => {
            const mock_agents = [{ id: '3', name: 'Agent 3' }];
            vi.mocked(api_request).mockResolvedValueOnce(mock_agents);

            // Trigger multiple requests in parallel
            const p1 = agent_api_service.get_agents();
            const p2 = agent_api_service.get_agents();

            expect(p1).toBe(p2); // Promises must be identical (memoized)

            const res1 = await p1;
            const res2 = await p2;

            expect(res1).toEqual(mock_agents);
            expect(res2).toEqual(mock_agents);
            expect(api_request).toHaveBeenCalledTimes(1); // Only 1 network request triggered
        });

        it('should invalidate cache on mutating operations', async () => {
            const mock_agents = [{ id: '4', name: 'Agent 4' }];
            vi.mocked(api_request).mockResolvedValue(mock_agents);

            const p1 = agent_api_service.get_agents();
            await p1;

            // Invalidate via mutation
            await agent_api_service.create_agent({ id: 'new-agent' } as any);

            const p2 = agent_api_service.get_agents();
            expect(p1).not.toBe(p2); // Should trigger a new network call

            await p2;
            expect(api_request).toHaveBeenCalledTimes(3); // 1st get_agents, 1 create_agent, 2nd get_agents
        });

        it('should handle AbortSignal composition correctly', async () => {
            const mock_agents = [{ id: '5', name: 'Agent 5' }];
            // Resolves after a short delay
            vi.mocked(api_request).mockImplementationOnce(() => new Promise(resolve => setTimeout(() => resolve(mock_agents), 50)));

            const controller = new AbortController();
            const p_normal = agent_api_service.get_agents();
            const p_aborted = agent_api_service.get_agents({ signal: controller.signal });

            // Abort the second request
            controller.abort();

            await expect(p_aborted).rejects.toThrow(/abort/i);
            const res_normal = await p_normal;
            expect(res_normal).toEqual(mock_agents); // The normal request still successfully finishes!
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
            is_locked: false,
            is_unlocked: () => true
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

        it('should send command with API key and rate limits (positional arguments)', async () => {
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

        it('should support the new DispatchCommandInput DTO overload signature', async () => {
            mock_get_api_key.mockResolvedValueOnce('secret-key-dto');
            vi.mocked(api_request).mockResolvedValueOnce({});

            await agent_api_service.send_command({
                agent_id: 'agent-1',
                message: 'Hello DTO',
                model_id: 'test-model',
                provider: 'openai',
                budget_usd: 15
            });

            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                method: 'POST',
                body: expect.stringContaining('"message":"Hello DTO"')
            }));
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                body: expect.stringContaining('"api_key":"secret-key-dto"')
            }));
            expect(api_request).toHaveBeenCalledWith('/v1/agents/agent-1/tasks', expect.objectContaining({
                body: expect.stringContaining('"budget_usd":15')
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

    describe('Typed Errors Propagation', () => {
        it('should map API errors into specialized typed error classes', async () => {
            const buildError = (status: number) => {
                const err = new ApiError('Error message', 'about:blank', status);
                return err;
            };

            vi.mocked(api_request).mockRejectedValueOnce(buildError(401));
            await expect(agent_api_service.get_agents()).rejects.toThrow(AuthError);

            vi.mocked(api_request).mockRejectedValueOnce(buildError(429));
            await expect(agent_api_service.get_agents()).rejects.toThrow(RateLimitError);

            vi.mocked(api_request).mockRejectedValueOnce(buildError(400));
            await expect(agent_api_service.get_agents()).rejects.toThrow(ValidationError);

            vi.mocked(api_request).mockRejectedValueOnce(buildError(500));
            await expect(agent_api_service.get_agents()).rejects.toThrow(ServerError);
        });
    });

    describe('import_capability input validation', () => {
        it('should reject files exceeding 5MB', async () => {
            const oversized = new File(['a'.repeat(6 * 1024 * 1024)], 'skill.yaml', { type: 'text/yaml' });
            await expect(agent_api_service.import_capability(oversized)).rejects.toThrow(ValidationError);
            expect(api_request).not.toHaveBeenCalled();
        });

        it('should reject invalid file extensions', async () => {
            const bad_ext = new File(['code'], 'malicious.exe', { type: 'application/octet-stream' });
            await expect(agent_api_service.import_capability(bad_ext)).rejects.toThrow(ValidationError);
            expect(api_request).not.toHaveBeenCalled();
        });
    });

    describe('search_memory relative URLs', () => {
        it('should perform semantic searches using clean relative paths', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok', entries: [] });
            await agent_api_service.search_memory('agent behavior', 'agent-99');

            expect(api_request).toHaveBeenCalledWith(
                '/v1/search/memory?query=agent%20behavior&agent_id=agent-99',
                expect.objectContaining({ method: 'GET' })
            );
        });
    });

    describe('Telemetry Key Scrubbing (CWE-319/532 Option B)', () => {
        it('should redact api_key and authorization tokens from telemetry span attributes', async () => {
            const actual = await vi.importActual<typeof import('./base_api_service')>('./base_api_service');
            const real_api_request = actual.api_request;
            
            const mock_fetch = vi.fn().mockResolvedValue({
                ok: true,
                status: 204,
                text: async () => ''
            } as any);
            vi.stubGlobal('fetch', mock_fetch);

            const add_span_spy = vi.spyOn(use_trace_store.getState(), 'add_span');

            await real_api_request('/v1/test', {
                method: 'POST',
                body: JSON.stringify({ message: 'hello', api_key: 'sk-secret-123' })
            });

            expect(add_span_spy).toHaveBeenCalled();
            const span = add_span_spy.mock.calls[0][0];
            expect(span.attributes['http.request.body']).toContain('"[REDACTED]"');
            expect(span.attributes['http.request.body']).not.toContain('sk-secret-123');

            vi.unstubAllGlobals();
            add_span_spy.mockRestore();
        });

        it('should scrub secrets from FormData and plain objects correctly', async () => {
            const { scrub_secrets } = await vi.importActual<typeof import('./base_api_service')>('./base_api_service');

            const body_obj = { api_key: 'secret', token: 'oauth', message: 'hello' };
            const scrubbed_obj = scrub_secrets(body_obj) as any;
            expect(scrubbed_obj.api_key).toBe('[REDACTED]');
            expect(scrubbed_obj.token).toBe('[REDACTED]');
            expect(scrubbed_obj.message).toBe('hello');

            const form = new FormData();
            form.append('api_key', 'form-secret');
            form.append('name', 'form-name');
            const scrubbed_form = scrub_secrets(form) as FormData;
            expect(scrubbed_form.get('api_key')).toBe('[REDACTED]');
            expect(scrubbed_form.get('name')).toBe('form-name');
        });

        it('should scrub raw secrets from error messages in track_operation telemetry logs', async () => {
            const { track_operation } = await vi.importActual<typeof import('../utils/telemetry')>('../utils/telemetry');

            const sensitive_operation = async () => {
                throw new Error('Failed to connect with API key sk-proj-supersecretkey1234567890');
            };

            await expect(
                track_operation('TestAPI', 'Running operation with secret', sensitive_operation)
            ).rejects.toThrow();

            expect(event_bus.emit_log).toHaveBeenCalled();
            const log_calls = vi.mocked(event_bus.emit_log).mock.calls;
            const error_log = log_calls.find(call => call[0].severity === 'error');
            expect(error_log).toBeDefined();
            expect(error_log![0].text).not.toContain('sk-proj-supersecretkey1234567890');
            expect(error_log![0].text).toContain('[REDACTED]');
        });

        it('should scrub raw secrets from thrown ApiErrors in api_request', async () => {
            const actual = await vi.importActual<typeof import('./base_api_service')>('./base_api_service');
            const real_api_request = actual.api_request;

            const mock_fetch = vi.fn().mockResolvedValue({
                ok: false,
                status: 400,
                statusText: 'Bad Request',
                text: async () => JSON.stringify({
                    type: 'validation_error',
                    title: 'Bad Request',
                    detail: 'Invalid api_key: sk-proj-secret123456'
                })
            } as any);
            vi.stubGlobal('fetch', mock_fetch);

            await expect(
                real_api_request('/v1/test', { method: 'GET' })
            ).rejects.toThrow(/\[REDACTED\]/);

            vi.unstubAllGlobals();
        });
    });
});

// Metadata: [agent_api_service_test]
