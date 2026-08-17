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
import { useMissionsManager } from './useMissionsManager';
import { use_workspace_store } from '../stores/workspace_store';
import { use_agent_store } from '../stores/agent_store';

describe('useMissionsManager', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        use_workspace_store.setState({
            clusters: [
                { id: 'cluster-1', name: 'Alpha Mission', agents: ['agent-1'], objective: 'Research', department: 'Executive' } as any
            ]
        });
        use_agent_store.setState({
            agents: [
                { id: 'agent-1', name: 'Lead Agent' } as any
            ]
        });
    });

    it('initializes and selects active cluster', () => {
        const { result } = renderHook(() => useMissionsManager());
        expect(result.current.selected_cluster_id).toBe('cluster-1');
        expect(result.current.active_cluster?.name).toBe('Alpha Mission');
    });

    it('handles cluster selection updates', () => {
        const { result } = renderHook(() => useMissionsManager());

        act(() => {
            result.current.set_selected_cluster_id('cluster-2');
        });

        expect(result.current.selected_cluster_id).toBe('cluster-2');
    });

    it('creates new cluster with auto-assigned name', () => {
        const { result } = renderHook(() => useMissionsManager());

        act(() => {
            result.current.create_cluster({
                name: 'Beta Mission',
                department: 'Engineering'
            });
        });

        expect(use_workspace_store.getState().clusters.length).toBe(2);
    });
});
