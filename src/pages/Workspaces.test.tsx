/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the Workspace management and project isolation logic.** 
 * Verifies the switching between isolated neural environments, cluster governance (approve/reject branches), and legacy agent silo management. 
 * Mocks `use_workspace_store` to control cluster and branch state, isolating governance logic from backend persistence.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: State leakage between workspaces (e.g. agents appearing in the wrong project) or failure to hydrate the `workspace_store` after a project switch.
 * - **Telemetry Link**: Search `[Workspaces.test]` in tracing logs.
 */


/**
 * @file Workspaces.test.tsx
 * @description Suite for the Workspace Manager (Cluster Governance) page.
 * @module Pages/Workspaces
 * @testedBehavior
 * - Cluster Monitoring: Rendering of active workspace branches and pending merges.
 * - Governance: Handling of "Approve" and "Reject" actions for task branches.
 * - Silo Management: Verification of legacy agent isolation within silos.
 * @aiContext
 * - Mocks use_workspace_store to control cluster and branch state.
 * - Isolates cluster governance logic from backend persistence via load_agents mock.
 * - Refactored for 100% snake_case architectural parity.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import Workspaces from './Workspaces';
import { use_workspace_store } from '../stores/workspace_store';
import { load_agents } from '../services/agent_service';
import type { Agent } from '../types';

// Mock Dependencies
vi.mock('../stores/workspace_store');
vi.mock('../services/agent_service', () => ({
    load_agents: vi.fn()
}));

// Mock ResizeObserver which might be used by lucide/recharts/tooltips
global.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
};

describe('Workspaces Page', () => {
    const mock_agents: Agent[] = [
        {
            id: 'agent-1', name: 'Alpha Agent', role: 'Dev', department: 'Engineering',
            model: 'test', status: 'idle', tokens_used: 0, category: 'ai'
        },
        {
            id: 'agent-2', name: 'Beta Agent', role: 'Tester', department: 'Engineering',
            model: 'test', status: 'idle', tokens_used: 0, category: 'ai'
        },
        {
            id: 'silo-agent', name: 'Legacy Silo', role: 'Manager', department: 'Executive',
            model: 'test', status: 'idle', tokens_used: 0, category: 'ai'
        }
    ];

    const mock_clusters = [
        {
            id: 'cluster-1',
            name: 'Frontend Core',
            department: 'Engineering',
            path: '/org/frontend',
            budget_usd: 100,
            alpha_id: 'agent-1',
            collaborators: ['agent-1', 'agent-2'],
            pending_tasks: [
                { id: 'task-1', description: 'Fix CSS', agent_id: 'agent-2', timestamp: Date.now(), status: 'pending' },
                { id: 'task-2', description: 'Refactor JS', agent_id: 'agent-1', timestamp: Date.now() - 10000, status: 'completed' }
            ]
        }
    ];

    const mock_approve_branch = vi.fn();
    const mock_reject_branch = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();
        
        // Setup initial store state
        (use_workspace_store as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
            clusters: mock_clusters,
            approve_branch: mock_approve_branch,
            reject_branch: mock_reject_branch
        });

        (load_agents as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(mock_agents);
    });

    it('renders the header and general layout', async () => {
        await act(async () => {
            render(<Workspaces />);
        });

        expect(screen.getByText('WORKSPACE MANAGER')).toBeInTheDocument();
        expect(screen.getByText('1 ACTIVE WORKSPACES', { exact: false })).toBeInTheDocument();
    });

    it('renders cluster details correctly', async () => {
        await act(async () => {
            render(<Workspaces />);
        });

        // Cluster name & Department
        expect(screen.getByText('FRONTEND CORE')).toBeInTheDocument();
        expect(screen.getByText(/Engineering CLUSTER/i)).toBeInTheDocument();

        // Active Tasks count (1 pending)
        expect(screen.getByText(/Active Task Branches \(1\)/i)).toBeInTheDocument();

        // Tasks
        expect(screen.getByText('Fix CSS')).toBeInTheDocument();
        expect(screen.getByText('Refactor JS')).toBeInTheDocument();
    });

    it('handles approving a branch', async () => {
        await act(async () => {
            render(<Workspaces />);
        });

        // The approve button should be present for the 'pending' task
        const approve_button = screen.getAllByRole('button')[0]; // first button is approve, second is reject
        
        expect(approve_button).toBeInTheDocument();
        
        await act(async () => {
            fireEvent.click(approve_button);
        });

        expect(mock_approve_branch).toHaveBeenCalledWith('cluster-1', 'task-1');
    });

    it('handles rejecting a branch', async () => {
        await act(async () => {
            render(<Workspaces />);
        });

        const reject_button = screen.getAllByRole('button')[1]; // second button is reject
        
        expect(reject_button).toBeInTheDocument();
        
        await act(async () => {
            fireEvent.click(reject_button);
        });

        expect(mock_reject_branch).toHaveBeenCalledWith('cluster-1', 'task-1');
    });

    it('displays legacy agent silos for agents not in any cluster', async () => {
        await act(async () => {
            render(<Workspaces />);
        });

        expect(screen.getByText('Legacy Agent Silos')).toBeInTheDocument();
        
        // Legacy Silo should be visible (agent-1 and agent-2 are in cluster-1)
        expect(screen.getByText('Legacy Silo')).toBeInTheDocument();
    });

    it('shows empty state when no pending tasks', async () => {
        (use_workspace_store as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
            clusters: [{ ...mock_clusters[0], pending_tasks: [] }],
            approve_branch: mock_approve_branch,
            reject_branch: mock_reject_branch
        });

        await act(async () => {
            render(<Workspaces />);
        });

        expect(screen.getByText('No pending merges for this cluster')).toBeInTheDocument();
    });
});


// Metadata: [Workspaces_test]

// Metadata: [Workspaces_test]
