/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification of the Node Task Box's dynamic rendering** of agent missions and task logs. 
 * Ensures correct status color mapping and mission progress visualization for the Knowledge Graph.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Incorrect status hex-code resolution or overflow issues in long-form task descriptions.
 * - **Telemetry Link**: Search `[Node_Task_Box.test]` in tracing logs.
 */


import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { Node_Task_Box } from '../Node_Task_Box';
import type { Agent } from '../../../types';

describe('Node_Task_Box', () => {
    const mockAgent: Agent = {
        id: '1',
        name: 'Test Agent',
        status: 'active',
        isSuspended: false,
        department: 'Operations',
        role: 'Assistant',
        tokens_used: 0,
        model: 'gemini-2.0-flash',
        category: 'core'
    } as Agent;

    it('renders idle status by default', () => {
        render(<Node_Task_Box agent={mockAgent} />);
        
        expect(screen.getByText('System Idle • Standing By...')).toBeInTheDocument();
    });

    it('renders "Agent Not Active" when suspended', () => {
        const suspendedAgent = { ...mockAgent, isSuspended: true, status: 'suspended' as any };
        render(<Node_Task_Box agent={suspendedAgent} />);
        
        // Should show suspended label
        expect(screen.getByText('Agent Not Active • Link Deactivated')).toBeInTheDocument();
        
        // Verify it has the correct color class
        const statusElement = screen.getByText('Agent Not Active • Link Deactivated');
        expect(statusElement).toHaveClass('text-rose-400');
    });

    it('centers the status text', () => {
        render(<Node_Task_Box agent={mockAgent} />);
        
        // The element containing the text should have center classes
        const container = screen.getByText('System Idle • Standing By...');
        expect(container).toHaveClass('flex', 'items-center', 'justify-center');
    });
});

