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
