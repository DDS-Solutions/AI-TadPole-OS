/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / Cluster_Sidebar.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Cluster_Sidebar } from './Cluster_Sidebar';
import { use_department_store } from '../../stores/department_store';
import type { Mission_Cluster } from '../../stores/workspace_store';

describe('Cluster_Sidebar Component', () => {
    const mock_on_select_cluster = vi.fn();
    const mock_on_create_cluster = vi.fn();
    const mock_on_delete_cluster = vi.fn();
    const mock_on_toggle_active = vi.fn();
    const mock_on_update_department = vi.fn();
    const mock_on_update_budget = vi.fn();

    const mock_cluster: Mission_Cluster = {
        id: 'cluster-1',
        name: 'Alpha Swarm Cluster',
        department: 'Engineering',
        path: '/workspaces/alpha',
        collaborators: ['agent-1'],
        theme: 'blue',
        pending_tasks: [],
        budget_usd: 100
    };

    beforeEach(() => {
        vi.clearAllMocks();
        use_department_store.setState({
            departments: ['Engineering', 'Executive', 'Operations', 'Swarm Core']
        });
    });

    it('renders cluster card with department options from use_department_store including Swarm Core', () => {
        render(
            <Cluster_Sidebar
                clusters={[mock_cluster]}
                selected_cluster_id="cluster-1"
                agents={[]}
                on_select_cluster={mock_on_select_cluster}
                on_create_cluster={mock_on_create_cluster}
                on_delete_cluster={mock_on_delete_cluster}
                on_toggle_active={mock_on_toggle_active}
                on_update_department={mock_on_update_department}
                on_update_budget={mock_on_update_budget}
            />
        );

        const dept_select = screen.getByLabelText('Reassign Cluster Department');
        expect(dept_select).toBeInTheDocument();
        expect(dept_select).toHaveValue('Engineering');

        expect(screen.getByRole('option', { name: 'Engineering' })).toBeInTheDocument();
        expect(screen.getByRole('option', { name: 'Swarm Core' })).toBeInTheDocument();
    });

    it('triggers on_update_department when selecting Swarm Core', () => {
        render(
            <Cluster_Sidebar
                clusters={[mock_cluster]}
                selected_cluster_id="cluster-1"
                agents={[]}
                on_select_cluster={mock_on_select_cluster}
                on_create_cluster={mock_on_create_cluster}
                on_delete_cluster={mock_on_delete_cluster}
                on_toggle_active={mock_on_toggle_active}
                on_update_department={mock_on_update_department}
                on_update_budget={mock_on_update_budget}
            />
        );

        const dept_select = screen.getByLabelText('Reassign Cluster Department');
        fireEvent.change(dept_select, { target: { value: 'Swarm Core' } });

        expect(mock_on_update_department).toHaveBeenCalledWith('cluster-1', 'Swarm Core');
    });
});
