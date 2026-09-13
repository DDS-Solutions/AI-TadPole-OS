/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Ui / Page_Header.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import React from 'react';
import { Page_Header } from './Page_Header';

import { useEngineStatus } from '../../hooks/use_engine_status';

// Mock useEngineStatus hook
vi.mock('../../hooks/use_engine_status', () => ({
    useEngineStatus: vi.fn(),
}));

// Mock Connection_Banner
vi.mock('./Connection_Banner', () => ({
    Connection_Banner: ({ state }: { state: string }) => <div data-testid="connection-banner" data-state={state} />,
}));

describe('Page_Header', () => {
    beforeEach(() => {
        vi.mocked(useEngineStatus).mockReturnValue({ status: 'connected' });
    });

    it('renders the title and subtitle correctly', () => {
        render(<Page_Header title="Dashboard" subtitle="Overview of your system" />);
        
        expect(screen.getByText('Dashboard')).toBeInTheDocument();
        expect(screen.getByText('Overview of your system')).toBeInTheDocument();
    });

    it('renders actions when provided', () => {
        render(
            <Page_Header 
                title="Dashboard" 
                actions={<button data-testid="new-mission">New Mission</button>} 
            />
        );
        
        expect(screen.getByTestId('new-mission')).toBeInTheDocument();
    });

    it('integrates with engine status to show banner', () => {
        vi.mocked(useEngineStatus).mockReturnValue({ status: 'disconnected' });
        
        render(<Page_Header title="Dashboard" />);
        
        const banner = screen.getByTestId('connection-banner');
        expect(banner).toHaveAttribute('data-state', 'disconnected');
    });
});
