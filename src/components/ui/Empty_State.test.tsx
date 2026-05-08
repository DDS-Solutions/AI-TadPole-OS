/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Empty_State_test]` in observability traces.
 */


import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';
import { Empty_State } from './Empty_State';

describe('Empty_State', () => {
    it('renders the title and description correctly', () => {
        render(<Empty_State title="No Missions" description="Start a new mission to see data." />);
        
        expect(screen.getByText('No Missions')).toBeInTheDocument();
        expect(screen.getByText('Start a new mission to see data.')).toBeInTheDocument();
    });

    it('renders the default icon', () => {
        render(<Empty_State title="No Missions" />);
        expect(screen.getByLabelText('icon')).toHaveTextContent('📭');
    });

    it('renders a custom string icon', () => {
        render(<Empty_State title="No Missions" icon="🔍" />);
        expect(screen.getByLabelText('icon')).toHaveTextContent('🔍');
    });

    it('renders a custom ReactNode icon', () => {
        render(<Empty_State title="No Missions" icon={<span data-testid="custom-icon">✨</span>} />);
        expect(screen.getByTestId('custom-icon')).toBeInTheDocument();
    });

    it('renders action object correctly and handles clicks', () => {
        const onClick = vi.fn();
        render(
            <Empty_State 
                title="No Missions" 
                action={{ label: 'Create New', onClick }} 
            />
        );
        
        const btn = screen.getByText('Create New');
        fireEvent.click(btn);
        expect(onClick).toHaveBeenCalledTimes(1);
    });

    it('renders ReactNode action correctly', () => {
        render(
            <Empty_State 
                title="No Missions" 
                action={<button data-testid="custom-action">Custom Action</button>} 
            />
        );
        
        expect(screen.getByTestId('custom-action')).toBeInTheDocument();
    });

    it('applies dashed variant styles', () => {
        const { container } = render(<Empty_State title="Empty" variant="dashed" />);
        const div = container.firstChild as HTMLElement;
        expect(div).toHaveClass('border-dashed');
    });
});

// Metadata: [Empty_State_test]
