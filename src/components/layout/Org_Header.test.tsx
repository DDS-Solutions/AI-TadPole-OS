import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Org_Header } from './Org_Header';

// Mock Swarm_Status_Header
vi.mock('../Swarm_Status_Header', () => ({
    Swarm_Status_Header: () => <div data-testid="swarm-status-header" />
}));

vi.mock('../ui', () => ({
    Tooltip: ({ children, content }: any) => <div title={content}>{children}</div>
}));

describe('Org_Header Component', () => {
    it('renders the correct title based on pathname', () => {
        const { rerender } = render(
            <Org_Header pathname="/" connectionState="connected" engineHealth={null} />
        );
        expect(screen.getByText('Operations Center')).toBeInTheDocument();

        rerender(<Org_Header pathname="/org-chart" connectionState="connected" engineHealth={null} />);
        expect(screen.getByText('Agent Hierarchy Layer')).toBeInTheDocument();

        rerender(<Org_Header pathname="/models" connectionState="connected" engineHealth={null} />);
        expect(screen.getByText('AI Provider Manager')).toBeInTheDocument();
    });

    it('displays connection status correctly', () => {
        const { rerender } = render(
            <Org_Header pathname="/" connectionState="connected" engineHealth={{ uptime: 100, agentCount: 5 }} />
        );
        expect(screen.getByText('ONLINE • 5 AGENTS')).toBeInTheDocument();

        rerender(<Org_Header pathname="/" connectionState="connecting" engineHealth={null} />);
        expect(screen.getByText('CONNECTING...')).toBeInTheDocument();

        rerender(<Org_Header pathname="/" connectionState="disconnected" engineHealth={null} />);
        expect(screen.getByText('ENGINE OFFLINE')).toBeInTheDocument();
    });

    it('renders the swarm status header and command palette hint', () => {
        render(<Org_Header pathname="/" connectionState="connected" engineHealth={null} />);
        
        expect(screen.getByTestId('swarm-status-header')).toBeInTheDocument();
        expect(screen.getByText('K')).toBeInTheDocument();
    });
});
