/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Hierarchy / Node_Task_Box.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { Node_Task_Box } from '../Node_Task_Box';
import type { Agent } from '../../../types';

describe('Node_Task_Box', () => {
    const mockAgent: Agent = {
        id: '1',
        name: 'Test Agent',
        status: 'idle',
        department: 'Operations',
        role: 'Assistant',
        tokens_used: 0,
        model: 'gemini-2.0-flash',
        category: 'core'
    } as Agent;

    it('renders idle status by default', () => {
        render(<Node_Task_Box agent={mockAgent} />);
        
        expect(screen.getByText(/System Idle/i)).toBeInTheDocument();
    });

    it('renders "Agent Not Active" when suspended', () => {
        const suspendedAgent = { ...mockAgent, status: 'suspended' as any };
        render(<Node_Task_Box agent={suspendedAgent} />);
        
        // Should show suspended label
        expect(screen.getByText(/Agent Not Active/i)).toBeInTheDocument();
        
        // Verify it has the correct color class
        const statusElement = screen.getByText(/Agent Not Active/i);
        expect(statusElement).toHaveClass('text-[color:var(--color-danger)]');
    });

    it('centers the status text', () => {
        render(<Node_Task_Box agent={mockAgent} />);
        
        // The element containing the text should have center classes
        const container = screen.getByText(/System Idle/i);
        expect(container).toHaveClass('flex', 'items-center', 'justify-center');
    });
});
