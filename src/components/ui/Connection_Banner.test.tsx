/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Ui / Connection_Banner.test
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
import { Connection_Banner } from './Connection_Banner';

// Mock i18n
vi.mock('../../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    },
}));

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
    WifiOff: () => <div data-testid="wifi-off-icon" />,
    AlertTriangle: () => <div data-testid="alert-triangle-icon" />,
}));

describe('Connection_Banner', () => {
    it('renders nothing when connected', () => {
        const { container } = render(<Connection_Banner state="connected" />);
        expect(container.firstChild).toBeNull();
    });

    it('renders nothing when connecting', () => {
        const { container } = render(<Connection_Banner state="connecting" />);
        expect(container.firstChild).toBeNull();
    });

    it('renders disconnected message when disconnected', () => {
        render(<Connection_Banner state="disconnected" />);
        expect(screen.getByText('system.disconnected')).toBeInTheDocument();
        expect(screen.getByTestId('wifi-off-icon')).toBeInTheDocument();
    });

    it('renders error message when in error state', () => {
        render(<Connection_Banner state="error" />);
        expect(screen.getByText('system.connection_error')).toBeInTheDocument();
        expect(screen.getByTestId('alert-triangle-icon')).toBeInTheDocument();
    });

    it('applies basic positioning styles', () => {
        render(<Connection_Banner state="error" />);
        const banner = screen.getByTestId('connection-banner');
        expect(banner).toHaveClass('absolute');
        expect(banner).toHaveClass('top-0');
    });
});
