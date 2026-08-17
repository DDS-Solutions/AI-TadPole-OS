/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Assist Note
 * Regression coverage for the adjacent production module and its public contracts.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Contract, rendering, state transition, or error-handling regression.
 * - **Trace Scope**: Vitest assertions and test-local mocks.
 */

import { describe, it, expect } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useAgentMetrics } from './use_agent_metrics';
import type { Agent } from '../types';

describe('useAgentMetrics', () => {
    it('calculates token sums, cost totals, and budget utilization', () => {
        const mock_agents: Agent[] = [
            {
                id: 'a1',
                name: 'Agent 1',
                status: 'active',
                cost_usd: 0.50,
                budget_usd: 2.00,
                tokens_used: 1000,
                input_tokens: 600,
                output_tokens: 400
            } as any,
            {
                id: 'a2',
                name: 'Agent 2',
                status: 'offline',
                cost_usd: 0.25,
                budget_usd: 1.00,
                tokens_used: 500,
                input_tokens: 300,
                output_tokens: 200
            } as any
        ];

        const assigned = new Set(['a1', 'a2']);

        const { result } = renderHook(() => useAgentMetrics({
            agents_list: mock_agents,
            assigned_agent_ids: assigned
        }));

        expect(result.current.active_agents).toBe(1);
        expect(result.current.online_count).toBe(1);
        expect(result.current.total_cost).toBe(0.75);
        expect(result.current.total_budget).toBe(3.00);
        expect(result.current.budget_util).toBe(25);
        expect(result.current.total_tokens).toBe(1500);
        expect(result.current.total_input_tokens).toBe(900);
        expect(result.current.total_output_tokens).toBe(600);
    });

    it('safely handles empty lists and zero budgets without division by zero', () => {
        const { result } = renderHook(() => useAgentMetrics({
            agents_list: [],
            assigned_agent_ids: new Set()
        }));

        expect(result.current.active_agents).toBe(0);
        expect(result.current.total_cost).toBe(0);
        expect(result.current.budget_util).toBe(0);
    });
});
