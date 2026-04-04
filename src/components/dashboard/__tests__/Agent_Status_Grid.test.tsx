import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { Agent_Status_Grid } from '../Agent_Status_Grid';
import type { Agent, MissionCluster } from '../../../types';

// Mocking Hierarchy_Node to focus on filtering logic
vi.mock('../../Hierarchy_Node', () => ({
    Hierarchy_Node: ({ agent }: { agent: Agent }) => (
        <div data-testid={`agent-node-${agent.id}`}>
            {agent.name} - {agent.department}
        </div>
    )
}));

const mockAgents: Agent[] = [
    {
        id: '1',
        name: 'Alpha One',
        status: 'active',
        department: 'Executive',
        role: 'Orchestrator',
        isSuspended: false,
        tokensUsed: 0,
        model: 'gemini-2.0-flash',
        category: 'core'
    } as Agent,
    {
        id: '2',
        name: 'Beta Two',
        status: 'idle',
        department: 'Operations',
        role: 'Researcher',
        isSuspended: false,
        tokensUsed: 0,
        model: 'gemini-2.0-flash',
        category: 'core'
    } as Agent,
    {
        id: '3',
        name: 'Gamma Three',
        status: 'speaking',
        department: 'Creative',
        role: 'Designer',
        isSuspended: false,
        tokensUsed: 0,
        model: 'gemini-2.0-flash',
        category: 'core'
    } as Agent,
];

const mockClusters: MissionCluster[] = [
    {
        id: 'cluster-1',
        name: 'Project Phoenix',
        collaborators: ['1', '2'],
        isActive: true,
        alphaId: '1',
        department: 'Executive'
    } as MissionCluster
];

const defaultProps = {
    agents: mockAgents,
    assignedAgentIds: new Set(['1', '2']),
    availableRoles: ['Orchestrator', 'Researcher', 'Designer'],
    clusters: mockClusters,
    onSkillTrigger: vi.fn(),
    onConfigureClick: vi.fn(),
    onModelChange: vi.fn(),
    onModel2Change: vi.fn(),
    onModel3Change: vi.fn(),
    onRoleChange: vi.fn(),
    handleAgentUpdate: vi.fn(),
    onToggleCluster: vi.fn(),
};

describe('Agent_Status_Grid', () => {
    it('renders global view by default and filters agents correctly', () => {
        render(<Agent_Status_Grid {...defaultProps} />);
        
        // Agent 1 (active), Agent 2 (assigned), and Agent 3 (speaking) should be visible
        expect(screen.getByTestId('agent-node-1')).toBeInTheDocument();
        expect(screen.getByTestId('agent-node-2')).toBeInTheDocument();
        expect(screen.getByTestId('agent-node-3')).toBeInTheDocument();
    });


    it('filters for dynamic Mission Clusters', () => {
        render(<Agent_Status_Grid {...defaultProps} />);
        
        // Select Cluster Tab
        const clusterTab = screen.getByText('Project Phoenix');
        fireEvent.click(clusterTab);

        // Agents 1 and 2 are collaborators, Agent 3 is not
        expect(screen.getByTestId('agent-node-1')).toBeInTheDocument();
        expect(screen.getByTestId('agent-node-2')).toBeInTheDocument();
        expect(screen.queryByTestId('agent-node-3')).not.toBeInTheDocument();
    });

    it('displays empty state when no agents match filter', () => {
        render(<Agent_Status_Grid {...defaultProps} agents={[]} />);
        expect(screen.getByText('No Neural Nodes Detected in this Sector')).toBeInTheDocument();
    });
});
