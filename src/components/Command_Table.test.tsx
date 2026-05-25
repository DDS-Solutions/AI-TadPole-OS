/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Command_Table_test]` in observability traces.
 */

import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Command_Table } from './Command_Table';
import { use_agent_store } from '../stores/agent_store';
import { use_sovereign_store } from '../stores/sovereign_store';
import { useEngineStatus } from '../hooks/use_engine_status';

// Mock dependencies
vi.mock('../stores/agent_store', () => ({
    use_agent_store: vi.fn(),
}));

vi.mock('../stores/sovereign_store', () => ({
    use_sovereign_store: vi.fn(),
}));

vi.mock('../hooks/use_engine_status', () => ({
    useEngineStatus: vi.fn(),
}));

vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string, options?: any) => {
            if (key === 'command.nodes_online') return `Nodes Online: ${options?.count}`;
            if (key === 'command.version') return `Version: ${options?.ver}`;
            if (key === 'command.latency') return `Latency: ${options?.val}`;
            if (key === 'common_units.ms') return 'ms';
            return key;
        },
    }
}));

describe('Command_Table Component', () => {
    const mock_agents = [
        { id: '1', name: 'Agent Alpha', status: 'active', cost_usd: 0.05, budget_usd: 1.0, tokens_used: 1000, theme_color: '#00ff00', department: 'RECON' },
        { id: '2', name: 'Agent Beta', status: 'idle', cost_usd: 1.2, budget_usd: 1.0, tokens_used: 5000, theme_color: '#ff0000', department: 'INTEL' },
    ];

    const mock_engine = {
        latency: 0.42,
        active_agents: 2,
    };

    const mock_sovereign = {
        set_scope: vi.fn(),
        set_detached: vi.fn(),
        set_selected_agent_id: vi.fn(),
    };

    beforeEach(() => {
        vi.clearAllMocks();
        (use_agent_store as any).mockReturnValue({ agents: mock_agents });
        (use_sovereign_store as any).mockReturnValue(mock_sovereign);
        (useEngineStatus as any).mockReturnValue(mock_engine);
    });

    it('renders agent table with telemetry data', () => {
        render(<Command_Table />);
        
        expect(screen.getByText('Agent Alpha')).toBeInTheDocument();
        expect(screen.getByText('Agent Beta')).toBeInTheDocument();
        expect(screen.getByText('1,000')).toBeInTheDocument();
        expect(screen.getByText('5,000')).toBeInTheDocument();
    });

    it('displays live engine status in the footer', () => {
        render(<Command_Table />);
        
        expect(screen.getByText('Nodes Online: 2')).toBeInTheDocument();
        expect(screen.getByText('Latency: 0.42ms')).toBeInTheDocument();
        expect(screen.getByText('Version: 1.1.70')).toBeInTheDocument();
    });

    it('calculates utilization and handles budget overflow', () => {
        render(<Command_Table />);
        
        // Agent Alpha: 0.05 / 1.0 = 5%
        expect(screen.getByText('5.0%')).toBeInTheDocument();
        
        // Agent Beta: 1.2 / 1.0 = 120%
        expect(screen.getByText('120.0%')).toBeInTheDocument();
    });

    it('triggers join_chat when clicking message button', () => {
        render(<Command_Table />);
        
        const join_buttons = screen.getAllByRole('button');
        fireEvent.click(join_buttons[0]);
        
        expect(mock_sovereign.set_selected_agent_id).toHaveBeenCalledWith('2');
        expect(mock_sovereign.set_scope).toHaveBeenCalledWith('agent');
        expect(mock_sovereign.set_detached).toHaveBeenCalledWith(false);
    });
});

// Metadata: [Command_Table_test]
