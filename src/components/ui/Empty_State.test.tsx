/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Ui / Empty_State.test
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
        expect(screen.getByLabelText(/icon/i)).toHaveTextContent('📭');
    });

    it('renders a custom string icon', () => {
        render(<Empty_State title="No Missions" icon="🔍" />);
        expect(screen.getByLabelText(/icon/i)).toHaveTextContent('🔍');
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
