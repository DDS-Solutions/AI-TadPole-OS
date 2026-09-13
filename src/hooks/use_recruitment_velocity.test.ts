/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / use_recruitment_velocity.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useRecruitmentVelocity } from './use_recruitment_velocity';

describe('useRecruitmentVelocity', () => {
    it('calculates recruited agent count in the last 24 hours', () => {
        const now = Date.now();
        const mock_agents = [
            { id: '1', name: 'Recent Agent', created_at: new Date(now - 2 * 60 * 60 * 1000).toISOString() }, // 2 hours ago
            { id: '2', name: 'Older Agent', created_at: new Date(now - 30 * 60 * 60 * 1000).toISOString() }, // 30 hours ago (outside 24h)
            { id: '3', name: 'Brand New Agent', created_at: new Date(now - 5 * 60 * 1000).toISOString() },   // 5 mins ago
            { id: '4', name: 'No Timestamp' },
            { id: '5', name: 'Invalid Date', created_at: 'not-a-valid-date' }
        ] as any;

        const { result } = renderHook(() => useRecruitmentVelocity(mock_agents));
        expect(result.current).toBe(2);
    });

    it('handles empty agent array safely', () => {
        const { result } = renderHook(() => useRecruitmentVelocity([]));
        expect(result.current).toBe(0);
    });

    it('updates calculation over time intervals', () => {
        vi.useFakeTimers();
        const base_time = new Date('2026-09-09T12:00:00Z').getTime();
        vi.setSystemTime(base_time);

        // Agent created 23h 59m ago
        const mock_agents = [
            { id: '1', name: 'Border Agent', created_at: new Date(base_time - (23 * 60 + 59) * 60 * 1000).toISOString() }
        ] as any;

        const { result } = renderHook(() => useRecruitmentVelocity(mock_agents));
        expect(result.current).toBe(1);

        // Advance 2 minutes so the agent is now >24 hours old
        act(() => {
            vi.advanceTimersByTime(2 * 60 * 1000);
        });

        expect(result.current).toBe(0);
        vi.useRealTimers();
    });
});
