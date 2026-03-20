/**
 * @file AgentManager.test.tsx
 * @description Suite for the Agent Swarm Manager page.
 * @module Pages/AgentManager
 * @testedBehavior
 * - Agent Inventory: Proper rendering of the managed agents list.
 * - Configuration Access: Opening and closing the AgentConfigPanel for existing agents.
 * - Search & Filter: Dynamic filtering of agent cards based on text input.
 * @aiContext
 * - Mocks useAgentStore and useWorkspaceStore to provide a controlled list of agents.
 * - Mocks AgentConfigPanel to simplify integration testing of the manager layout.
 */
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import AgentManager from './AgentManager';
import { useAgentStore } from '../stores/agentStore';
import { useWorkspaceStore } from '../stores/workspaceStore';

// Mock stores
vi.mock('../stores/agentStore', () => ({
    useAgentStore: vi.fn(),
}));

vi.mock('../stores/workspaceStore', () => ({
    useWorkspaceStore: vi.fn(),
}));

vi.mock('../stores/settingsStore', () => ({
    getSettings: vi.fn(() => ({
        defaultModel: 'gpt-4',
        defaultTemperature: 0.7
    }))
}));

vi.mock('../stores/roleStore', () => ({
    useRoleStore: vi.fn((selector) => selector({ roles: {} })),
}));

// Mock child components
vi.mock('../components/AgentConfigPanel', () => ({
    default: ({ agent, onClose, onUpdate, isNew }: any) => (
        <div data-testid="config-panel">
            {isNew ? 'Creating' : 'Editing'} {agent?.name}
            <button onClick={() => onUpdate(agent.id, { ...agent, name: 'Updated name' })}>Save</button>
            <button onClick={onClose}>Close</button>
        </div>
    )
}));

describe('AgentManager Page', () => {
    const mockAgents = [
        { id: 'agent-1', name: 'Test Agent 1', role: 'Worker', status: 'idle', model: 'gpt-4', themeColor: '#000', modelConfig: { temperature: 0.7 } },
        { id: 'agent-2', name: 'Test Agent 2', role: 'Boss', status: 'active', model: 'gpt-4', themeColor: '#000', modelConfig: { temperature: 0.7 } }
    ];

    const mockFetchAgents = vi.fn();
    const mockUpdateAgent = vi.fn();
    const mockAddAgent = vi.fn();
    
    beforeEach(() => {
        vi.clearAllMocks();
        (useAgentStore as any).mockReturnValue({
            agents: mockAgents,
            fetchAgents: mockFetchAgents,
            updateAgent: mockUpdateAgent,
            addAgent: mockAddAgent,
            isLoading: false,
        });
        (useWorkspaceStore as any).mockReturnValue({
            clusters: [{ id: '1', name: 'Cluster 1' }],
        });
    });

    it('renders agent list', () => {
        render(<AgentManager />);
        expect(screen.getByText(/Agent Swarm Manager/i)).toBeInTheDocument();
        expect(screen.getByText('Test Agent 1')).toBeInTheDocument();
        expect(screen.getByText('Test Agent 2')).toBeInTheDocument();
    });

    it('opens and closes config panel', async () => {
        render(<AgentManager />);
        
        // Click on the agent card container
        fireEvent.click(screen.getByText('Test Agent 1').closest('div')!);
        
        expect(screen.getByTestId('config-panel')).toBeInTheDocument();
        expect(screen.getByText(/Editing Test Agent 1/i)).toBeInTheDocument();
        
        fireEvent.click(screen.getByText('Close'));
        expect(screen.queryByTestId('config-panel')).not.toBeInTheDocument();
    });

    it('handles agent update', async () => {
        render(<AgentManager />);
        
        fireEvent.click(screen.getByText('Test Agent 1').closest('div')!);
        fireEvent.click(screen.getByText('Save'));
        
        expect(mockUpdateAgent).toHaveBeenCalled();
    });

    it('filters agents by name', () => {
        render(<AgentManager />);
        
        const searchInput = screen.getByPlaceholderText(/Search agents/i);
        fireEvent.change(searchInput, { target: { value: 'Test Agent 2' } });
        
        expect(screen.queryByText('Test Agent 1')).not.toBeInTheDocument();
        expect(screen.getByText('Test Agent 2')).toBeInTheDocument();
    });
});
