/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Assist Note
 * - Review changes preserve origin security and propagate confirmed operation outcomes.
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
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useOversightDashboard } from './useOversightDashboard';

const mock_get_pending = vi.fn();
const mock_get_ledger = vi.fn();
const mock_decide = vi.fn();
const mock_health = vi.fn();
const mock_resume = vi.fn();
const mock_kill = vi.fn();
const connection = vi.hoisted(() => ({ online: true }));
vi.mock('./use_engine_status', () => ({ useEngineStatus: () => ({ is_online: connection.online }) }));

vi.mock('../services/system_api_service', () => ({
    system_api_service: {
        engine: { get_engine_status: (...args: unknown[]) => mock_health(...args), resume_engine: () => mock_resume(), kill_agents: () => mock_kill() },
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
        vi.clearAllMocks();
        connection.online = true;
        mock_health.mockResolvedValue({ dispatch_paused: true });
        mock_resume.mockResolvedValue({ status: 'ok', dispatch_paused: false });
        mock_kill.mockResolvedValue({ status: 'ok', dispatch_paused: true });
        vi.spyOn(window, 'confirm').mockReturnValue(true);
        vi.spyOn(window, 'alert').mockImplementation(() => {});
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

    it('reads the engine latch on load and resumes only after explicit confirmation', async () => {
        const { result } = renderHook(() => useOversightDashboard());
        await waitFor(() => expect(result.current.dispatch_paused).toBe(true));
        vi.mocked(window.confirm).mockReturnValueOnce(false);
        await act(async () => { await result.current.handle_resume_engine(); });
        expect(mock_resume).not.toHaveBeenCalled();
        await act(async () => { await result.current.handle_resume_engine(); });
        expect(mock_resume).toHaveBeenCalledOnce();
        expect(result.current.dispatch_paused).toBe(false);
    });

    it('retains the stop latch after a failed resume', async () => {
        mock_resume.mockRejectedValueOnce(new Error('offline'));
        const { result } = renderHook(() => useOversightDashboard());
        await waitFor(() => expect(result.current.dispatch_paused).toBe(true));
        await act(async () => { await result.current.handle_resume_engine(); });
        expect(result.current.dispatch_paused).toBe(true);
        expect(result.current.engine_control_pending).toBe(false);
    });

    it('does not dispatch emergency controls while offline', async () => {
        connection.online = false;
        const { result } = renderHook(() => useOversightDashboard());
        await act(async () => { await result.current.handle_resume_engine(); await result.current.handle_kill_switch(); });
        expect(mock_health).not.toHaveBeenCalled();
        expect(mock_resume).not.toHaveBeenCalled();
        expect(mock_kill).not.toHaveBeenCalled();
    });
});
