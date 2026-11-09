/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Assist Note
 * - Review changes preserve origin security and propagate confirmed operation outcomes.
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Oversight / Oversight_Stats.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { Oversight_Stats } from './Oversight_Stats';

describe('Oversight_Stats Component', () => {
    const mock_stats = {
        pending: 4,
        approved: 12,
        rejected: 2
    };

    it.each([[true, true, false, false], [false, true, false, true], [true, null, false, true], [true, false, false, true], [true, true, true, true]])('gates resume on online, authoritative latch, and pending state (%s,%s,%s)', (online, paused, pending, disabled) => {
        const resume = vi.fn();
        render(<Oversight_Stats stats={mock_stats} is_online={online as boolean} dispatch_paused={paused as boolean | null} engine_control_pending={pending as boolean} handle_resume_engine={resume} handle_kill_switch={vi.fn()} handle_kill_engine={vi.fn()} on_navigate_security={vi.fn()} />);
        const button = screen.getByRole('button', { name: 'Resume engine' }) as HTMLButtonElement;
        expect(button.disabled).toBe(disabled);
        fireEvent.click(button);
        expect(resume).toHaveBeenCalledTimes(disabled ? 0 : 1);
    });

    it('renders pending, approved, and rejected counters', () => {
        render(
            <Oversight_Stats
                stats={mock_stats}
                is_online={true}
                handle_kill_switch={vi.fn()}
                handle_kill_engine={vi.fn()}
                on_navigate_security={vi.fn()}
            />
        );

        expect(screen.getByText('4')).toBeDefined();
        expect(screen.getByText('12')).toBeDefined();
        expect(screen.getByText('2')).toBeDefined();
    });

    it('triggers security navigation handler on click', () => {
        const nav_mock = vi.fn();
        render(
            <Oversight_Stats
                stats={mock_stats}
                is_online={true}
                handle_kill_switch={vi.fn()}
                handle_kill_engine={vi.fn()}
                on_navigate_security={nav_mock}
            />
        );

        const buttons = screen.getAllByRole('button');
        const sec_btn = buttons.find(b => b.textContent?.includes('Security') || b.querySelector('svg'));
        expect(sec_btn).toBeDefined();
        fireEvent.click(sec_btn!);
    });

    it('renders tooltip for resume engine button', () => {
        vi.useFakeTimers();
        render(
            <Oversight_Stats
                stats={mock_stats}
                is_online={true}
                dispatch_paused={true}
                handle_resume_engine={vi.fn()}
                handle_kill_switch={vi.fn()}
                handle_kill_engine={vi.fn()}
                on_navigate_security={vi.fn()}
            />
        );
        const button = screen.getByRole('button', { name: 'Resume engine' });
        const trigger = button.parentElement!;
        act(() => {
            fireEvent.mouseEnter(trigger);
            vi.advanceTimersByTime(350);
        });
        expect(screen.getByRole('tooltip')).toBeDefined();
        expect(screen.getByText(/Resume engine dispatch/i)).toBeDefined();
        vi.useRealTimers();
    });
});
