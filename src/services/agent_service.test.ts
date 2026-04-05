/**
 * @docs ARCHITECTURE:Services
 * 
 * ### AI Assist Note
 * **Test Suite**: Validates the Business logic for agent swarm orchestration. 
 */

/**
 * @file agent_service.test.ts
 * @description Suite for verifying the Agent Service orchestration logic.
 * @module Services/agent_service
 * @testedBehavior
 * - Normalization: Transformation of raw backend agent payloads to frontend-ready Agent interfaces.
 * - Persistence Mapping: Verification of field-level updates and model-to-provider auto-resolution.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks agent_api_service to isolate high-level service logic from network I/O.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { normalize_agent, persist_agent_update, type Raw_Agent } from './agent_service';
import { agent_api_service } from './agent_api_service';

vi.mock('./agent_api_service', () => ({
    agent_api_service: {
        get_agents: vi.fn(),
        update_agent: vi.fn(),
    }
}));

vi.mock('./system_api_service', () => ({
    system_api_service: {
        check_health: vi.fn(),
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
                current_task: 'Investigating regression',
                tokens_used: 500,
                token_usage: {
                    input_tokens: 300,
                    output_tokens: 200,
                    total_tokens: 500
                },
                skills: ['search', 'read'],
                workflows: ['workflow-1'],
                budget_usd: 10.0,
                cost_usd: 1.5,
                failure_count: 2,
                last_failure_at: '2023-01-01T00:00:00Z',
                model: 'gpt-4o'
            };

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
            } as const;

            const normalized_from_backend = normalize_agent({
                id: 'agent-1',
                name: 'Existing',
                status: 'active',
                current_task: 'Fresh task from backend'
            }, undefined, existing_agent);

            const normalized_fallback = normalize_agent({
                id: 'agent-1',
                name: 'Existing',
                status: 'active'
            }, undefined, existing_agent);

            expect(normalized_from_backend.current_task).toBe('Fresh task from backend');
            expect(normalized_fallback.current_task).toBe('Existing task');
        });

        it('handles missing fields with defaults', () => {
            const raw_agent: Raw_Agent = { id: '2', name: '' };
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
            };
            const normalized_agent = normalize_agent(raw_agent);
            expect(normalized_agent.role).toBe('Architect');
            expect(normalized_agent.department).toBe('Executive');
        });
    });

    describe('persist_agent_update', () => {
        it('calls agent_api_service.update_agent with correctly mapped payload', async () => {
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
                model_id: 'claude-3-sonnet',
                provider: 'anthropic'
            });
        });

        it('uses model_config provided explicitly', async () => {
            await persist_agent_update('agent-1', {
                model_config: {
                    model_id: 'custom-model',
                    provider: 'ollama',
                    temperature: 0.5,
                    system_prompt: 'hi'
                }
            });

            expect(agent_api_service.update_agent).toHaveBeenCalledWith('agent-1', {
                model_id: 'custom-model',
                provider: 'ollama',
                temperature: 0.5,
                system_prompt: 'hi'
            });
        });
    });
});

