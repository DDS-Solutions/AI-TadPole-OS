/**
 * @docs ARCHITECTURE:Contracts
 * 
 * ### AI Assist Note
 * **Test Suite**: Verifies the bidirectional mapping between frontend Domain models 
 * and backend Wire DTOs. Ensures that normalization logic handles edge cases 
 * like stringified JSON arrays and legacy department names.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Regression in snake_case to camelCase conversion or failed parsing of SQLx JSON fields.
 * - **Telemetry Link**: Not tracked (Unit Test).
 */

import { describe, it, expect } from 'vitest';
import { normalize_agent_dto as from_backend_agent } from '../domain/agents/normalizers';
import { serialize_agent_update as to_agent_update_payload } from '../domain/agents/serializers';
import type { Agent } from '../contracts/agent/domain';

describe('agent_mappers', () => {
    describe('from_backend_agent', () => {
        it('should correctly normalize a raw backend agent', () => {
            const raw = {
                id: 'agent-1',
                name: 'Test Agent',
                role: 'Analyst',
                department: 'QA',
                budgetUsd: 100,
                costUsd: 10,
                tokensUsed: 1000,
                tokenUsage: {
                    inputTokens: 400,
                    outputTokens: 600,
                    totalTokens: 1000
                },
                skills: JSON.stringify(['skill1', 'skill2']),
                workflows: JSON.stringify(['flow1']),
                metadata: { custom: 'data' },
                model: 'gpt-4o',
                modelConfig: {
                    provider: 'openai',
                    temperature: 0.7
                }
            };

            const normalized = from_backend_agent(raw as any, './workspaces/test');
            
            expect(normalized.id).toBe('agent-1');
            expect(normalized.budget_usd).toBe(100);
            expect(normalized.skills).toContain('skill1');
            expect(normalized.model_config?.provider).toBe('openai');
            expect(normalized.workspace_path).toBe('./workspaces/test');
        });
    });

    describe('from_backend_agent (Robustness)', () => {
        it('should handle hybrid DTOs (mixed snake and camel case)', () => {
            const hybrid = {
                id: 'agent-hybrid',
                name: 'Hybrid Agent',
                role: 'Hybrid',
                department: 'Operations',
                budget_usd: 500, // snake
                costUsd: 50,      // camel
                tokens_used: 1000,
                modelId: 'gpt-4o' // camel
            };

            const normalized = from_backend_agent(hybrid as any);
            
            expect(normalized.budget_usd).toBe(500);
            expect(normalized.cost_usd).toBe(50);
            expect(normalized.model).toBe('gpt-4o');
        });

        it('should preserve identity from existing_agent when DTO has missing fields', () => {
            const existing: Agent = {
                id: 'agent-1',
                name: 'Original Name',
                role: 'Original Role',
                department: 'Engineering',
                status: 'idle',
                tokens_used: 0,
                model: 'claude-3',
                skills: ['old-skill'],
                workflows: [],
                budget_usd: 100
            };

            const partial_dto = {
                id: 'agent-1',
                name: 'Updated Name',
                // missing role, department, etc.
            };

            const normalized = from_backend_agent(partial_dto as any, undefined, existing);
            
            expect(normalized.name).toBe('Updated Name');
            expect(normalized.role).toBe('Original Role'); // Preserved
            expect(normalized.budget_usd).toBe(100);       // Preserved
        });

        it('should map legacy department names', () => {
            const legacy = { id: 'a', department: 'QA' };
            const normalized = from_backend_agent(legacy as any);
            expect(normalized.department).toBe('Quality Assurance');
        });

        it('should handle stringified skills from SQLx', () => {
            const raw = { id: 'a', skills: '["rust", "ts"]' };
            const normalized = from_backend_agent(raw as any);
            expect(normalized.skills).toEqual(['rust', 'ts']);
        });
    });

    describe('to_agent_update_payload', () => {
        it('should map frontend partials to backend DTOs with camelCase', () => {
            const updates: Partial<Agent> = {
                name: 'New Name',
                budget_usd: 500,
                skills: ['new-skill']
            };

            const payload = to_agent_update_payload(updates as any);
            
            expect(payload.name).toBe('New Name');
            expect(payload.budgetUsd).toBe(500);
            expect(payload.skills).toEqual(['new-skill']);
            // Ensure snake_case from Agent is mapped to camelCase in payload
            expect(payload).toHaveProperty('budgetUsd');
            expect(payload).not.toHaveProperty('budget_usd');
        });
    });
});

// Metadata: [agent_mappers_test]

// Metadata: [agent_mappers_test]
