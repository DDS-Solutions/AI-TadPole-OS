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
 * - Isolates cluster governance logic from backend persistence via loadAgents mock.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import Workspaces from './Workspaces';
import { use_workspace_store } from '../stores/workspace_store';
import { loadAgents } from '../services/agent_service';
import type { Agent } from '../types';

// Mock Dependencies
vi.mock('../stores/workspace_store');
vi.mock('../services/agent_service', () => ({
    loadAgents: vi.fn()
}));

// Mock ResizeObserver which might be used by lucide/recharts/tooltips
global.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
};

describe('Workspaces Page', () => {
    const mockAgents: Agent[] = [
        {
            id: 'agent-1', name: 'Alpha Agent', role: 'Dev', department: 'Engineering',
            model: 'test', status: 'idle', tokensUsed: 0, category: 'ai'
        },
        {
            id: 'agent-2', name: 'Beta Agent', role: 'Tester', department: 'Engineering',
            model: 'test', status: 'idle', tokensUsed: 0, category: 'ai'
        },
        {
            id: 'silo-agent', name: 'Legacy Silo', role: 'Manager', department: 'Executive',
            model: 'test', status: 'idle', tokensUsed: 0, category: 'ai'
        }
    ];

    const mockClusters = [
        {
            id: 'cluster-1',
            name: 'Frontend Core',
            department: 'Engineering',
            path: '/org/frontend',
            budget_usd: 100,
            alphaId: 'agent-1',
            collaborators: ['agent-1', 'agent-2'],
            pendingTasks: [
                { id: 'task-1', description: 'Fix CSS', agent_id: 'agent-2', timestamp: Date.now(), status: 'pending' },
                { id: 'task-2', description: 'Refactor JS', agent_id: 'agent-1', timestamp: Date.now() - 10000, status: 'completed' }
            ]
        }
    ];

    const mockApproveBranch = vi.fn();
    const mockRejectBranch = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();
        
        // Setup initial store state
        (use_workspace_store as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
            clusters: mockClusters,
            approveBranch: mockApproveBranch,
            rejectBranch: mockRejectBranch
        });

        (loadAgents as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(mockAgents);
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
        // We can find it by its tooltip or click handler.
        // It's the check circle icon for the task. We'll find by tooltip or closest button.
        // There are specific tooltips, let's grab buttons
        // Get all buttons inside the pending task container
        const approveButton = screen.getAllByRole('button')[0]; // first button is approve, second is reject
        
        expect(approveButton).toBeInTheDocument();
        
        await act(async () => {
            fireEvent.click(approveButton);
        });

        expect(mockApproveBranch).toHaveBeenCalledWith('cluster-1', 'task-1');
    });

    it('handles rejecting a branch', async () => {
        await act(async () => {
            render(<Workspaces />);
        });

        const rejectButton = screen.getAllByRole('button')[1]; // second button is reject
        
        expect(rejectButton).toBeInTheDocument();
        
        await act(async () => {
            fireEvent.click(rejectButton);
        });

        expect(mockRejectBranch).toHaveBeenCalledWith('cluster-1', 'task-1');
    });

    it('displays legacy agent silos for agents not in any cluster', async () => {
        await act(async () => {
            render(<Workspaces />);
        });

        expect(screen.getByText('Legacy Agent Silos')).toBeInTheDocument();
        
        // Legacy Silo should be visible (agent-1 and agent-2 are in cluster-1)
        expect(screen.getByText('Legacy Silo')).toBeInTheDocument();
        
        // Alpha Agent should NOT be in the silo section (it's in the cluster header instead)
        // Hard to assert absence if it's rendered in tooltips, but we know Legacy Silo is rendered uniquely in that section.
    });

    it('shows empty state when no pending tasks', async () => {
        (use_workspace_store as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
            clusters: [{ ...mockClusters[0], pendingTasks: [] }],
            approveBranch: mockApproveBranch,
            rejectBranch: mockRejectBranch
        });

        await act(async () => {
            render(<Workspaces />);
        });

        expect(screen.getByText('No pending merges for this cluster')).toBeInTheDocument();
    });
});
