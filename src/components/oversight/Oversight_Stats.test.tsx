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
import { Oversight_Stats } from './Oversight_Stats';

describe('Oversight_Stats Component', () => {
    const mock_stats = {
        pending: 4,
        approved: 12,
        rejected: 2
    };

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
});
