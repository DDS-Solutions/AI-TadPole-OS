import { describe, it, expect, vi, beforeEach } from 'vitest';
import { normalizeAgent, persistAgentUpdate, type RawAgent } from './agentService';
import { AgentApiService } from './AgentApiService';

vi.mock('./AgentApiService', () => ({
    AgentApiService: {
        getAgents: vi.fn(),
        updateAgent: vi.fn(),
    }
}));

vi.mock('./SystemApiService', () => ({
    SystemApiService: {
        checkHealth: vi.fn(),
    }
}));

describe('agentService', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('normalizeAgent', () => {
        it('normalizes a full raw agent correctly', () => {
            const raw: RawAgent = {
                id: 'agent-1',
                name: 'Test Agent',
                role: 'Analyst',
                department: 'Research',
                status: 'working',
                tokens_used: 500,
                skills: ['search', 'read'],
                workflows: ['workflow-1'],
                budgetUsd: 10.0,
                costUsd: 1.5,
                failure_count: 2,
                last_failure_at: '2023-01-01T00:00:00Z',
                model: 'gpt-4o'
            };

            const normalized = normalizeAgent(raw, '/tmp/workspace');

            expect(normalized.id).toBe('agent-1');
            expect(normalized.name).toBe('Test Agent');
            expect(normalized.status).toBe('active'); // 'working' -> 'active'
            expect(normalized.tokensUsed).toBe(500);
            expect(normalized.skills).toEqual(['search', 'read']);
            expect(normalized.workflows).toEqual(['workflow-1']);
            expect(normalized.department).toBe('Research');
            expect(normalized.failureCount).toBe(2);
            expect(normalized.workspacePath).toBe('/tmp/workspace');
        });

        it('handles missing fields with defaults', () => {
            const raw: RawAgent = { id: '2', name: '' };
            const normalized = normalizeAgent(raw);

            expect(normalized.name).toBe('Unnamed Agent');
            expect(normalized.role).toBe('AI Agent');
            expect(normalized.department).toBe('Operations');
            expect(normalized.skills).toEqual([]);
            expect(normalized.tokensUsed).toBe(0);
        });

        it('prefers metadata for role and department if primary fields missing', () => {
            const raw: RawAgent = { 
                id: '3', 
                name: 'MetaAgent',
                metadata: { role: 'Architect', department: 'Executive' }
            };
            const normalized = normalizeAgent(raw);
            expect(normalized.role).toBe('Architect');
            expect(normalized.department).toBe('Executive');
        });
    });

    describe('persistAgentUpdate', () => {
        it('calls AgentApiService.updateAgent with correctly mapped payload', async () => {
            await persistAgentUpdate('agent-1', {
                name: 'New Name',
                skills: ['skill-1'],
                budgetUsd: 20
            });

            expect(AgentApiService.updateAgent).toHaveBeenCalledWith('agent-1', {
                name: 'New Name',
                skills: ['skill-1'],
                budgetUsd: 20
            });
        });

        it('resolves provider automatically for common models if missing', async () => {
            await persistAgentUpdate('agent-1', {
                model: 'claude-3-sonnet'
            });

            expect(AgentApiService.updateAgent).toHaveBeenCalledWith('agent-1', {
                modelId: 'claude-3-sonnet',
                provider: 'anthropic'
            });
        });

        it('uses modelConfig provided explicitly', async () => {
            await persistAgentUpdate('agent-1', {
                modelConfig: {
                    modelId: 'custom-model',
                    provider: 'ollama',
                    temperature: 0.5,
                    systemPrompt: 'hi'
                }
            });

            expect(AgentApiService.updateAgent).toHaveBeenCalledWith('agent-1', {
                modelId: 'custom-model',
                provider: 'ollama',
                temperature: 0.5,
                systemPrompt: 'hi'
            });
        });
    });
});
