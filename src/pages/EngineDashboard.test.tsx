/**
 * @file EngineDashboard.test.tsx
 * @description Suite for the Neural Engine internal status and telemetry dashboard.
 * @module Pages/EngineDashboard
 * @testedBehavior
 * - Connection Monitoring: Real-time engine health and connection string rendering.
 * - Telemetry Visualization: Active node monitoring and state tracking.
 * @aiContext
 * - Intensive use of useEngineStatus hook mocking to simulate various cluster states.
 */
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import EngineDashboard from './EngineDashboard';
import { useEngineStatus } from '../hooks/useEngineStatus';

// Mock useEngineStatus hook
vi.mock('../hooks/useEngineStatus', () => ({
    useEngineStatus: vi.fn(),
}));

describe('EngineDashboard Page', () => {
    const mockOnlineStatus = {
        isOnline: true,
        cpu: 45.5,
        memory: 8.2,
        latency: 42,
        connectionState: 'Connected',
        activeAgents: 12,
        maxDepth: 3,
        tpm: 5000,
        recruitCount: 2
    };

    const mockOfflineStatus = {
        isOnline: false,
        cpu: 0,
        memory: 0,
        latency: 0,
        connectionState: 'Disconnected',
        activeAgents: 0,
        maxDepth: 0,
        tpm: 0,
        recruitCount: 0
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders telemetry metrics when online', () => {
        (useEngineStatus as Mock).mockReturnValue(mockOnlineStatus);
        render(<EngineDashboard />);

        expect(screen.getByText(/Neural Engine Telemetry/i)).toBeInTheDocument();
        expect(screen.getByText('45.5%')).toBeInTheDocument();
        expect(screen.getByText('8.2GB')).toBeInTheDocument();
        expect(screen.getByText('42ms')).toBeInTheDocument();
        expect(screen.getByText(/CONNECTED/i)).toBeInTheDocument();
    });

    it('renders indicators correctly when offline', () => {
        (useEngineStatus as Mock).mockReturnValue(mockOfflineStatus);
        render(<EngineDashboard />);

        expect(screen.getByText(/DISCONNECTED/i)).toBeInTheDocument();
        // Use getAllByText as "Offline" appears in multiple status labels now
        const offlineLabels = screen.getAllByText(/Offline/i);
        expect(offlineLabels.length).toBeGreaterThan(0);
    });

    it('renders the telemetry cards', () => {
        (useEngineStatus as Mock).mockReturnValue(mockOnlineStatus);
        render(<EngineDashboard />);

        // The SwarmTelemetry component renders 4 main cards
        expect(screen.getByText(/Swarm Density/i)).toBeInTheDocument();
        expect(screen.getByText(/Logic Depth/i)).toBeInTheDocument();
        expect(screen.getByText(/Swarm Velocity/i)).toBeInTheDocument();
        expect(screen.getByText(/Fiscal Burn/i)).toBeInTheDocument();
    });

    it('updates progress bar widths based on metrics', () => {
        (useEngineStatus as Mock).mockReturnValue(mockOnlineStatus);
        const { getByText } = render(<EngineDashboard />);

        // Find CPU Usage wrapper and then its progress bar
        const cpuLabel = getByText('CPU Usage');
        const card = cpuLabel.closest('div.p-5');
        const progressBar = card?.querySelector('.bg-current');

        expect(progressBar).toHaveStyle({ width: '45.5%' });
    });
});
