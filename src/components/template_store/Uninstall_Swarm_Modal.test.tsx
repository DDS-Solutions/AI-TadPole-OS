/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Uninstall_Swarm_Modal.test
 * - **Primary Entrypoints**: none (test harness)
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 * - `[Structural]` Dialog conforms to WAI-ARIA modal dialog semantics with keyboard navigation and focus restoration.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `Uninstall_Swarm_Modal.test.tsx`
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { Uninstall_Swarm_Modal } from './Uninstall_Swarm_Modal';
import type { InstalledSwarmSummary } from './types';

describe('Uninstall_Swarm_Modal Component', () => {
    const mockSwarm: InstalledSwarmSummary = {
        id: 'field_services',
        name: 'Field Services & Dispatch',
        description: 'Dispatcher and technician management',
        industry: 'Field Services',
        installed_at: '2026-08-25T14:30:00Z',
        template_path: 'templates/field_services',
        agents: ['dispatcher', 'lead_tech', 'billing_sync'],
        workflows: ['daily_dispatch.md', 'ticket_escalation.md'],
        skills: ['schedule_job.py'],
        mcp_servers: ['stripe-mcp']
    };

    it('does not render when isOpen is false or swarm is null', () => {
        const { rerender } = render(
            <Uninstall_Swarm_Modal
                swarm={mockSwarm}
                isOpen={false}
                onClose={vi.fn()}
                onConfirm={vi.fn()}
                isUninstalling={false}
            />
        );
        expect(screen.queryByRole('dialog')).not.toBeInTheDocument();

        rerender(
            <Uninstall_Swarm_Modal
                swarm={null}
                isOpen={true}
                onClose={vi.fn()}
                onConfirm={vi.fn()}
                isUninstalling={false}
            />
        );
        expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });

    it('renders with accessible dialog semantics, impact summary, and breakdown', () => {
        render(
            <Uninstall_Swarm_Modal
                swarm={mockSwarm}
                isOpen={true}
                onClose={vi.fn()}
                onConfirm={vi.fn()}
                isUninstalling={false}
            />
        );

        const dialog = screen.getByRole('dialog');
        expect(dialog).toHaveAttribute('aria-modal', 'true');
        expect(dialog).toHaveAttribute('aria-labelledby', 'uninstall-title');

        expect(screen.getByText(/Deactivation Notice/i)).toBeInTheDocument();
        expect(screen.getByText('3')).toBeInTheDocument(); // 3 agents
        expect(screen.getByText('2')).toBeInTheDocument(); // 2 workflows
        expect(screen.getAllByText('1')).toHaveLength(2); // 1 skill and 1 MCP server
    });

    it('calls onConfirm with swarmId and archive boolean when confirmed', async () => {
        const onConfirm = vi.fn().mockResolvedValue(undefined);
        const onClose = vi.fn();

        render(
            <Uninstall_Swarm_Modal
                swarm={mockSwarm}
                isOpen={true}
                onClose={onClose}
                onConfirm={onConfirm}
                isUninstalling={false}
            />
        );

        const confirmBtn = screen.getByRole('button', { name: /Confirm Uninstall|Confirm Deactivation/i });
        await act(async () => {
            fireEvent.click(confirmBtn);
        });

        expect(onConfirm).toHaveBeenCalledWith('field_services', true);
    });

    it('supports toggling archive option off', async () => {
        const onConfirm = vi.fn().mockResolvedValue(undefined);
        render(
            <Uninstall_Swarm_Modal
                swarm={mockSwarm}
                isOpen={true}
                onClose={vi.fn()}
                onConfirm={onConfirm}
                isUninstalling={false}
            />
        );

        const archiveCheckbox = screen.getByRole('checkbox');
        expect(archiveCheckbox).toBeChecked();

        await act(async () => {
            fireEvent.click(archiveCheckbox);
        });
        expect(archiveCheckbox).not.toBeChecked();

        const confirmBtn = screen.getByRole('button', { name: /Confirm Uninstall|Confirm Deactivation/i });
        await act(async () => {
            fireEvent.click(confirmBtn);
        });

        expect(onConfirm).toHaveBeenCalledWith('field_services', false);
    });

    it('displays error banner when error prop is provided', () => {
        render(
            <Uninstall_Swarm_Modal
                swarm={mockSwarm}
                isOpen={true}
                error="Database lock timeout on agent deletion"
                onClose={vi.fn()}
                onConfirm={vi.fn()}
                isUninstalling={false}
            />
        );

        expect(screen.getByText('Database lock timeout on agent deletion')).toBeInTheDocument();
    });

    it('dismisses on Escape key when not in flight', async () => {
        const onClose = vi.fn();
        render(
            <Uninstall_Swarm_Modal
                swarm={mockSwarm}
                isOpen={true}
                onClose={onClose}
                onConfirm={vi.fn()}
                isUninstalling={false}
            />
        );

        await act(async () => {
            fireEvent.keyDown(window, { key: 'Escape' });
        });
        expect(onClose).toHaveBeenCalledTimes(1);
    });

    it('ignores Escape key when isUninstalling is true', async () => {
        const onClose = vi.fn();
        render(
            <Uninstall_Swarm_Modal
                swarm={mockSwarm}
                isOpen={true}
                onClose={onClose}
                onConfirm={vi.fn()}
                isUninstalling={true}
            />
        );

        await act(async () => {
            fireEvent.keyDown(window, { key: 'Escape' });
        });
        expect(onClose).not.toHaveBeenCalled();
    });

    it('resets archive state to true when switching swarm targets', async () => {
        const { rerender } = render(
            <Uninstall_Swarm_Modal
                swarm={mockSwarm}
                isOpen={true}
                onClose={vi.fn()}
                onConfirm={vi.fn()}
                isUninstalling={false}
            />
        );

        const archiveCheckbox = screen.getByRole('checkbox');
        await act(async () => {
            fireEvent.click(archiveCheckbox);
        });
        expect(archiveCheckbox).not.toBeChecked();

        // Switch to different swarm
        const anotherSwarm: InstalledSwarmSummary = {
            ...mockSwarm,
            id: 'analytics',
            name: 'Analytics Swarm'
        };

        rerender(
            <Uninstall_Swarm_Modal
                swarm={anotherSwarm}
                isOpen={true}
                onClose={vi.fn()}
                onConfirm={vi.fn()}
                isUninstalling={false}
            />
        );

        expect(screen.getByRole('checkbox')).toBeChecked();
    });

    it('prevents double-fire during in-flight confirmation', async () => {
        let resolveConfirm: () => void = () => {};
        const onConfirm = vi.fn().mockImplementation(() => new Promise<void>((resolve) => {
            resolveConfirm = resolve;
        }));

        render(
            <Uninstall_Swarm_Modal
                swarm={mockSwarm}
                isOpen={true}
                onClose={vi.fn()}
                onConfirm={onConfirm}
                isUninstalling={false}
            />
        );

        const confirmBtn = screen.getByRole('button', { name: /Confirm Uninstall|Confirm Deactivation/i });

        // Double click rapidly
        await act(async () => {
            fireEvent.click(confirmBtn);
            fireEvent.click(confirmBtn);
        });

        expect(onConfirm).toHaveBeenCalledTimes(1);

        await act(async () => {
            resolveConfirm();
        });
    });
});
