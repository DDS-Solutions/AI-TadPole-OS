/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification of the Engine Dashboard's core telemetry metrics** (CPU, Memory, Latency). 
 * Ensures accurate real-time dashboard updates from the `engine_status` hook and validates the 'OFFLINE' state indicators. 
 * Mocks `Swarm_Visualizer` and `Telemetry_Graph` to focus on aggregate status metric rendering.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Telemetry lag during high-frequency updates or incorrect color mapping for status transition bars.
 * - **Telemetry Link**: Search `[Engine_Dashboard.test]` in tracing logs.
 */


/**
 * @file Engine_Dashboard.test.tsx
 * @description Suite for the Neural Engine internal status and telemetry dashboard.
 * @module Pages/Engine_Dashboard
 * @testedBehavior
 * - Connection Monitoring: Real-time engine health and connection string rendering.
 * - Telemetry Visualization: Active node monitoring and state tracking.
 * @aiContext
 * - Intensive use of useEngineStatus hook mocking to simulate various cluster states.
 * - Refactored for 100% snake_case architectural parity.
 */
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import Engine_Dashboard from './Engine_Dashboard';
import { useEngineStatus } from '../hooks/use_engine_status';

vi.mock('../hooks/use_engine_status', () => ({
    useEngineStatus: vi.fn(),
}));

vi.mock('../components/Swarm_Visualizer', () => ({
    Swarm_Visualizer: () => <div data-testid="swarm-visualizer" />
}));

vi.mock('../components/Telemetry_Graph', () => ({
    Telemetry_Graph: () => <div data-testid="telemetry-graph" />
}));

describe('Engine_Dashboard Page', () => {
    const mock_online_status = {
        is_online: true,
        cpu: 45.5,
        memory: 8.2,
        latency: 42,
        connection_state: 'Connected',
        active_agents: 12,
        max_depth: 3,
        tpm: 5000,
        recruit_count: 2
    };

    const mock_offline_status = {
        is_online: false,
        cpu: 0,
        memory: 0,
        latency: 0,
        connection_state: 'Disconnected',
        active_agents: 0,
        max_depth: 0,
        tpm: 0,
        recruit_count: 0
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders telemetry metrics when online', () => {
        (useEngineStatus as Mock).mockReturnValue(mock_online_status);
        render(<Engine_Dashboard />);

        expect(screen.getByText(/Neural Engine Telemetry/i)).toBeInTheDocument();
        expect(screen.getByText('45.5%')).toBeInTheDocument();
        expect(screen.getByText('8.2GB')).toBeInTheDocument();
        expect(screen.getByText('42ms')).toBeInTheDocument();
        expect(screen.getByText(/CONNECTED/i)).toBeInTheDocument();
    });

    it('renders indicators correctly when offline', () => {
        (useEngineStatus as Mock).mockReturnValue(mock_offline_status);
        render(<Engine_Dashboard />);

        expect(screen.getByText(/DISCONNECTED/i)).toBeInTheDocument();
        // Use getAllByText as "Offline" appears in multiple status labels now
        const offline_labels = screen.getAllByText(/Offline/i);
        expect(offline_labels.length).toBeGreaterThan(0);
    });

    it('renders the telemetry cards', () => {
        (useEngineStatus as Mock).mockReturnValue(mock_online_status);
        render(<Engine_Dashboard />);

        // The Swarm_Telemetry component renders 4 main cards
        expect(screen.getByText(/Swarm Density/i)).toBeInTheDocument();
        expect(screen.getByText(/Logic Depth/i)).toBeInTheDocument();
        expect(screen.getByText(/Swarm Velocity/i)).toBeInTheDocument();
        expect(screen.getByText(/Fiscal Burn/i)).toBeInTheDocument();
    });

    it('updates progress bar widths based on metrics', () => {
        (useEngineStatus as Mock).mockReturnValue(mock_online_status);
        render(<Engine_Dashboard />);

        // Find CPU Usage wrapper and then its progress bar
        const cpu_label = screen.getByText('CPU Usage');
        const card = cpu_label.closest('div.p-5');
        const progress_bar = card?.querySelector('.bg-current');

        expect(progress_bar).toHaveStyle({ width: '45.5%' });
    });
});


// Metadata: [Engine_Dashboard_test]

// Metadata: [Engine_Dashboard_test]
