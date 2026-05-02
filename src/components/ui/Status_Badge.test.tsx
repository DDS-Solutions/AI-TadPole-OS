/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Status_Badge_test]` in observability traces.
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Status_Badge } from './Status_Badge';
import React from 'react';

describe('Status_Badge', () => {
    it('renders with the correct status text', () => {
        render(<Status_Badge status="active" />);
        expect(screen.getByText('active')).toBeDefined();
    });

    it('applies the correct classes for active status', () => {
        const { container } = render(<Status_Badge status="active" />);
        const badge = container.firstChild as HTMLElement;
        // active status maps to success theme tokens
        expect(badge.className).toContain('color-success-text');
    });

    it('applies the correct classes for error status', () => {
        const { container } = render(<Status_Badge status="error" />);
        const badge = container.firstChild as HTMLElement;
        // error status maps to danger theme tokens
        expect(badge.className).toContain('color-danger-text');
    });

    it('renders different sizes', () => {
        const { rerender, container } = render(<Status_Badge status="idle" size="sm" />);
        let badge = container.firstChild as HTMLElement;
        expect(badge.className).toContain('text-[9px]');

        rerender(<Status_Badge status="idle" size="lg" />);
        badge = container.firstChild as HTMLElement;
        expect(badge.className).toContain('text-[11px]');
    });

    it('enables pulse animation when requested', () => {
        const { container } = render(<Status_Badge status="thinking" pulse={true} />);
        const dot = container.querySelector('.rounded-full');
        expect(dot?.className).toContain('animate-pulse');
    });

    it('falls back to idle for unknown status', () => {
        render(<Status_Badge status="unknown_status" />);
        expect(screen.getByText('unknown_status')).toBeDefined();
    });
});

// Metadata: [Status_Badge_test]
