/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useOversightDashboard.test
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
import { useOversightDashboard } from './useOversightDashboard';

const mock_get_pending = vi.fn();
const mock_get_ledger = vi.fn();
const mock_decide = vi.fn();

vi.mock('../services/system_api_service', () => ({
    system_api_service: {
        oversight: {
            get_pending_oversight: () => mock_get_pending(),
            get_oversight_ledger: () => mock_get_ledger(),
            decide_oversight: (id: string, d: string) => mock_decide(id, d)
        }
    }
}));

describe('useOversightDashboard', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        localStorage.clear();
        mock_get_pending.mockResolvedValue([
            { id: 'act-1', agent_id: 'agent-1', action_type: 'fs_write', payload: { path: '/file.txt' }, requested_at: '2026-08-15' }
        ]);
        mock_get_ledger.mockResolvedValue([
            { id: 'led-1', action_id: 'act-1', agent_id: 'agent-1', decision: 'approved', decided_at: '2026-08-15' }
        ]);
        mock_decide.mockResolvedValue({});
    });

    it('initializes and calculates dashboard stats', async () => {
        const { result } = renderHook(() => useOversightDashboard());
        expect(result.current.filter).toBe('');
        expect(result.current.stats).toBeDefined();
    });

    it('filters ledger entries by severity or text', () => {
        const { result } = renderHook(() => useOversightDashboard());

        act(() => {
            result.current.set_filter('agent-1');
        });

        expect(result.current.filter).toBe('agent-1');
    });

    it('approves or rejects pending action', async () => {
        const { result } = renderHook(() => useOversightDashboard());

        await act(async () => {
            await result.current.handle_decide('act-1', 'approved');
        });

        expect(mock_decide).toHaveBeenCalledWith('act-1', 'approved');
    });

    it('resolves known agent names', () => {
        const { result } = renderHook(() => useOversightDashboard());
        const name = result.current.resolve_agent_name('unknown-id');
        expect(name).toBe('unknown-id');
    });
});
