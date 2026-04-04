import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Swarm_Status_Header } from './Swarm_Status_Header';
import { use_settings_store } from '../stores/settings_store';
import { use_workspace_store } from '../stores/workspace_store';
import { useEngineStatus } from '../hooks/use_engine_status';

// Mock the stores and hooks
vi.mock('../stores/settings_store', () => ({
    use_settings_store: vi.fn(),
}));

vi.mock('../stores/workspace_store', () => ({
    use_workspace_store: vi.fn(),
}));

vi.mock('../hooks/use_engine_status', () => ({
    useEngineStatus: vi.fn(),
}) as never); // Added 'as never' here to ensure type safety for the mock return

// Mock Tooltip to simplify testing
vi.mock('./ui', () => ({
    Tooltip: ({ children, content }: { children: React.ReactNode, content: string }) => (
        <div data-testid="tooltip" data-content={content}>{children}</div>
    ),
}));

describe('Swarm_Status_Header Component', () => {
    const mockSettings = {
        maxAgents: 100,
        maxClusters: 20,
        maxSwarmDepth: 10,
        maxTaskLength: 65536,
        defaultBudgetUsd: 5.50,
    };

    const mockClusters = [
        { id: 'c1', collaborators: ['a1', 'a2'] },
        { id: 'c2', collaborators: ['a3'] },
    ];

    beforeEach(() => {
        vi.clearAllMocks();

        vi.mocked(use_settings_store).mockReturnValue({
            settings: mockSettings,
        } as never);

        vi.mocked(use_workspace_store).mockReturnValue({
            clusters: mockClusters,
        } as never);

        vi.mocked(useEngineStatus).mockReturnValue({
            activeAgents: 5,
            isOnline: true,
        } as never);
    });

    it('renders all swarm metrics correctly', () => {
        render(<Swarm_Status_Header />);

        // Active Agents: 3/100 (2 from cluster 1, 1 from cluster 2)
        expect(screen.getByText(/Active Agents:/i)).toBeInTheDocument();
        expect(screen.getByText('3/100')).toBeInTheDocument();

        // Active Clusters: 2/20
        expect(screen.getByText(/Active Clusters:/i)).toBeInTheDocument();
        expect(screen.getByText('2/20')).toBeInTheDocument();

        // Nodes Online: 5
        expect(screen.getByText(/Nodes Online:/i)).toBeInTheDocument();
        expect(screen.getByText('5')).toBeInTheDocument();

        // Max Depth: 10
        expect(screen.getByText(/Max Depth:/i)).toBeInTheDocument();
        expect(screen.getByText('10')).toBeInTheDocument();

        // Task Limit: 64k (65536 / 1024)
        expect(screen.getByText(/Task Limit:/i)).toBeInTheDocument();
        expect(screen.getByText('64k')).toBeInTheDocument();

        // Base Budget: $5.50
        expect(screen.getByText(/Base Budget:/i)).toBeInTheDocument();
        expect(screen.getByText('$5.50')).toBeInTheDocument();
    });

    it('displays OFFLINE when engine is disconnected', () => {
        vi.mocked(useEngineStatus).mockReturnValue({
            activeAgents: 0,
            isOnline: false,
        } as never);

        render(<Swarm_Status_Header />);

        expect(screen.getByText('OFFLINE')).toBeInTheDocument();
    });

    it('contains correct tooltip information', () => {
        render(<Swarm_Status_Header />);

        const tooltips = screen.getAllByTestId('tooltip');
        expect(tooltips.length).toBe(6);

        expect(tooltips[0].getAttribute('data-content')).toContain('Total agents assigned');
        expect(tooltips[1].getAttribute('data-content')).toContain('mission clusters vs system limit');
        expect(tooltips[2].getAttribute('data-content')).toContain('Real-time telemetry');
    });
});
