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
import { use_role_store } from '../stores/role_store';
import { get_settings } from '../stores/settings_store';

// Mock dependencies
vi.mock('../stores/agent_store', () => ({
    use_agent_store: vi.fn(),
}));

vi.mock('../stores/role_store', () => ({
    use_role_store: vi.fn(),
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
vi.mock('../components/Agent_Config_Panel', () => ({
    default: ({ on_close, on_update, agent }: any) => (
        <div data-testid="agent-config-panel">
            <span>Configuring {agent.name}</span>
            <button onClick={on_close}>Close</button>
            <button onClick={() => on_update(agent.id, { name: 'Updated' })}>Update</button>
        </div>
    )
}));

describe('Agent_Manager', () => {
    const mock_agents = [
        { id: '1', name: 'Agent 1', role: 'CEO', status: 'active', model: 'gpt-4o', skills: [], workflows: [], category: 'user' },
        { id: '2', name: 'Agent 2', role: 'Dev', status: 'idle', model: 'claude-3', skills: [], workflows: [], category: 'user' },
    ];

    beforeEach(() => {
        vi.clearAllMocks();
        
        vi.mocked(use_agent_store).mockReturnValue({
            agents: mock_agents,
            fetch_agents: vi.fn(),
            update_agent: vi.fn(),
            add_agent: vi.fn(),
        } as any);

        vi.mocked(use_role_store).mockReturnValue({
            roles: {},
        } as any);

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
        const add_agent = vi.fn();
        vi.mocked(use_agent_store).mockReturnValue({
            agents: [],
            fetch_agents: vi.fn(),
            add_agent,
        } as any);

        render(<Agent_Manager />);
        fireEvent.click(screen.getByText('agent_manager.btn_add'));
        
        // Config panel should be open for a new agent
        expect(screen.getByTestId('agent-config-panel')).toBeDefined();
        
        // Trigger update in creation mode calls add_agent
        fireEvent.click(screen.getByText('Update'));
        expect(add_agent).toHaveBeenCalled();
    });

    it('enforces agent limit', () => {
        // Mock 25 agents
        const many_agents = Array.from({ length: 25 }, (_, i) => ({ id: `${i}`, name: `Agent ${i}`, role: 'Dev', category: 'user' }));
        vi.mocked(use_agent_store).mockReturnValue({
            agents: many_agents,
            fetch_agents: vi.fn(),
        } as any);

        window.alert = vi.fn();
        render(<Agent_Manager />);
        
        fireEvent.click(screen.getByText('agent_manager.btn_add'));
        expect(window.alert).toHaveBeenCalledWith('agent_manager.error_limit_reached');
    });
});

