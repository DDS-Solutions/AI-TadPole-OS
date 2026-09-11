/**
 * @docs ARCHITECTURE:Contracts
 *
 * ### AI Context Alignment
 * - **Subsystem**: Test Verification Suite / symmetry.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect } from 'vitest';
import { normalize_agent_dto } from '../../domain/agents/normalizers';
import { serialize_agent_update } from '../../domain/agents/serializers';
import { normalize_role_blueprint, serialize_role } from '../../domain/roles/normalizer';
import type { AgentDto } from '../../contracts/agent/wire';
import type { Role_Blueprint_Dto } from '../../contracts/role/wire';

describe('Contract Symmetry Tests', () => {
    describe('Agent Symmetry', () => {
        it('should maintain field integrity through normalize -> serialize loop', () => {
            const mock_dto: AgentDto = {
                id: 'agent-123',
                name: 'Test Agent',
                role: 'Analyst',
                department: 'Engineering',
                status: 'idle',
                tokensUsed: 1000,
                model: 'claude-3-5-sonnet',
                activeModelSlot: 1,
                skills: ['coding', 'research'],
                workflows: [],
                mcpTools: [],
                budgetUsd: 50.0,
                costUsd: 1.5,
                requiresOversight: false,
                category: 'user',
                connectorConfigs: [],
                metadata: { 'internal_id': 'xyz' },
                tokenUsage: {
                    inputTokens: 400,
                    outputTokens: 600
                },
                failureCount: 0
            };

            const domain = normalize_agent_dto(mock_dto);
            const update_dto = serialize_agent_update(domain);

            // Verify flattened token counts are correctly mapped to nested wire update DTO
            expect(update_dto.tokenUsage?.inputTokens).toBe(mock_dto.tokenUsage?.inputTokens);
            expect(update_dto.tokenUsage?.outputTokens).toBe(mock_dto.tokenUsage?.outputTokens);
            expect(update_dto.budgetUsd).toBe(mock_dto.budgetUsd);
            expect(update_dto.name).toBe(mock_dto.name);
        });
    });

    describe('Role Symmetry', () => {
        it('should handle stringified JSON arrays correctly in normalize -> serialize loop', () => {
            const mock_dto: Role_Blueprint_Dto = {
                id: 'auditor-v1',
                name: 'Security Auditor',
                department: 'Engineering',
                description: 'Audits code for safety.',
                skills: JSON.stringify(['audit', 'rust']),
                workflows: JSON.stringify(['verification']),
                mcpTools: JSON.stringify(['code-analysis']),
                requiresOversight: true,
                modelId: 'gpt-4o'
            };

            const domain = normalize_role_blueprint(mock_dto);
            
            expect(domain.skills).toEqual(['audit', 'rust']);
            expect(domain.mcp_tools).toEqual(['code-analysis']);
            expect(domain.requires_oversight).toBe(true);

            const reserialized = serialize_role(domain);
            expect(reserialized.skills).toBe(mock_dto.skills);
            expect(reserialized.mcp_tools).toBe(mock_dto.mcpTools);
            expect(reserialized.requiresOversight).toBe(mock_dto.requiresOversight);
        });
    });
});
