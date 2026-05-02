/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[use_dashboard_data_test]` in observability traces.
 */

import { renderHook } from '@testing-library/react';
import { useDashboardData } from './use_dashboard_data';
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock all the stores and hooks
vi.mock('../stores/agent_store', () => ({
    use_agent_store: vi.fn(() => ({
        agents: [
            { id: '1', status: 'active', cost_usd: 0.5, tokens_used: 1000, created_at: new Date().toISOString() },
            { id: '2', status: 'idle', cost_usd: 0.2, tokens_used: 500, created_at: new Date().toISOString() }
        ],
        fetch_agents: vi.fn(),
        update_agent: vi.fn(),
        add_agent: vi.fn(),
        init_telemetry: vi.fn(() => vi.fn())
    }))
}));

vi.mock('../stores/node_store', () => ({
    use_node_store: vi.fn(() => ({
        nodes: [{ id: 'n1', status: 'healthy', running_agents: ['1'] }],
        fetch_nodes: vi.fn(),
        discover_nodes: vi.fn(),
        is_loading: false
    }))
}));

vi.mock('../hooks/use_engine_status', () => ({
    useEngineStatus: vi.fn(() => ({ is_online: true }))
}));

vi.mock('../services/event_bus', () => ({
    event_bus: {
        get_history: vi.fn(() => []),
        subscribe_logs: vi.fn(() => vi.fn())
    }
}));

vi.mock('../stores/workspace_store', () => ({
    use_workspace_store: vi.fn(() => ({
        clusters: [{ id: 'c1', collaborators: ['1'] }],
        toggle_cluster_active: vi.fn()
    }))
}));

vi.mock('../stores/role_store', () => ({
    use_role_store: vi.fn((selector: any) => selector({
        roles: { 'Architect': {}, 'Developer': {} }
    }))
}));

describe('useDashboardData', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('aggregates agent and node data correctly', () => {
        const { result } = renderHook(() => useDashboardData());

        expect(result.current.agents_count).toBe(2);
        expect(result.current.is_online).toBe(true);
        expect(result.current.nodes).toHaveLength(1);
    });

    it('calculates total cost and tokens correctly', () => {
        const { result } = renderHook(() => useDashboardData());

        expect(result.current.total_cost).toBe(0.7);
        expect(result.current.total_tokens).toBe(1500);
    });

    it('identifies active agents in assigned clusters', () => {
        const { result } = renderHook(() => useDashboardData());

        // Agent 1 is active and in cluster collaborators
        // Agent 2 is idle and NOT in cluster collaborators
        expect(result.current.active_agents).toBe(1);
    });

    it('calculates recruitment velocity correctly', () => {
        const { result } = renderHook(() => useDashboardData());

        // Both agents were created "now" in the mock
        expect(result.current.recruit_velocity).toBe(2);
    });

    it('extracts available roles from role store', () => {
        const { result } = renderHook(() => useDashboardData());

        expect(result.current.available_roles).toEqual(['Architect', 'Developer']);
    });
});

// Metadata: [use_dashboard_data_test]
