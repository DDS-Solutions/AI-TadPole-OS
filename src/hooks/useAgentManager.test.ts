/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useAgentManager.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useAgentManager } from './useAgentManager';
import { use_agent_store } from '../stores/agent_store';
import { use_role_store } from '../stores/role_store';

describe('useAgentManager', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        use_agent_store.setState({
            agents: [
                { id: 'agent-1', name: 'Analyst', role: 'Research', category: 'user', department: 'Analytics' as any } as any,
                { id: 'agent-2', name: 'Developer', role: 'Engineering', category: 'ai', department: 'Tech' as any } as any
            ]
        });
        use_role_store.setState({
            roles: [
                { id: 'Research', title: 'Research', department: 'Analytics' as any, permissions: [] }
            ]
        });
    });

    it('initializes and filters agents by tab and search query', () => {
        const { result } = renderHook(() => useAgentManager());
        expect(result.current.agents.length).toBe(1); // category: 'user'

        act(() => {
            result.current.set_active_tab('ai');
        });
        expect(result.current.agents.length).toBe(1); // category: 'ai'

        act(() => {
            result.current.set_search_query('Nonexistent');
        });
        expect(result.current.agents.length).toBe(0);
    });

    it('handles agent selection', () => {
        const { result } = renderHook(() => useAgentManager());

        act(() => {
            result.current.set_selected_agent(use_agent_store.getState().agents[0]);
        });

        expect(result.current.selected_agent?.name).toBe('Analyst');
    });

    it('creates new agent draft via handle_add_new_click', () => {
        const { result } = renderHook(() => useAgentManager());

        act(() => {
            result.current.handle_add_new_click();
        });

        expect(result.current.is_creating).toBe(true);
        expect(result.current.selected_agent).toBeDefined();
        expect(result.current.selected_agent?.name).toBe('UNNAMED_NODE');
    });

    it('sorts agents from worst health (highest failure_count) to best health (0) when health_status is selected', () => {
        use_agent_store.setState({
            agents: [
                { id: 'agent-1', name: 'Healthy Agent', role: 'Research', category: 'user', failure_count: 0 } as any,
                { id: 'agent-2', name: 'Throttled Agent', role: 'Dev', category: 'user', failure_count: 5 } as any,
                { id: 'agent-3', name: 'Degraded Agent', role: 'Dev', category: 'user', failure_count: 2 } as any,
            ]
        });

        const { result } = renderHook(() => useAgentManager());

        act(() => {
            result.current.set_filter_role('health_status');
        });

        const names = result.current.agents.map(a => a.name);
        expect(names).toEqual(['Throttled Agent', 'Degraded Agent', 'Healthy Agent']);
    });
});
