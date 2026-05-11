/**
 * @docs ARCHITECTURE:Quality:Verification
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:TestSuites**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Dashboard_Route_Smoke_test]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:TestSuites
 *
 * Route smoke coverage for the dashboard shell. Each APP_ROUTES entry is
 * mounted through Dashboard_Layout with a lightweight page mock so registry
 * drift, lazy import drift, and tab resolution failures are caught together.
 */

import { describe, expect, it, beforeEach, vi } from 'vitest';
import { render, screen, cleanup } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import Dashboard_Layout from './Dashboard_Layout';
import { APP_ROUTES } from '../constants/routes';
import { use_tab_store } from '../stores/tab_store';

const { mock_fetch_agents } = vi.hoisted(() => ({
    mock_fetch_agents: vi.fn(),
}));

const path_to_test_id = (path: string) => `route-${path.replace(/^\//, '').replace(/\//g, '-') || 'root'}`;

vi.mock('../components/layout/Sidebar', () => ({ Sidebar: () => <div data-testid="sidebar" /> }));
vi.mock('../components/layout/Tab_Bar', () => ({ Tab_Bar: () => <div data-testid="tab-bar" /> }));
vi.mock('../components/layout/Ticker_Header', () => ({ Ticker_Header: () => <div data-testid="ticker-header" /> }));
vi.mock('../components/ui', () => ({ Portal_Window: ({ children }: any) => <div data-testid="portal-window">{children}</div> }));
vi.mock('../components/SovereignChat', () => ({ SovereignChat: () => <div data-testid="sovereign-chat" /> }));
vi.mock('../components/Command_Palette', () => ({ Command_Palette: () => <div data-testid="command-palette" /> }));
vi.mock('../components/layout/Observability_Sidebar', () => ({ Observability_Sidebar: () => <div data-testid="observability-sidebar" /> }));
vi.mock('../components/ui/Toast_Center', () => ({ Toast_Center: () => <div data-testid="toast-center" /> }));

vi.mock('../stores/tab_store', () => ({
    use_tab_store: vi.fn(),
}));

vi.mock('../stores/agent_store', () => ({
    use_agent_store: {
        getState: vi.fn(() => ({
            fetch_agents: mock_fetch_agents,
        })),
    },
}));

vi.mock('../services/socket', () => ({
    tadpole_os_socket: {
        connect: vi.fn(),
        subscribe_pulse: vi.fn().mockReturnValue(() => {}),
    },
}));

vi.mock('../pages/Ops_Dashboard', () => ({ default: () => <div data-testid="route-dashboard" /> }));
vi.mock('../pages/Org_Chart', () => ({ default: () => <div data-testid="route-org-chart" /> }));
vi.mock('../pages/Standups', () => ({ default: () => <div data-testid="route-standups" /> }));
vi.mock('../pages/Workspaces', () => ({ default: () => <div data-testid="route-workspaces" /> }));
vi.mock('../pages/Missions', () => ({ default: () => <div data-testid="route-missions" /> }));
vi.mock('../pages/Model_Manager', () => ({ default: () => <div data-testid="route-models" /> }));
vi.mock('../pages/Agent_Manager', () => ({ default: () => <div data-testid="route-agents" /> }));
vi.mock('../pages/Engine_Dashboard', () => ({ default: () => <div data-testid="route-engine" /> }));
vi.mock('../pages/Oversight_Dashboard', () => ({ default: () => <div data-testid="route-oversight" /> }));
vi.mock('../pages/Skills', () => ({ default: () => <div data-testid="route-skills" /> }));
vi.mock('../pages/Benchmark_Analytics', () => ({ default: () => <div data-testid="route-benchmarks" /> }));
vi.mock('../pages/Scheduled_Jobs', () => ({ default: () => <div data-testid="route-scheduled-jobs" /> }));
vi.mock('../pages/Model_Store', () => ({ default: () => <div data-testid="route-infra-model-store" /> }));
vi.mock('../pages/Docs', () => ({ default: () => <div data-testid="route-docs" /> }));
vi.mock('../pages/Settings', () => ({ default: () => <div data-testid="route-settings" /> }));
vi.mock('../pages/Template_Store', () => ({ default: () => <div data-testid="route-store" /> }));
vi.mock('../pages/Security_Dashboard', () => ({ default: () => <div data-testid="route-security" /> }));

describe('Dashboard route smoke', () => {
    beforeEach(() => {
        cleanup();
        vi.clearAllMocks();
    });

    it.each(APP_ROUTES)('mounts $path through the dashboard shell', async (route) => {
        vi.mocked(use_tab_store).mockReturnValue({
            tabs: [{ id: route.path, title: route.label, path: route.path, icon: route.icon, is_detached: false }],
            active_tab_id: route.path,
            is_system_log_detached: false,
            toggle_system_log_detachment: vi.fn(),
            is_trace_stream_detached: false,
            toggle_trace_stream_detachment: vi.fn(),
            is_lineage_stream_detached: false,
            toggle_lineage_stream_detachment: vi.fn(),
            toggle_tab_detachment: vi.fn(),
        } as any);

        render(
            <MemoryRouter initialEntries={[route.path]}>
                <Dashboard_Layout />
            </MemoryRouter>
        );

        expect(await screen.findByTestId(path_to_test_id(route.path))).toBeInTheDocument();
    });
});


// Metadata: [Dashboard_Route_Smoke_test]
