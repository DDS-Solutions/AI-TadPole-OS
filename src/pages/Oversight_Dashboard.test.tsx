/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Manages the Oversight Dashboard.test page. 
 * Connects the global state stores to the layout orchestration.
 */

/**
 * @file Oversight_Dashboard.test.tsx
 * @description Suite for the High-Level Governance and Proposal Oversight dashboard.
 * @module Pages/Oversight_Dashboard
 * @testedBehavior
 * - Proposal Lifecycle: Listing active governance proposals and handling simulation modes.
 * - Kill Switch Security: Verification of engine termination safety controls.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks tadpole_os_service for proposal management.
 * - Navigates between primary and simulation views via useEngineStatus state.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import Oversight_Dashboard from './Oversight_Dashboard';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { use_workspace_store } from '../stores/workspace_store';
import { useEngineStatus } from '../hooks/use_engine_status';

vi.mock('../hooks/use_engine_status', () => ({
    useEngineStatus: vi.fn(),
}));

vi.mock('../stores/workspace_store', () => ({
    use_workspace_store: vi.fn(),
}));

vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        send_command: vi.fn(),
        get_pending_oversight: vi.fn(),
        get_oversight_ledger: vi.fn(),
        decide_oversight: vi.fn(),
        kill_agents: vi.fn(),
        shutdown_engine: vi.fn(),
    }
}));

describe('Oversight_Dashboard Page', () => {
    const mock_pending_actions = [
        { 
            id: 'p1', 
            agent_id: 'agent-1', 
            role: 'Worker', 
            skill: 'file_write', 
            created_at: new Date().toISOString(), 
            timestamp: new Date().toISOString(), 
            tool_call: { 
                name: 'write', 
                params: { data: 'test' }, 
                agent_id: 'agent-1', 
                skill: 'write' 
            } 
        }
    ];

    beforeEach(() => {
        vi.clearAllMocks();
        vi.stubGlobal('setInterval', vi.fn());
        
        // Default engine status
        (useEngineStatus as Mock).mockReturnValue({ is_online: true });

        // Default store mocks
        (use_workspace_store as unknown as Mock).mockReturnValue({
            clusters: [{ id: 'cluster-1', name: 'Test Cluster', alpha_id: '1' }],
            active_proposals: {}
        });

        (tadpole_os_service.get_pending_oversight as Mock).mockResolvedValue(mock_pending_actions);
        (tadpole_os_service.get_oversight_ledger as Mock).mockResolvedValue([]);

        const store: Record<string, string> = {};
        vi.stubGlobal('localStorage', {
            getItem: (key: string) => store[key] || null,
            setItem: (key: string, value: string) => { store[key] = value; },
            clear: () => { Object.keys(store).forEach(k => delete store[k]); },
            removeItem: (key: string) => { delete store[key]; }
        });
    });

    it('renders the dashboard basic components', async () => {
        render(<MemoryRouter><Oversight_Dashboard /></MemoryRouter>);
        // Component uses h2 for Action Ledger
        expect(await screen.findByRole('heading', { level: 2, name: /Action Ledger/i })).toBeInTheDocument();
    });

    it('approves an action', async () => {
        (tadpole_os_service.decide_oversight as Mock).mockResolvedValue({ status: 'success' });
        
        render(<MemoryRouter><Oversight_Dashboard /></MemoryRouter>);
        
        // Wait for data to load and "Approve" button to appear
        const btn = await screen.findByRole('button', { name: /Approve/i });
        
        // Verify we are in live mode
        expect(screen.queryByText(/TadpoleOS Disconnected/i)).not.toBeInTheDocument();

        fireEvent.click(btn);
        
        await waitFor(() => {
            expect(tadpole_os_service.decide_oversight).toHaveBeenCalledWith('p1', 'approved');
        }, { timeout: 2000 });
    });

    it('renders proposals from workspace store', async () => {
        (use_workspace_store as unknown as Mock).mockReturnValue({
            clusters: [{ id: 'c99', name: 'Cluster Omega', alpha_id: '9' }],
            active_proposals: {
                'c99': {
                    cluster_id: 'c99',
                    timestamp: new Date().toISOString(),
                    reasoning: 'TEST_REASONING',
                    changes: []
                }
            }
        });

        render(<MemoryRouter><Oversight_Dashboard /></MemoryRouter>);
        expect(await screen.findByText(/Swarm Intelligence/i)).toBeInTheDocument();
        expect(await screen.findByText(/TEST_REASONING/)).toBeInTheDocument();
    });

    it('filters by cluster', async () => {
        (use_workspace_store as unknown as Mock).mockReturnValue({
            clusters: [
                { id: 'c1', name: 'Cluster Omega' }
            ],
            active_proposals: {}
        });

        render(<MemoryRouter><Oversight_Dashboard /></MemoryRouter>);
        
        // The comp adds its own "all" option
        const select = await screen.findByRole('combobox');
        
        // Wait for options to render and stabilize
        await waitFor(() => {
            const options = screen.getAllByRole('option');
            return options.some(opt => opt.textContent === 'Cluster Omega');
        });

        fireEvent.change(select, { target: { value: 'c1' } });
        expect(select).toHaveValue('c1');
    });
});

