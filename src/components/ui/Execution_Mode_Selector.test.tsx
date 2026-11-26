/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Ui / Execution_Mode_Selector.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Verifies Chat, Plan, and Swarm mode toggling and Brainstorm -> Swarm handoff button state.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import '@testing-library/jest-dom';
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { Execution_Mode_Selector } from './Execution_Mode_Selector';

describe('Execution_Mode_Selector', () => {
    it('renders Chat, Plan, and Swarm buttons with accurate aria-pressed state', () => {
        const on_change = vi.fn();
        render(<Execution_Mode_Selector mode="plan" on_change={on_change} />);

        expect(screen.getByRole('group', { name: 'Execution mode' })).toBeInTheDocument();
        expect(screen.getByRole('button', { name: 'Chat mode' })).toHaveAttribute('aria-pressed', 'false');
        expect(screen.getByRole('button', { name: 'Plan mode' })).toHaveAttribute('aria-pressed', 'true');
        expect(screen.getByRole('button', { name: 'Swarm mode' })).toHaveAttribute('aria-pressed', 'false');

        fireEvent.click(screen.getByRole('button', { name: 'Chat mode' }));
        expect(on_change).toHaveBeenCalledWith('chat');

        fireEvent.click(screen.getByRole('button', { name: 'Swarm mode' }));
        expect(on_change).toHaveBeenCalledWith('swarm');
    });

    it('shows Launch Swarm handoff button only outside Swarm mode when brainstorm context exists', () => {
        const on_change = vi.fn();
        const on_launch_swarm = vi.fn();

        const { rerender } = render(
            <Execution_Mode_Selector
                mode="chat"
                on_change={on_change}
                can_launch_swarm={true}
                on_launch_swarm={on_launch_swarm}
            />
        );

        const launch_btn = screen.getByRole('button', { name: 'Launch Swarm Mission from brainstorm' });
        expect(launch_btn).toBeInTheDocument();
        fireEvent.click(launch_btn);
        expect(on_launch_swarm).toHaveBeenCalledTimes(1);

        rerender(
            <Execution_Mode_Selector
                mode="swarm"
                on_change={on_change}
                can_launch_swarm={true}
                on_launch_swarm={on_launch_swarm}
            />
        );
        expect(screen.queryByRole('button', { name: 'Launch Swarm Mission from brainstorm' })).not.toBeInTheDocument();

        rerender(
            <Execution_Mode_Selector
                mode="plan"
                on_change={on_change}
                can_launch_swarm={false}
                on_launch_swarm={on_launch_swarm}
            />
        );
        expect(screen.queryByRole('button', { name: 'Launch Swarm Mission from brainstorm' })).not.toBeInTheDocument();
    });
});
