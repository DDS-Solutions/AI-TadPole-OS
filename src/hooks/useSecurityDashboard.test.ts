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
import { useSecurityDashboard } from './useSecurityDashboard';

const mock_get_security_snapshot = vi.fn();
const mock_update_security_quota = vi.fn();

vi.mock('../services/system_api_service', () => ({
    system_api_service: {
        oversight: {
            get_security_snapshot: (...args: any[]) => mock_get_security_snapshot(...args),
            update_security_quota: (...args: any[]) => mock_update_security_quota(...args)
        }
    }
}));

describe('useSecurityDashboard', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        mock_get_security_snapshot.mockResolvedValue({
            quotas: {
                agent_quotas: [
                    { entity_id: 'agent-1', current_spend_usd: 0.25, max_failures: 3, current_failures: 0, budget_limit_usd: 1.0 }
                ]
            },
            audit_trail: {
                data: [
                    { id: 'audit-1', tool_name: 'terminal', timestamp: '2026-08-15T00:00:00Z', permission_mode: 'Allow', parameters: {} }
                ],
                total: 1
            },
            agent_health: [
                { agent_id: 'agent-1', name: 'Scout', status: 'active', failures: 0, last_heartbeat: 12345 }
            ]
        });
        mock_update_security_quota.mockResolvedValue({});
    });

    it('initializes and loads security quotas and health records', async () => {
        const { result } = renderHook(() => useSecurityDashboard(0));

        await act(async () => {
            await result.current.refresh();
        });

        expect(result.current.is_loading).toBe(false);
        expect(result.current.sorted_health.length).toBe(1);
        expect(result.current.sorted_quotas.length).toBe(1);
    });

    it('filters agent health records based on search query', async () => {
        const { result } = renderHook(() => useSecurityDashboard(0));

        await act(async () => {
            await result.current.refresh();
        });

        act(() => {
            result.current.set_health_search('unknown');
        });

        expect(result.current.sorted_health.length).toBe(0);

        act(() => {
            result.current.set_health_search('scout');
        });

        expect(result.current.sorted_health.length).toBe(1);
    });

    it('updates quotas via system API', async () => {
        const { result } = renderHook(() => useSecurityDashboard(0));

        await act(async () => {
            await result.current.update_quota('agent-1', 1.0, 0.5);
        });

        expect(mock_update_security_quota).toHaveBeenCalledWith('agent-1', 1.5);
    });
});
