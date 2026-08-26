/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Installed_Swarms_List.test
 * - **Primary Entrypoints**: none (test harness)
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `Installed_Swarms_List.test.tsx`
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { Installed_Swarms_List } from './Installed_Swarms_List';
import type { InstalledSwarmSummary } from './types';
import { system_api_service } from '../../services/system_api_service';

vi.mock('../../services/system_api_service', () => ({
    system_api_service: {
        engine: {
            get_installed_templates: vi.fn(),
            uninstall_template: vi.fn()
        }
    }
}));

describe('Installed_Swarms_List Component', () => {
    const mockSwarms: InstalledSwarmSummary[] = [
        {
            id: 'marketing',
            name: 'Marketing Automation Swarm',
            description: 'Automated lead acquisition and copywriting',
            industry: 'Marketing',
            installed_at: '2026-08-25T12:00:00Z',
            template_path: 'templates/marketing',
            agents: ['lead_gen', 'copywriter'],
            workflows: ['daily_campaign.md'],
            skills: ['lead_search.py'],
            mcp_servers: ['hubspot']
        },
        {
            id: 'logistics',
            name: 'Fleet Logistics Hub',
            description: 'Route optimization and driver assignment',
            industry: 'Transportation',
            installed_at: '2026-08-25T13:00:00Z',
            template_path: 'templates/logistics',
            agents: ['dispatcher', 'route_planner'],
            workflows: ['route_check.md'],
            skills: [],
            mcp_servers: []
        }
    ];

    beforeEach(() => {
        vi.clearAllMocks();
        vi.mocked(system_api_service.engine.get_installed_templates).mockResolvedValue({
            swarms: mockSwarms
        });
        vi.mocked(system_api_service.engine.uninstall_template).mockResolvedValue({
            status: 'success',
            message: 'Uninstalled',
            uninstalled_agents: ['lead_gen', 'copywriter'],
            uninstalled_workflows: ['daily_campaign.md'],
            uninstalled_skills: ['lead_search.py'],
            uninstalled_mcp_servers: ['hubspot'],
            archived_path: 'data/swarm_config/archive/marketing'
        });
    });

    it('renders empty state when swarms list is empty', () => {
        const onBrowse = vi.fn();
        render(
            <Installed_Swarms_List
                swarms={[]}
                isLoading={false}
                error={null}
                onUninstallClick={vi.fn()}
                onRefresh={vi.fn()}
                onBrowseMarketplace={onBrowse}
            />
        );

        expect(screen.getByText(/No Swarms Installed Yet/i)).toBeInTheDocument();
        const exploreBtn = screen.getByText(/Explore Industry Templates/i);
        fireEvent.click(exploreBtn);
        expect(onBrowse).toHaveBeenCalled();
    });

    it('renders loading indicator when isLoading is true', () => {
        render(
            <Installed_Swarms_List
                swarms={[]}
                isLoading={true}
                error={null}
                onUninstallClick={vi.fn()}
                onRefresh={vi.fn()}
                onBrowseMarketplace={vi.fn()}
            />
        );

        expect(screen.getByText(/Loading installed swarms.../i)).toBeInTheDocument();
    });

    it('renders error message when error is provided', () => {
        render(
            <Installed_Swarms_List
                swarms={[]}
                isLoading={false}
                error="Server connection failure"
                onUninstallClick={vi.fn()}
                onRefresh={vi.fn()}
                onBrowseMarketplace={vi.fn()}
            />
        );

        expect(screen.getByText(/Failed to load installed swarms/i)).toBeInTheDocument();
        expect(screen.getByText('Server connection failure')).toBeInTheDocument();
    });

    it('renders installed swarms cards with metrics and badges', () => {
        render(
            <Installed_Swarms_List
                swarms={mockSwarms}
                isLoading={false}
                error={null}
                onUninstallClick={vi.fn()}
                onRefresh={vi.fn()}
                onBrowseMarketplace={vi.fn()}
            />
        );

        expect(screen.getByText('Marketing Automation Swarm')).toBeInTheDocument();
        expect(screen.getByText('Fleet Logistics Hub')).toBeInTheDocument();
        expect(screen.getByText('Marketing')).toBeInTheDocument();
        expect(screen.getByText('Transportation')).toBeInTheDocument();
    });

    it('filters installed swarms using search input and shows empty filter message', async () => {
        render(
            <Installed_Swarms_List
                swarms={mockSwarms}
                isLoading={false}
                error={null}
                onUninstallClick={vi.fn()}
                onRefresh={vi.fn()}
                onBrowseMarketplace={vi.fn()}
            />
        );

        const searchInput = screen.getByLabelText(/Search installed swarms/i);
        await act(async () => {
            fireEvent.change(searchInput, { target: { value: 'Logistics' } });
        });

        expect(screen.queryByText('Marketing Automation Swarm')).not.toBeInTheDocument();
        expect(screen.getByText('Fleet Logistics Hub')).toBeInTheDocument();

        await act(async () => {
            fireEvent.change(searchInput, { target: { value: 'NonExistent' } });
        });
        expect(screen.getByText(/No installed swarms matching your search/i)).toBeInTheDocument();
    });

    it('triggers onUninstallClick when parent provides callback', async () => {
        const onUninstall = vi.fn();
        render(
            <Installed_Swarms_List
                swarms={mockSwarms}
                isLoading={false}
                error={null}
                onUninstallClick={onUninstall}
                onRefresh={vi.fn()}
                onBrowseMarketplace={vi.fn()}
            />
        );

        const uninstallButtons = screen.getAllByRole('button', { name: /^Uninstall/i });
        await act(async () => {
            fireEvent.click(uninstallButtons[0]);
        });

        expect(onUninstall).toHaveBeenCalledWith(mockSwarms[0]);
    });

    it('handles internal modal lifecycle in controlled mode and notifies onRefresh', async () => {
        const onRefresh = vi.fn();
        render(
            <Installed_Swarms_List
                swarms={mockSwarms}
                isLoading={false}
                error={null}
                onRefresh={onRefresh}
            />
        );

        // Click uninstall on first swarm (opens internal modal since onUninstallClick is undefined)
        const uninstallButtons = screen.getAllByRole('button', { name: /^Uninstall/i });
        await act(async () => {
            fireEvent.click(uninstallButtons[0]);
        });

        // Modal is open
        expect(screen.getByText(/Deactivation Notice/i)).toBeInTheDocument();

        // Confirm deactivation
        const confirmBtn = screen.getByRole('button', { name: /Confirm Deactivation/i });
        await act(async () => {
            fireEvent.click(confirmBtn);
        });

        expect(system_api_service.engine.uninstall_template).toHaveBeenCalledWith('marketing', true);
        expect(onRefresh).toHaveBeenCalled();

        // Success banner is visible
        await waitFor(() => {
            expect(screen.getByText(/Deactivated Marketing Automation Swarm:/i)).toBeInTheDocument();
        });
    });

    it('displays inline error in modal on uninstall failure without alert', async () => {
        vi.mocked(system_api_service.engine.uninstall_template).mockRejectedValueOnce(
            new Error('Database lock error')
        );

        render(
            <Installed_Swarms_List
                swarms={mockSwarms}
                isLoading={false}
                error={null}
                onRefresh={vi.fn()}
            />
        );

        const uninstallButtons = screen.getAllByRole('button', { name: /^Uninstall/i });
        await act(async () => {
            fireEvent.click(uninstallButtons[0]);
        });

        const confirmBtn = screen.getByRole('button', { name: /Confirm Deactivation/i });
        await act(async () => {
            fireEvent.click(confirmBtn);
        });

        // Error banner is rendered inside modal
        expect(screen.getByText('Database lock error')).toBeInTheDocument();
    });

    it('uncontrolled mode fetches automatically on mount', async () => {
        render(<Installed_Swarms_List />);

        await waitFor(() => {
            expect(system_api_service.engine.get_installed_templates).toHaveBeenCalledTimes(1);
            expect(screen.getByText('Marketing Automation Swarm')).toBeInTheDocument();
        });
    });

    it('prevents refetch loop when parent re-renders with new onRefresh callback identity', async () => {
        const { rerender } = render(<Installed_Swarms_List onRefresh={() => {}} />);

        // In uncontrolled mode, initial mount triggers 1 fetch
        await waitFor(() => {
            expect(system_api_service.engine.get_installed_templates).toHaveBeenCalledTimes(1);
        });

        // Re-render with brand new function reference
        rerender(<Installed_Swarms_List onRefresh={() => {}} />);
        rerender(<Installed_Swarms_List onRefresh={() => {}} />);

        // Should NOT trigger any additional fetches
        expect(system_api_service.engine.get_installed_templates).toHaveBeenCalledTimes(1);
    });
});
