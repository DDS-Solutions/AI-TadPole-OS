/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Oversight / Action_Ledger.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Action_Ledger } from './Action_Ledger';
import type { LedgerEntry } from '../../data/mock_oversight';

describe('Action_Ledger Component', () => {
    const mock_entries: LedgerEntry[] = [
        {
            id: 'led-1',
            agent_id: 'agent-alpha',
            skill: 'bash',
            decision: 'approved',
            timestamp: '2026-08-15T12:00:00Z',
            params: { command: 'ls -la' },
            risk_level: 'low',
            reason: 'Directory listing',
            auto_approved: false
        } as any
    ];

    it('renders empty table state when ledger is empty', () => {
        render(
            <Action_Ledger
                ledger={[]}
                filter=""
                set_filter={vi.fn()}
                selected_cluster_id="all"
                set_selected_cluster_id={vi.fn()}
                clusters={[]}
                resolve_agent_name={(id) => id}
            />
        );

        expect(screen.getByRole('table')).toBeDefined();
    });

    it('renders ledger entries and responds to filter input', () => {
        const set_filter_mock = vi.fn();
        render(
            <Action_Ledger
                ledger={mock_entries}
                filter=""
                set_filter={set_filter_mock}
                selected_cluster_id="all"
                set_selected_cluster_id={vi.fn()}
                clusters={[]}
                resolve_agent_name={() => 'Alpha Agent'}
            />
        );

        expect(screen.getByText('Alpha Agent')).toBeDefined();
        expect(screen.getByText('bash')).toBeDefined();

        const input = screen.getByPlaceholderText('Filter actions...');
        fireEvent.change(input, { target: { value: 'bash' } });
        expect(set_filter_mock).toHaveBeenCalledWith('bash');
    });
});
