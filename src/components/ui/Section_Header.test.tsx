/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Ui / Section_Header.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import React from 'react';
import { Section_Header } from './Section_Header';

describe('Section_Header', () => {
    it('renders the title correctly', () => {
        render(<Section_Header title="System Status" />);
        expect(screen.getByText('System Status')).toBeInTheDocument();
    });

    it('renders subtitle when provided', () => {
        render(<Section_Header title="System Status" subtitle="Real-time telemetry" />);
        expect(screen.getByText('Real-time telemetry')).toBeInTheDocument();
    });

    it('renders badge when provided', () => {
        render(<Section_Header title="Active Agents" badge={5} />);
        expect(screen.getByText('5')).toBeInTheDocument();
    });

    it('renders string icon correctly', () => {
        render(<Section_Header title="Logs" icon="📄" />);
        expect(screen.getByLabelText(/icon/i)).toHaveTextContent('📄');
    });

    it('renders ReactNode icon correctly', () => {
        render(<Section_Header title="Settings" icon={<span data-testid="custom-icon">⚙️</span>} />);
        expect(screen.getByTestId('custom-icon')).toBeInTheDocument();
    });

    it('renders action element correctly', () => {
        render(
            <Section_Header 
                title="Files" 
                action={<button data-testid="action-btn">Upload</button>} 
            />
        );
        expect(screen.getByTestId('action-btn')).toBeInTheDocument();
    });
});
