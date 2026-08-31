/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Skills / Hook_Card.test
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
import { Hook_Card } from './Hook_Card';
import type { Hook_Definition } from '../../stores/skill_store';

describe('Hook_Card Component', () => {
    const mock_hook: Hook_Definition = {
        name: 'PreCommitAudit',
        description: 'Verify system integrity before committing',
        hook_type: 'pre_validation',
        content: 'verify()',
        active: true,
        category: 'user'
    };

    it('renders hook identity, type, and active status', () => {
        render(
            <Hook_Card
                hook={mock_hook}
                on_edit={vi.fn()}
                on_delete={vi.fn()}
            />
        );

        expect(screen.getByText('PreCommitAudit')).toBeDefined();
        expect(screen.getByText('pre_validation')).toBeDefined();
    });

    it('triggers on_edit and on_delete callback handlers', () => {
        const on_edit_mock = vi.fn();
        const on_delete_mock = vi.fn();

        render(
            <Hook_Card
                hook={mock_hook}
                on_edit={on_edit_mock}
                on_delete={on_delete_mock}
            />
        );

        const buttons = screen.getAllByRole('button');
        fireEvent.click(buttons[0]); // Edit
        expect(on_edit_mock).toHaveBeenCalledWith(mock_hook);

        fireEvent.click(buttons[1]); // Delete
        expect(on_delete_mock).toHaveBeenCalledWith('PreCommitAudit');
    });
});
