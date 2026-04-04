import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import Agent_Manager from './Agent_Manager';
import { use_agent_store } from '../stores/agent_store';
import { use_role_store } from '../stores/role_store';
import { getSettings } from '../stores/settings_store';

// Mock dependencies
vi.mock('../stores/agent_store', () => ({
    use_agent_store: vi.fn(),
}));

vi.mock('../stores/role_store', () => ({
    use_role_store: vi.fn(),
}));

vi.mock('../stores/settings_store', () => ({
    getSettings: vi.fn(),
}));

vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    }
}));

// Mock Agent_Config_Panel to simplify Agent_Manager testing
vi.mock('../components/Agent_Config_Panel', () => ({
    default: ({ onClose, onUpdate, agent }: any) => (
        <div data-testid="agent-config-panel">
            <span>Configuring {agent.name}</span>
            <button onClick={onClose}>Close</button>
            <button onClick={() => onUpdate(agent.id, { name: 'Updated' })}>Update</button>
        </div>
    )
}));

describe('Agent_Manager', () => {
    const mockAgents = [
        { id: '1', name: 'Agent 1', role: 'CEO', status: 'active', model: 'gpt-4o', skills: [], workflows: [], category: 'user' },
        { id: '2', name: 'Agent 2', role: 'Dev', status: 'idle', model: 'claude-3', skills: [], workflows: [], category: 'user' },
    ];

    beforeEach(() => {
        vi.mocked(use_agent_store).mockReturnValue({
            agents: mockAgents,
            fetchAgents: vi.fn(),
            update_agent: vi.fn(),
            addAgent: vi.fn(),
        } as any);

        vi.mocked(use_role_store).mockReturnValue({
            roles: {},
        } as any);

        vi.mocked(getSettings).mockReturnValue({
            defaultModel: 'gpt-4o',
            defaultTemperature: 0.7
        } as any);
    });

    it('renders agent list and search bar', () => {
        render(<Agent_Manager />);
        expect(screen.getByText('agent_manager.title')).toBeDefined();
        expect(screen.getByText('Agent 1')).toBeDefined();
        expect(screen.getByText('Agent 2')).toBeDefined();
    });

    it('filters agents by search query', () => {
        render(<Agent_Manager />);
        const searchInput = screen.getByPlaceholderText('agent_manager.search_placeholder');
        
        fireEvent.change(searchInput, { target: { value: 'Agent 1' } });
        
        expect(screen.getByText('Agent 1')).toBeDefined();
        expect(screen.queryByText('Agent 2')).toBeNull();
    });

    it('opens and closes config panel on agent click', () => {
        render(<Agent_Manager />);
        const agentCard = screen.getByText('Agent 1');
        
        fireEvent.click(agentCard);
        expect(screen.getByTestId('agent-config-panel')).toBeDefined();
        
        fireEvent.click(screen.getByText('Close'));
        expect(screen.queryByTestId('agent-config-panel')).toBeNull();
    });

    it('calls addAgent when creating a new agent', () => {
        const addAgent = vi.fn();
        vi.mocked(use_agent_store).mockReturnValue({
            agents: [],
            fetchAgents: vi.fn(),
            addAgent,
        } as any);

        render(<Agent_Manager />);
        fireEvent.click(screen.getByText('agent_manager.btn_add'));
        
        // Config panel should be open for a new agent
        expect(screen.getByTestId('agent-config-panel')).toBeDefined();
        
        // Trigger update in creation mode calls addAgent
        fireEvent.click(screen.getByText('Update'));
        expect(addAgent).toHaveBeenCalled();
    });

    it('enforces agent limit', () => {
        // Mock 25 agents
        const manyAgents = Array.from({ length: 25 }, (_, i) => ({ id: `${i}`, name: `Agent ${i}`, role: 'Dev' }));
        vi.mocked(use_agent_store).mockReturnValue({
            agents: manyAgents,
            fetchAgents: vi.fn(),
        } as any);

        window.alert = vi.fn();
        render(<Agent_Manager />);
        
        fireEvent.click(screen.getByText('agent_manager.btn_add'));
        expect(window.alert).toHaveBeenCalledWith('agent_manager.error_limit_reached');
    });
});
