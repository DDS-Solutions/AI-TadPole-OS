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

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Standup_Meeting_Area } from './Standup_Meeting_Area';

describe('Standup_Meeting_Area Component', () => {
    it('renders meeting area and start sync button', () => {
        const toggle_live_mock = vi.fn();
        render(
            <Standup_Meeting_Area
                is_live={false}
                sync_error={null}
                can_start_sync={true}
                live_seconds={0}
                target_type="agent"
                selected_target_id="agent-1"
                agents={[{ id: 'agent-1', name: 'Scout Agent' } as any]}
                clusters={[]}
                set_target_type={vi.fn()}
                set_selected_target_id={vi.fn()}
                toggle_live={toggle_live_mock}
            />
        );

        const buttons = screen.getAllByRole('button');
        const start_btn = buttons[buttons.length - 2];
        expect(start_btn).toBeDefined();

        fireEvent.click(start_btn);
        expect(toggle_live_mock).toHaveBeenCalled();
    });

    it('renders live status and active visualizer state', () => {
        render(
            <Standup_Meeting_Area
                is_live={true}
                sync_error={null}
                can_start_sync={true}
                live_seconds={65}
                target_type="agent"
                selected_target_id="agent-1"
                agents={[{ id: 'agent-1', name: 'Scout Agent' } as any]}
                clusters={[]}
                set_target_type={vi.fn()}
                set_selected_target_id={vi.fn()}
                toggle_live={vi.fn()}
            />
        );

        expect(screen.getByText(/00:01:05/)).toBeDefined();
    });
});
