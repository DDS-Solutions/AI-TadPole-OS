/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / Workspaces_E2E.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, act, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import Workspaces from './Workspaces';
import { use_workspace_store } from '../stores/workspace_store';
import { use_agent_store } from '../stores/agent_store';
import { set_workspace_store } from '../services/workspace_service';
import { system_api_service } from '../services/system_api_service';
import type { Agent } from '../types';

// Mock system_api_service
vi.mock('../services/system_api_service', () => ({
    system_api_service: {
        workspace: {
            get_workspaces_status: vi.fn(),
            get_workspace_files: vi.fn(),
            get_file_history: vi.fn(),
            restore_file_version: vi.fn(),
        },
        oversight: {
            get_mission_quotas: vi.fn(),
            get_governance_settings: vi.fn(),
            update_mission_quota: vi.fn(),
            update_governance_settings: vi.fn(),
        }
    }
}));

// Mock ResizeObserver
global.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
};

describe('Workspaces Screen End-to-End Suite', () => {
    const mock_agents: Agent[] = [
        {
            id: 'alpha-lead',
            name: 'Commander Alpha',
            role: 'Lead Architect',
            department: 'Executive',
            model: 'gemini-1.5-pro',
            status: 'idle',
            tokens_used: 1200,
            category: 'ai',
            theme_color: '#3b82f6'
        },
        {
            id: 'agent-dev',
            name: 'Senior Dev Agent',
            role: 'Software Engineer',
            department: 'Engineering',
            model: 'claude-3-5-sonnet',
            status: 'idle',
            tokens_used: 4500,
            category: 'ai',
            theme_color: '#10b981'
        },
        {
            id: 'agent-silo',
            name: 'Unassigned Agent',
            role: 'Lone Researcher',
            department: 'Research',
            model: 'gpt-4o',
            status: 'idle',
            tokens_used: 300,
            category: 'ai'
        }
    ];

    const initial_clusters = [
        {
            id: 'cl-command-e2e',
            name: 'Strategic Command Unit',
            department: 'Executive' as const,
            path: '/workspaces/strategic-command',
            budget_usd: 1000,
            cost_usd: 120,
            alpha_id: 'alpha-lead',
            collaborators: ['alpha-lead', 'agent-dev'],
            theme: 'blue' as const,
            pending_tasks: [
                {
                    id: 'task-branch-1',
                    description: 'Deploy Secure IPC Transport',
                    agent_id: 'agent-dev',
                    target_path: '/workspaces/strategic-command/ipc.rs',
                    status: 'pending' as const,
                    timestamp: 1700000000000
                },
                {
                    id: 'task-branch-2',
                    description: 'Legacy Token Optimization',
                    agent_id: 'alpha-lead',
                    target_path: '/workspaces/strategic-command/tokens.rs',
                    status: 'completed' as const,
                    timestamp: 1699990000000
                }
            ]
        }
    ];

    beforeEach(() => {
        vi.clearAllMocks();
        set_workspace_store(use_workspace_store);

        // Reset real stores to predictable test baseline
        use_agent_store.setState({
            agents: mock_agents,
            is_loading: false,
            error: null
        });

        use_workspace_store.setState({
            clusters: initial_clusters,
            sync_status: {},
            active_proposals: {}
        });

        // Setup mock response from backend GET /v1/system/workspaces/status
        vi.mocked(system_api_service.workspace.get_workspaces_status).mockResolvedValue([
            {
                id: 'manifest-cmd',
                agent_id: 'alpha-lead',
                source_type: 'fs',
                source_uri: 'workspaces/strategic-command',
                last_sync_at: '2026-10-28T16:00:00Z',
                status: 'idle',
                file_count: 142,
                total_bytes: 2097152, // 2 MB
                detected_environments: ['vs_code', 'docker', 'wasm_sandbox'],
                mounted_okf_nodes: [
                    { id: 'okf-sec-gate', title: 'Zero Trust Guard', concept_type: 'playbook', security_tier: 'GOLD' },
                    { id: 'okf-audit', title: 'Audit Protocol', concept_type: 'playbook', security_tier: 'SILVER' }
                ],
                okf_validation: {
                    status: 'nominal',
                    message: undefined
                }
            }
        ]);
    });

    afterEach(() => {
        vi.restoreAllMocks();
    });

    it('renders the complete screen hierarchy end-to-end with real stores', async () => {
        await act(async () => {
            render(<Workspaces />);
        });

        // 1. Header validation
        expect(screen.getByText('WORKSPACE MANAGER')).toBeInTheDocument();
        expect(screen.getByText(/1 ACTIVE WORKSPACES/i)).toBeInTheDocument();

        // 2. Cluster details
        expect(screen.getByText('STRATEGIC COMMAND UNIT')).toBeInTheDocument();
        expect(screen.getByText(/Executive CLUSTER/i)).toBeInTheDocument();

        // 3. Collaborator avatars
        expect(screen.getByText('C')).toBeInTheDocument(); // Commander Alpha
        expect(screen.getByText('S')).toBeInTheDocument(); // Senior Dev Agent

        // 4. Pending task branches
        expect(screen.getByText('Deploy Secure IPC Transport')).toBeInTheDocument();
        expect(screen.getByText('Legacy Token Optimization')).toBeInTheDocument();

        // 5. Sovereign Synchronization Metrics table
        expect(screen.getByText('Sovereign Synchronization Metrics')).toBeInTheDocument();
        expect(screen.getByText('Strategic Command Unit')).toBeInTheDocument();

        // 6. Legacy Silos
        expect(screen.getByText('Legacy Agent Silos')).toBeInTheDocument();
        expect(screen.getByText('Unassigned Agent')).toBeInTheDocument();
    });

    it('successfully syncs telemetry from backend and renders environment & OKF badges', async () => {
        await act(async () => {
            render(<Workspaces />);
        });

        // Wait for telemetry sync to resolve
        await waitFor(() => {
            expect(system_api_service.workspace.get_workspaces_status).toHaveBeenCalled();
        });

        // Verify byte formatting and status in details card (2 MB ACTIVE • IDLE)
        await waitFor(() => {
            expect(screen.getByText(/2 MB ACTIVE • IDLE/i)).toBeInTheDocument();
        });

        // Verify detected environments rendered
        expect(screen.getByText('VS Code')).toBeInTheDocument();
        expect(screen.getByText('Docker Sandbox')).toBeInTheDocument();
        expect(screen.getByText('WASM Isolation')).toBeInTheDocument();

        // Verify OKF governance cards and mounted playbooks
        expect(screen.getByText('OKF GOVERNANCE')).toBeInTheDocument();
        expect(screen.getByText('NOMINAL STATE')).toBeInTheDocument();
        expect(screen.getByText('Zero Trust Guard')).toBeInTheDocument();
        expect(screen.getByText('GOLD')).toBeInTheDocument();
        expect(screen.getByText('Audit Protocol')).toBeInTheDocument();
        expect(screen.getByText('SILVER')).toBeInTheDocument();

        // Verify metrics table shows synced depth
        expect(screen.getByText('142 Files')).toBeInTheDocument();
    });

    it('handles warning and critical OKF states with alert banner display', async () => {
        vi.mocked(system_api_service.workspace.get_workspaces_status).mockResolvedValueOnce([
            {
                id: 'manifest-warn',
                agent_id: 'alpha-lead',
                source_type: 'fs',
                source_uri: '/workspaces/strategic-command',
                last_sync_at: '2026-10-28T16:00:00Z',
                status: 'error',
                file_count: 88,
                total_bytes: 1048576,
                detected_environments: ['docker'],
                mounted_okf_nodes: [],
                okf_validation: {
                    status: 'critical',
                    message: 'Unauthorized container root mount detected in cluster'
                }
            }
        ]);

        await act(async () => {
            render(<Workspaces />);
        });

        await waitFor(() => {
            expect(screen.getByText('CRITICAL SEC VIOLATION')).toBeInTheDocument();
        });

        expect(screen.getByText('Unauthorized container root mount detected in cluster')).toBeInTheDocument();
        expect(screen.getByText('ERROR')).toBeInTheDocument();
    });

    it('approves a task branch end-to-end, mutating store state and updating UI', async () => {
        await act(async () => {
            render(<Workspaces />);
        });

        // Find approve button
        const buttons = screen.getAllByRole('button');
        const approve_btn = buttons[0]; // First button corresponds to approve
        expect(approve_btn).toBeInTheDocument();

        // Initial state check
        const initial_task = use_workspace_store.getState().clusters[0].pending_tasks.find(t => t.id === 'task-branch-1');
        expect(initial_task?.status).toBe('pending');

        // Click approve
        await act(async () => {
            fireEvent.click(approve_btn);
        });

        // Verify store mutated to completed
        const updated_task = use_workspace_store.getState().clusters[0].pending_tasks.find(t => t.id === 'task-branch-1');
        expect(updated_task?.status).toBe('completed');
    });

    it('rejects a task branch end-to-end, mutating store state and updating UI', async () => {
        await act(async () => {
            render(<Workspaces />);
        });

        // Find reject button (second button)
        const buttons = screen.getAllByRole('button');
        const reject_btn = buttons[1];
        expect(reject_btn).toBeInTheDocument();

        // Click reject
        await act(async () => {
            fireEvent.click(reject_btn);
        });

        // Verify store mutated to rejected
        const updated_task = use_workspace_store.getState().clusters[0].pending_tasks.find(t => t.id === 'task-branch-1');
        expect(updated_task?.status).toBe('rejected');
    });

    it('correctly handles empty state when no clusters exist', async () => {
        use_workspace_store.setState({ clusters: [] });

        await act(async () => {
            render(<Workspaces />);
        });

        expect(screen.getByText('0 ACTIVE WORKSPACES', { exact: false })).toBeInTheDocument();
        expect(screen.getByText('No active synchronization clusters detected.')).toBeInTheDocument();
    });

    it('correctly handles path normalization with leading vs non-leading slashes', async () => {
        // Backend returns non-leading slash 'workspaces/strategic-command'
        // Cluster path has leading slash '/workspaces/strategic-command'
        await act(async () => {
            render(<Workspaces />);
        });

        await waitFor(() => {
            expect(screen.getByText(/2 MB ACTIVE • IDLE/i)).toBeInTheDocument();
        });

        // Both keys exist in sync_status
        const status = use_workspace_store.getState().sync_status;
        expect(status['workspaces/strategic-command']).toBeDefined();
        expect(status['/workspaces/strategic-command']).toBeDefined();
    });
});
