/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / Cluster_Create_Drawer.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Cluster_Create_Drawer } from './Cluster_Create_Drawer';

describe('Cluster_Create_Drawer Component', () => {
    const mock_on_close = vi.fn();
    const mock_on_create = vi.fn();

    it('does not render when isOpen is false', () => {
        const { container } = render(
            <Cluster_Create_Drawer
                isOpen={false}
                onClose={mock_on_close}
                onCreate={mock_on_create}
                departments={['Engineering']}
                team_presets={[]}
                agents={[]}
            />
        );
        expect(container).toBeEmptyDOMElement();
    });

    it('renders input and calls onCreate when submit is clicked', () => {
        render(
            <Cluster_Create_Drawer
                isOpen={true}
                onClose={mock_on_close}
                onCreate={mock_on_create}
                departments={['Engineering']}
                team_presets={[]}
                agents={[]}
            />
        );

        const name_input = screen.getByLabelText('Mission Name...');
        fireEvent.change(name_input, { target: { value: 'New Test Cluster' } });

        const submit_button = screen.getByRole('button', { name: /create/i });
        fireEvent.click(submit_button);

        expect(mock_on_create).toHaveBeenCalledWith(
            expect.objectContaining({
                name: 'New Test Cluster',
                department: 'Engineering',
                budget_usd: 1
            })
        );
        expect(mock_on_close).toHaveBeenCalled();
    });
});
