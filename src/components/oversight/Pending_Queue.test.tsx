/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Oversight / Pending_Queue.test
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
import { Pending_Queue } from './Pending_Queue';
import type { OversightEntry } from '../../data/mock_oversight';

describe('Pending_Queue Component', () => {
    const mock_pending: OversightEntry[] = [
        {
            id: 'pending-1',
            agent_id: 'agent-scout',
            created_at: '2026-08-15T12:00:00Z',
            tool_call: {
                agent_id: 'agent-scout',
                skill: 'bash_exec',
                description: 'Run deployment script',
                params: { cmd: 'cargo build' }
            }
        } as any
    ];

    it('renders nothing when pending list is empty', () => {
        const { container } = render(
            <Pending_Queue
                pending={[]}
                resolve_agent_name={(id) => id}
                handle_decide={vi.fn()}
            />
        );

        expect(container.firstChild).toBeNull();
    });

    it('renders pending items and triggers approve/reject actions', () => {
        const handle_decide_mock = vi.fn();
        render(
            <Pending_Queue
                pending={mock_pending}
                resolve_agent_name={() => 'Scout Agent'}
                handle_decide={handle_decide_mock}
            />
        );

        expect(screen.getByText('Scout Agent')).toBeDefined();
        expect(screen.getByText('bash_exec')).toBeDefined();
        expect(screen.getByText('Run deployment script')).toBeDefined();

        const buttons = screen.getAllByRole('button');
        expect(buttons.length).toBeGreaterThan(0);

        // Click first button (approve)
        fireEvent.click(buttons[0]);
        expect(handle_decide_mock).toHaveBeenCalledWith('pending-1', 'approved');
    });
});
