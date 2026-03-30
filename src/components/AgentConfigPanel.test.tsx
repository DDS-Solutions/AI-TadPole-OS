import '@testing-library/jest-dom';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import AgentConfigPanel from './AgentConfigPanel';
import { TadpoleOSService } from '../services/tadpoleosService';
import type { Agent, AgentStatus } from '../types';

// Mock TadpoleOSService
vi.mock('../services/tadpoleosService', () => ({
    TadpoleOSService: {
        pauseAgent: vi.fn().mockResolvedValue({ success: true }),
        resumeAgent: vi.fn().mockResolvedValue({ success: true }),
        updateAgentConfig: vi.fn().mockResolvedValue({ success: true }),
    }
}));

describe('AgentConfigPanel', () => {
    const mockAgent: Agent = {
        id: 'agent-1',
        name: 'Test Agent',
        status: 'idle' as AgentStatus,
        isSuspended: false,
        role: 'CEO' as any,
        department: 'Operations',
        tokensUsed: 100,
        model: 'gemini-2.0-flash',
        category: 'core'
    } as any; // Cast to any to avoid complex nested Agent type requirements in test

    const mockOnUpdate = vi.fn();
    const mockOnClose = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders agent details correctly', () => {
        render(<AgentConfigPanel agent={mockAgent} onUpdate={mockOnUpdate} onClose={mockOnClose} />);
        
        // Name is in an input value
        expect(screen.getByDisplayValue('Test Agent')).toBeInTheDocument();
        // Role is in a select or text
        expect(screen.getByText('CEO')).toBeInTheDocument(); // Select options are uppercased in header

    });

    it('can pause and resume agent', async () => {
        const { rerender } = render(<AgentConfigPanel agent={mockAgent} onUpdate={mockOnUpdate} onClose={mockOnClose} />);
        
        // Button uses aria-label and title from i18n keys agent_config.btn_pause -> "SUSPEND LINK"
        const pauseButton = screen.getByLabelText('SUSPEND LINK');
        fireEvent.click(pauseButton);

        await waitFor(() => {
            expect(TadpoleOSService.pauseAgent).toHaveBeenCalledWith('agent-1');
            expect(mockOnUpdate).toHaveBeenCalledWith('agent-1', { status: 'suspended' as AgentStatus });
        });

        // Simulating the update from parent
        const suspendedAgent = { ...mockAgent, status: 'suspended' as AgentStatus, isSuspended: true };
        rerender(<AgentConfigPanel agent={suspendedAgent} onUpdate={mockOnUpdate} onClose={mockOnClose} />);

        // Button now shows Resume -> "RESUME LINK"
        const resumeButton = screen.getByLabelText('RESUME LINK');
        fireEvent.click(resumeButton);

        await waitFor(() => {
            expect(TadpoleOSService.resumeAgent).toHaveBeenCalledWith('agent-1');
            expect(mockOnUpdate).toHaveBeenCalledWith('agent-1', { status: 'idle' as AgentStatus });
        });
    });
});
