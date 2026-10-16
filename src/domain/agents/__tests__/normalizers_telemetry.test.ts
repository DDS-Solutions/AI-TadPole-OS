/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Domain / Agents / normalizers_telemetry.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect } from 'vitest';
import { normalize_agent_dto } from '../normalizers';
import type { AgentDto } from '../../../contracts/agent';

describe('normalize_agent_dto Token and Telemetry Parity', () => {
    it('prioritizes accumulated lifetime tokensUsed over turn-level tokenUsage.totalTokens', () => {
        const raw_dto = {
            id: '1',
            name: 'Agent of Nine',
            role: 'CEO',
            department: 'Executive',
            status: 'idle',
            tokensUsed: 2698905,
            tokenUsage: {
                inputTokens: 8000,
                outputTokens: 4000,
                totalTokens: 12000
            },
            costUsd: 5.22,
            budgetUsd: 10.0
        } as unknown as AgentDto;

        const normalized = normalize_agent_dto(raw_dto);
        expect(normalized.tokens_used).toBe(2698905);
        expect(normalized.cost_usd).toBe(5.22);
        expect(normalized.budget_usd).toBe(10.0);
    });

    it('correctly maps snake_case tokens_used and cost_usd from database/backend models', () => {
        const raw_dto = {
            id: '2',
            name: 'Tadpole Alpha',
            role: 'COO',
            department: 'Operations',
            status: 'idle',
            tokens_used: 3924829,
            cost_usd: 8.2274,
            budget_usd: 25.0
        } as unknown as AgentDto;

        const normalized = normalize_agent_dto(raw_dto);
        expect(normalized.tokens_used).toBe(3924829);
        expect(normalized.cost_usd).toBe(8.2274);
        expect(normalized.budget_usd).toBe(25.0);
    });

    it('falls back to tokenUsage.totalTokens when lifetime counter is omitted', () => {
        const raw_dto = {
            id: 'coder',
            name: 'Coder Agent',
            role: 'Developer',
            department: 'Engineering',
            status: 'active',
            tokenUsage: {
                inputTokens: 1500,
                outputTokens: 2500,
                totalTokens: 4000
            }
        } as unknown as AgentDto;

        const normalized = normalize_agent_dto(raw_dto);
        expect(normalized.tokens_used).toBe(4000);
        expect(normalized.input_tokens).toBe(1500);
        expect(normalized.output_tokens).toBe(2500);
    });

    it('falls back to existing_agent state when token fields are undefined in partial update', () => {
        const existing = {
            id: 'audit_specialist',
            name: 'Audit Specialist',
            role: 'Auditor',
            department: 'Security',
            status: 'idle',
            tokens_used: 390903,
            cost_usd: 0.7828,
            budget_usd: 15.0,
            model: 'gemma4:12b',
            skills: [],
            workflows: [],
            mcp_tools: [],
            active_model_slot: 1 as const,
            failure_count: 0,
            connector_configs: [],
            metadata: {},
            requires_oversight: false
        };

        const partial_dto = {
            id: 'audit_specialist',
            status: 'active'
        } as unknown as AgentDto;

        const normalized = normalize_agent_dto(partial_dto, undefined, existing);
        expect(normalized.status).toBe('active');
        expect(normalized.tokens_used).toBe(390903);
        expect(normalized.cost_usd).toBe(0.7828);
    });
});
