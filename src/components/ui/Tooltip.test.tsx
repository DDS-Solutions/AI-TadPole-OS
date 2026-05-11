/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Tooltip_test]` in observability traces.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { Tooltip } from './Tooltip';
import React from 'react';

// Mock the hook because it relies on DOM measurement that JSDOM might not handle perfectly
vi.mock('../../hooks/use_viewport_position', () => ({
    useViewportPosition: () => ({
        coords: { x: 100, y: 100 },
        actual_position: 'top',
        update_position: vi.fn(),
    }),
}));

// Mock framer-motion to render children immediately
vi.mock('framer-motion', () => ({
    motion: {
        div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
    },
    AnimatePresence: ({ children }: any) => <>{children}</>,
}));

describe('Tooltip', () => {
    beforeEach(() => {
        vi.useFakeTimers();
        // Ensure portal-root exists if the component tries to use it
        if (!document.getElementById('portal-root')) {
            const root = document.createElement('div');
            root.setAttribute('id', 'portal-root');
            document.body.appendChild(root);
        }
    });

    it('does not show tooltip by default', () => {
        render(
            <Tooltip content="Tooltip Content">
                <button>Hover me</button>
            </Tooltip>
        );
        expect(screen.queryByRole('tooltip')).toBeNull();
    });

    it('shows tooltip after delay on mouse enter', async () => {
        render(
            <Tooltip content="Tooltip Content" delay={100}>
                <button>Hover me</button>
            </Tooltip>
        );

        const trigger = screen.getByText('Hover me');
        fireEvent.mouseEnter(trigger);

        // Should not be visible yet
        expect(screen.queryByRole('tooltip')).toBeNull();

        // Fast-forward time
        act(() => {
            vi.advanceTimersByTime(150);
        });

        expect(screen.getByRole('tooltip')).toBeDefined();
        expect(screen.getByText('Tooltip Content')).toBeDefined();
    });

    it('hides tooltip on mouse leave', () => {
        render(
            <Tooltip content="Tooltip Content" delay={0}>
                <button>Hover me</button>
            </Tooltip>
        );

        const trigger = screen.getByText('Hover me');
        
        act(() => {
            fireEvent.mouseEnter(trigger);
            vi.runAllTimers();
        });
        expect(screen.getByRole('tooltip')).toBeDefined();

        act(() => {
            fireEvent.mouseLeave(trigger);
        });
        expect(screen.queryByRole('tooltip')).toBeNull();
    });
});

// Metadata: [Tooltip_test]
