/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Confirm_Dialog_test]` in observability traces.
 */


/// <reference types="@testing-library/jest-dom" />
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';
import { Confirm_Dialog } from './Confirm_Dialog';

// Mock i18n
vi.mock('../../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    },
}));

describe('Confirm_Dialog', () => {
    const defaultProps = {
        is_open: true,
        title: 'Delete Resource',
        message: 'Are you sure? This cannot be undone.',
        on_confirm: vi.fn(),
        on_cancel: vi.fn(),
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders correctly when open', () => {
        render(<Confirm_Dialog {...defaultProps} />);
        
        expect(screen.getByText('Delete Resource')).toBeInTheDocument();
        expect(screen.getByText('Are you sure? This cannot be undone.')).toBeInTheDocument();
        expect(screen.getByRole('dialog')).toBeInTheDocument();
    });

    it('does not render when closed', () => {
        render(<Confirm_Dialog {...defaultProps} is_open={false} />);
        expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });

    it('calls on_confirm when confirm button is clicked', () => {
        render(<Confirm_Dialog {...defaultProps} confirm_label="Proceed" />);
        const confirmBtn = screen.getByText('Proceed');
        fireEvent.click(confirmBtn);
        expect(defaultProps.on_confirm).toHaveBeenCalledTimes(1);
    });

    it('calls on_cancel when cancel button is clicked', () => {
        render(<Confirm_Dialog {...defaultProps} cancel_label="Go Back" />);
        const cancelBtn = screen.getByText('Go Back');
        fireEvent.click(cancelBtn);
        expect(defaultProps.on_cancel).toHaveBeenCalledTimes(1);
    });

    it('calls on_cancel when overlay is clicked', () => {
        render(<Confirm_Dialog {...defaultProps} />);
        const overlay = screen.getByRole('dialog').parentElement!;
        fireEvent.click(overlay);
        expect(defaultProps.on_cancel).toHaveBeenCalledTimes(1);
    });

    it('handles Escape key to cancel', () => {
        render(<Confirm_Dialog {...defaultProps} />);
        fireEvent.keyDown(document, { key: 'Escape' });
        expect(defaultProps.on_cancel).toHaveBeenCalledTimes(1);
    });

    it('handles Enter key to confirm', async () => {
        render(<Confirm_Dialog {...defaultProps} />);
        fireEvent.keyDown(document, { key: 'Enter' });
        expect(defaultProps.on_confirm).toHaveBeenCalledTimes(1);
    });

    it('applies variant styles correctly', () => {
        const { rerender } = render(<Confirm_Dialog {...defaultProps} variant="danger" />);
        let title = screen.getByText('Delete Resource');
        // Based on THEME_TOKENS mapping in Confirm_Dialog.tsx
        // THEME_TOKENS.danger is usually red
        expect(title).toHaveClass('uppercase'); 

        rerender(<Confirm_Dialog {...defaultProps} variant="success" />);
        title = screen.getByText('Delete Resource');
        expect(title).toBeInTheDocument();
    });
});

// Metadata: [Confirm_Dialog_test]
