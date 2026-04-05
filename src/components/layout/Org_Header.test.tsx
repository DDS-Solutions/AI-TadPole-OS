/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Manages the interface for Org Header.test. 
 * Part of the Tadpole-OS Modular UI ecosystem.
 */

/**
 * @file Org_Header.test.tsx
 * @description Suite for the Top-level Organizational Header.
 * @module Components/Layout/Org_Header
 * @testedBehavior
 * - Navigation: Renders the correct navigational context title for the current route.
 * - Connectivity: Displays engine connection status (ONLINE/OFFLINE/CONNECTING).
 * - Health Telemetry: Shows active agent counts when connected.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks i18n to return keys for stable assertion matching.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
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

