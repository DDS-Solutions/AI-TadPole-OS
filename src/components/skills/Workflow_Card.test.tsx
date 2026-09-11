/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Skills / Workflow_Card.test
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
import { Workflow_Card } from './Workflow_Card';
import type { Workflow_Definition } from '../../stores/skill_store';

describe('Workflow_Card Component', () => {
    const mock_workflow: Workflow_Definition = {
        name: 'ContinuousDeploy',
        content: '# Deployment Plan\n1. Run tests\n2. Build bundle\n3. Deploy',
        category: 'user'
    };

    it('renders workflow name and formatted content steps', () => {
        render(
            <Workflow_Card
                workflow={mock_workflow}
                on_edit={vi.fn()}
                on_assign={vi.fn()}
                on_delete={vi.fn()}
            />
        );

        expect(screen.getByText('ContinuousDeploy')).toBeDefined();
        expect(screen.getByText(/# Deployment Plan/)).toBeDefined();
    });

    it('triggers assign and edit handlers', () => {
        const on_edit_mock = vi.fn();
        const on_assign_mock = vi.fn();

        render(
            <Workflow_Card
                workflow={mock_workflow}
                on_edit={on_edit_mock}
                on_assign={on_assign_mock}
                on_delete={vi.fn()}
            />
        );

        const buttons = screen.getAllByRole('button');
        fireEvent.click(buttons[0]); // edit
        expect(on_edit_mock).toHaveBeenCalledWith(mock_workflow);

        fireEvent.click(buttons[1]); // assign
        expect(on_assign_mock).toHaveBeenCalledWith('ContinuousDeploy');
    });
});
