/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Validates the Organization Header's breadcrumb navigation and engine connectivity status.** 
 * Verifies correct title rendering for Ops Center, Hierarchy, and Provider layers. 
 * Mocks `i18n` and `settings_store` to isolate health telemetry display and connection state signaling.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Broken breadcrumb links or search input debouncing failures causing stale UI results during high-frequency route changes.
 * - **Telemetry Link**: Search `[Org_Header.test]` in tracing logs.
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Org_Header } from './Org_Header';

// Mock Swarm_Status_Header
vi.mock('../Swarm_Status_Header', () => ({
    Swarm_Status_Header: () => <div data-testid="swarm-status-header" />
}));

// Mock i18n
vi.mock('../../i18n', () => ({
    i18n: {
        t: (key: string, options?: any) => {
            if (key === 'header.engine_online_agents') return `ONLINE • ${options?.count} AGENTS`;
            return key;
        },
    },
}));

// Mock UI components
vi.mock('../ui', () => ({
    Tooltip: ({ children, content }: any) => <div title={content}>{children}</div>
}));

// Mock settings store
vi.mock('../../stores/settings_store', () => ({
    use_settings_store: () => ({
        settings: { privacy_mode: false },
        update_setting: vi.fn()
    })
}));

describe('Org_Header Component', () => {
    it('renders the correct title based on pathname', () => {
        const { rerender } = render(
            <Org_Header pathname="/" connection_state="connected" engine_health={null} />
        );
        expect(screen.getByText('header.ops_center')).toBeInTheDocument();

        rerender(<Org_Header pathname="/org-chart" connection_state="connected" engine_health={null} />);
        expect(screen.getByText('header.hierarchy_layer')).toBeInTheDocument();

        rerender(<Org_Header pathname="/models" connection_state="connected" engine_health={null} />);
        expect(screen.getByText('header.provider_manager')).toBeInTheDocument();
    });

    it('displays connection status correctly', () => {
        const { rerender } = render(
            <Org_Header pathname="/" connection_state="connected" engine_health={{ uptime: 100, agent_count: 5 }} />
        );
        expect(screen.getByText('ONLINE • 5 AGENTS')).toBeInTheDocument();

        rerender(<Org_Header pathname="/" connection_state="connecting" engine_health={null} />);
        expect(screen.getByText('header.connecting')).toBeInTheDocument();

        rerender(<Org_Header pathname="/" connection_state="disconnected" engine_health={null} />);
        expect(screen.getByText('header.engine_offline_label')).toBeInTheDocument();
    });

    it('renders the swarm status header and command palette hint', () => {
        render(<Org_Header pathname="/" connection_state="connected" engine_health={null} />);
        
        expect(screen.getByTestId('swarm-status-header')).toBeInTheDocument();
        expect(screen.getByText('K')).toBeInTheDocument();
    });
});

