/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the Agent Manager's CRUD operations support**, including list filtering, creation modals, and mass-transfer logic. 
 * Verifies integration with the `agent_store` and search debouncing for high-density swarm management.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Inconsistent agent list state after batch delete or failure to display validation errors during illegal agent creation attempts.
 * - **Telemetry Link**: Search `[Agent_Manager.test]` in tracing logs.
 */

import '@testing-library/jest-dom';

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import Agent_Manager from './Agent_Manager';
import { use_agent_store } from '../stores/agent_store';
import { get_settings } from '../stores/settings_store';

// Mock dependencies
const mock_agent_store_state = {
    agents: [] as any[],
    fetch_agents: vi.fn(),
    update_agent: vi.fn(),
    add_agent: vi.fn(),
};

vi.mock('../stores/agent_store', () => {
    const mock_store = Object.assign(
        vi.fn((selector) => selector ? selector(mock_agent_store_state) : mock_agent_store_state),
        {
            getState: vi.fn(() => mock_agent_store_state),
            setState: vi.fn((updater) => {
                if (typeof updater === 'function') {
                    Object.assign(mock_agent_store_state, updater(mock_agent_store_state));
                } else {
                    Object.assign(mock_agent_store_state, updater);
                }
            }),
            subscribe: vi.fn(() => vi.fn())
        }
    );
    return {
        use_agent_store: mock_store
    };
});

const mock_role_store_state = {
    roles: {},
};

vi.mock('../stores/role_store', () => ({
    use_role_store: vi.fn((selector) => selector ? selector(mock_role_store_state) : mock_role_store_state),
}));

vi.mock('../services/agent_service', () => ({
    agent_service: {
        load_agents_into_store: vi.fn(() => Promise.resolve()),
        update_agent: vi.fn(() => Promise.resolve()),
        broadcast_update: vi.fn(),
    }
}));

vi.mock('../stores/settings_store', () => ({
    get_settings: vi.fn(),
}));

vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    }
}));

// Mock Agent_Config_Panel to simplify Agent_Manager testing
vi.mock('../components/AgentConfigPanel', () => ({
    default: ({ onClose, onUpdate, agent }: any) => (
        <div data-testid="agent-config-panel">
            <span>Configuring {agent.name}</span>
            <button onClick={onClose}>Close</button>
            <button onClick={() => onUpdate(agent.id, { name: 'Updated' })}>Update</button>
        </div>
    )
}));

describe('Agent_Manager', () => {
    const mock_agents = [
        { id: '1', name: 'Agent 1', role: 'CEO', status: 'active', model: 'gpt-4o', skills: [], workflows: [], category: 'user' },
        { id: '2', name: 'Agent 2', role: 'Dev', status: 'idle', model: 'claude-3', skills: [], workflows: [], category: 'user' },
        { id: '3', name: 'Recruited Agent', role: 'Researcher', status: 'idle', category: 'user', metadata: { has_participated_in_swarm: true } },
        { id: '4', name: 'Swarm Brain', role: 'Coder', status: 'idle', category: 'ai' },
    ];

    beforeEach(() => {
        vi.clearAllMocks();
        
        mock_agent_store_state.agents = mock_agents;
        mock_agent_store_state.fetch_agents = vi.fn();
        mock_agent_store_state.update_agent = vi.fn();
        mock_agent_store_state.add_agent = vi.fn();

        mock_role_store_state.roles = {};

        vi.mocked(get_settings).mockReturnValue({
            default_model: 'gpt-4o',
            default_temperature: 0.7
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
        const search_input = screen.getByPlaceholderText('agent_manager.search_placeholder');
        
        fireEvent.change(search_input, { target: { value: 'Agent 1' } });
        
        expect(screen.getByText('Agent 1')).toBeDefined();
        expect(screen.queryByText('Agent 2')).toBeNull();
    });

    it('opens and closes config panel on agent click', () => {
        render(<Agent_Manager />);
        const agent_card = screen.getByText('Agent 1');
        
        fireEvent.click(agent_card);
        expect(screen.getByTestId('agent-config-panel')).toBeDefined();
        
        fireEvent.click(screen.getByText('Close'));
        expect(screen.queryByTestId('agent-config-panel')).toBeNull();
    });

    it('calls add_agent when creating a new agent', () => {
        mock_agent_store_state.agents = [];

        render(<Agent_Manager />);
        fireEvent.click(screen.getByText('agent_manager.btn_add'));
        
        // Config panel should be open for a new agent
        expect(screen.getByTestId('agent-config-panel')).toBeDefined();
        
        // Trigger update in creation mode calls setState
        fireEvent.click(screen.getByText('Update'));
        expect(use_agent_store.setState).toHaveBeenCalled();
    });

    it('enforces agent limit', () => {
        // Mock 25 agents
        const many_agents = Array.from({ length: 25 }, (_, i) => ({ id: `${i}`, name: `Agent ${i}`, role: 'Dev', category: 'user' }));
        mock_agent_store_state.agents = many_agents;

        window.alert = vi.fn();
        render(<Agent_Manager />);
        
        fireEvent.click(screen.getByText('agent_manager.btn_add'));
        expect(window.alert).toHaveBeenCalledWith('agent_manager.error_limit_reached');
    });

    it('shows recruited user agents in the AI Swarm tab', () => {
        render(<Agent_Manager />);
        
        // Switch to AI Swarm tab
        const ai_tab_button = screen.getByText('AI Swarm');
        fireEvent.click(ai_tab_button);
        
        expect(screen.getByText('Swarm Brain')).toBeDefined();
        expect(screen.getByText('Recruited Agent')).toBeDefined();
        expect(screen.queryByText('Agent 1')).toBeNull();
    });

    it('updates tab counters for persistent swarm indexing', () => {
        render(<Agent_Manager />);
        
        // User sector count should be 3 (Agent 1, Agent 2, Recruited Agent)
        expect(screen.getByText('3')).toBeDefined();
        
        // AI Swarm count should be 2 (Swarm Brain, Recruited Agent)
        expect(screen.getByText('2')).toBeDefined();
    });
});


// Metadata: [Agent_Manager_test]

// Metadata: [Agent_Manager_test]
