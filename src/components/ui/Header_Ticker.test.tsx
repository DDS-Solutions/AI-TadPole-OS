/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Header_Ticker_test]` in observability traces.
 */


import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import React from 'react';
import { Header_Ticker } from './Header_Ticker';

describe('Header_Ticker', () => {
    it('renders children correctly', () => {
        render(
            <Header_Ticker>
                <span data-testid="ticker-item">System Operational</span>
            </Header_Ticker>
        );
        
        // Should find multiple instances due to cloning strategy (3 in this case)
        const items = screen.getAllByTestId('ticker-item');
        expect(items.length).toBeGreaterThanOrEqual(1);
        expect(items[0]).toHaveTextContent('System Operational');
    });

    it('applies custom duration style', () => {
        const { container } = render(
            <Header_Ticker duration={60}>
                <span>Test</span>
            </Header_Ticker>
        );
        
        const marqueeDiv = container.querySelector('.animate-marquee') as HTMLElement;
        expect(marqueeDiv.style.getPropertyValue('--duration')).toBe('60s');
    });

    it('contains accessibility clones with aria-hidden', () => {
        const { container } = render(
            <Header_Ticker>
                <span>Test</span>
            </Header_Ticker>
        );
        
        const hiddenClones = container.querySelectorAll('[aria-hidden="true"]');
        expect(hiddenClones.length).toBe(2); // Based on the "triple-clone strategy" (1 original + 2 aria-hidden clones)
    });
});

// Metadata: [Header_Ticker_test]
