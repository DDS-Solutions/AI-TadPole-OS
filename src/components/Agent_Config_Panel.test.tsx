/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Manages the interface for Agent Config Panel.test. 
 * Part of the Tadpole-OS Modular UI ecosystem.
 */

/**
 * @file Agent_Config_Panel.test.tsx
 * @description Suite for the Agent Configuration Panel component.
 * @module Components/Agent_Config_Panel
 * @testedBehavior
 * - Rendering: Displays agent details from the provided agent object.
 * - Interaction: Triggers pause/resume actions and calls on_update.
 * - Validation: Ensures correct payload is sent to tadpole_os_service.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks tadpole_os_service for agent state transitions (pause/resume).
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import '@testing-library/jest-dom';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import Agent_Config_Panel from './Agent_Config_Panel';
import { tadpole_os_service } from '../services/tadpoleos_service';
import type { Agent, Agent_Status } from '../types';

// Mock tadpole_os_service
vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        pause_agent: vi.fn().mockResolvedValue({ success: true }),
        resume_agent: vi.fn().mockResolvedValue({ success: true }),
        update_agent: vi.fn().mockResolvedValue({ success: true }),
        get_agent_memory: vi.fn().mockResolvedValue({ entries: [] }),
    }
}));

// Mock i18n
vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string) => {
            if (key === 'agent_config.btn_pause') return 'SUSPEND LINK';
            if (key === 'agent_config.btn_resume') return 'RESUME LINK';
            return key;
        },
    },
}));

describe('Agent_Config_Panel', () => {
    const mock_agent: Agent = {
        id: 'agent-1',
        name: 'Test Agent',
        status: 'idle' as Agent_Status,
        role: 'CEO',
        department: 'Operations',
        tokens_used: 100,
        model: 'gemini-2.0-flash',
        category: 'core',
        model_config: {
            provider: 'google',
            model_id: 'gemini-2.0-flash',
            temperature: 0.7,
            system_prompt: '',
            skills: [],
            workflows: []
        }
    } as any;

    const mock_on_update = vi.fn();
    const mock_on_close = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders agent details correctly', () => {
        render(<Agent_Config_Panel agent={mock_agent} on_update={mock_on_update} on_close={mock_on_close} />);
        
        // Name is in an input value
        expect(screen.getByDisplayValue('Test Agent')).toBeInTheDocument();
        // Role is displayed
        expect(screen.getByText('CEO')).toBeInTheDocument();
    });

    it('can pause and resume agent', async () => {
        const { rerender } = render(<Agent_Config_Panel agent={mock_agent} on_update={mock_on_update} on_close={mock_on_close} />);
        
        // Pause button
        const pause_button = screen.getByLabelText('SUSPEND LINK');
        fireEvent.click(pause_button);

        await waitFor(() => {
            expect(tadpole_os_service.pause_agent).toHaveBeenCalledWith('agent-1');
            expect(mock_on_update).toHaveBeenCalledWith('agent-1', expect.objectContaining({ status: 'suspended' }));
        });

        // Simulating the update from parent
        const suspended_agent = { ...mock_agent, status: 'suspended' as Agent_Status };
        rerender(<Agent_Config_Panel agent={suspended_agent} on_update={mock_on_update} on_close={mock_on_close} />);

        // Resume button
        const resume_button = screen.getByLabelText('RESUME LINK');
        fireEvent.click(resume_button);

        await waitFor(() => {
            expect(tadpole_os_service.resume_agent).toHaveBeenCalledWith('agent-1');
            expect(mock_on_update).toHaveBeenCalledWith('agent-1', expect.objectContaining({ status: 'idle' }));
        });
    });
});

