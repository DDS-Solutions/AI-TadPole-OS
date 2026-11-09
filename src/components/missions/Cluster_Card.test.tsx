/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / Cluster_Card.test
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
import { Cluster_Card } from './Cluster_Card';
import type { Mission_Cluster } from '../../stores/workspace_store';

describe('Cluster_Card Component', () => {
    const mock_on_select = vi.fn();
    const mock_on_delete = vi.fn();
    const mock_on_toggle_active = vi.fn();
    const mock_on_update_department = vi.fn();
    const mock_on_update_budget = vi.fn();

    const sample_cluster: Mission_Cluster = {
        id: 'c-test-1',
        name: 'Alpha Ops',
        department: 'Engineering',
        path: '/workspaces/ops',
        collaborators: [],
        theme: 'blue',
        pending_tasks: [],
        budget_usd: 50
    };

    it('renders cluster name and triggers selection on click', () => {
        render(
            <Cluster_Card
                cluster={sample_cluster}
                is_selected={false}
                agent_map={new Map()}
                departments={['Engineering', 'Operations']}
                on_select_cluster={mock_on_select}
                on_delete_cluster={mock_on_delete}
                on_toggle_active={mock_on_toggle_active}
                on_update_department={mock_on_update_department}
                on_update_budget={mock_on_update_budget}
            />
        );

        expect(screen.getByText('Alpha Ops')).toBeInTheDocument();
        fireEvent.click(screen.getByRole('button', { name: /Alpha Ops/i }));
        expect(mock_on_select).toHaveBeenCalledWith('c-test-1');
    });

    it('allows updating department', () => {
        render(
            <Cluster_Card
                cluster={sample_cluster}
                is_selected={true}
                agent_map={new Map()}
                departments={['Engineering', 'Operations']}
                on_select_cluster={mock_on_select}
                on_delete_cluster={mock_on_delete}
                on_toggle_active={mock_on_toggle_active}
                on_update_department={mock_on_update_department}
                on_update_budget={mock_on_update_budget}
            />
        );

        const select = screen.getByLabelText('Reassign Cluster Department');
        fireEvent.change(select, { target: { value: 'Operations' } });
        expect(mock_on_update_department).toHaveBeenCalledWith('c-test-1', 'Operations');
    });
});

