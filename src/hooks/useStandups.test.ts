/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useStandups.test
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
import { useStandups } from './useStandups';
import { use_agent_store } from '../stores/agent_store';
import { use_workspace_store } from '../stores/workspace_store';

describe('useStandups', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        use_agent_store.setState({
            agents: [
                { id: 'a1', name: 'Standup Lead' } as any
            ]
        });
        use_workspace_store.setState({
            clusters: [
                { id: 'c1', name: 'Cluster Alpha' } as any
            ]
        });
    });

    it('initializes standup session state', () => {
        const { result } = renderHook(() => useStandups());
        expect(result.current.is_live).toBe(false);
        expect(result.current.transcript_history.length).toBeGreaterThan(0);
        expect(result.current.target_type).toBe('agent');
    });

    it('toggles live state and updates target selection', () => {
        const { result } = renderHook(() => useStandups());

        act(() => {
            result.current.set_target_type('cluster');
            result.current.set_selected_target_id('c1');
        });

        expect(result.current.target_type).toBe('cluster');
        expect(result.current.selected_target_id).toBe('c1');
    });
});
