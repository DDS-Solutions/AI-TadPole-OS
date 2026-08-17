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
});
