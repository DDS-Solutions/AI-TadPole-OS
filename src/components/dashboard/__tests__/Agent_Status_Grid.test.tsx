/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Manages the interface for Agent Status Grid.test. 
 * Part of the Tadpole-OS Modular UI ecosystem.
 */

import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { Agent_Status_Grid } from '../Agent_Status_Grid';
import type { Agent, Mission_Cluster } from '../../../types';

vi.mock('../../../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    },
}));

// Mocking Hierarchy_Node to focus on filtering logic
vi.mock('../../Hierarchy_Node', () => ({
    Hierarchy_Node: ({ agent }: { agent: Agent }) => (
        <div data-testid={`agent-node-${agent.id}`}>
            {agent.name} - {agent.department}
        </div>
    )
}));

const mock_agents: Agent[] = [
    {
        id: '1',
        name: 'Alpha One',
        status: 'active',
        department: 'Executive' as any,
        role: 'Orchestrator',
        is_suspended: false,
        tokens_used: 0,
        model: 'gemini-2.0-flash',
        category: 'core'
    } as Agent,
    {
        id: '2',
        name: 'Beta Two',
        status: 'idle',
        department: 'Operations' as any,
        role: 'Researcher',
        is_suspended: false,
        tokens_used: 0,
        model: 'gemini-2.0-flash',
        category: 'core'
    } as Agent,
    {
        id: '3',
        name: 'Gamma Three',
        status: 'speaking',
        department: 'Creative' as any,
        role: 'Designer',
        is_suspended: false,
        tokens_used: 0,
        model: 'gemini-2.0-flash',
        category: 'core'
    } as Agent,
];

const mock_clusters: Mission_Cluster[] = [
    {
        id: 'cluster-1',
        name: 'Project Phoenix',
        collaborators: ['1', '2'],
        is_active: true,
        alpha_id: '1',
        department: 'Executive'
    } as Mission_Cluster
];

const default_props = {
    agents: mock_agents,
    assigned_agent_ids: new Set(['1', '2']),
    available_roles: ['Orchestrator', 'Researcher', 'Designer'],
    clusters: mock_clusters,
    on_skill_trigger: vi.fn(),
    on_configure_click: vi.fn(),
    on_model_change: vi.fn(),
    on_model_2_change: vi.fn(),
    on_model_3_change: vi.fn(),
    on_role_change: vi.fn(),
    handle_agent_update: vi.fn(),
    on_toggle_cluster: vi.fn(),
};

describe('Agent_Status_Grid', () => {
    it('renders global view by default and filters agents correctly', () => {
        render(<Agent_Status_Grid {...default_props} />);
        
        // Agent 1 (active), Agent 2 (assigned), and Agent 3 (speaking) should be visible
        expect(screen.getByTestId('agent-node-1')).toBeInTheDocument();
        expect(screen.getByTestId('agent-node-2')).toBeInTheDocument();
        expect(screen.getByTestId('agent-node-3')).toBeInTheDocument();
    });

    it('filters for dynamic Mission Clusters', () => {
        render(<Agent_Status_Grid {...default_props} />);
        
        // Select Cluster Tab
        const cluster_tab = screen.getByText('Project Phoenix');
        fireEvent.click(cluster_tab);

        // Agents 1 and 2 are collaborators, Agent 3 is not
        expect(screen.getByTestId('agent-node-1')).toBeInTheDocument();
        expect(screen.getByTestId('agent-node-2')).toBeInTheDocument();
        expect(screen.queryByTestId('agent-node-3')).not.toBeInTheDocument();
    });

    it('displays empty state when no agents match filter', () => {
        render(<Agent_Status_Grid {...default_props} agents={[]} />);
        expect(screen.getByText('No Neural Nodes Detected in this Sector')).toBeInTheDocument();
    });
});

