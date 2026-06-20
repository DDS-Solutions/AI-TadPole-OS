/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Integrated verification of Agent configuration state transitions**, including pause/resume signaling and i18n label mapping. 
 * Mocks `tadpole_os_service` to validate cross-component update callbacks and agent memory hydration.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Mismatched vitest mocks for `tadpole_os_service` or incorrect i18n key resolution causing label mismatches.
 * - **Telemetry Link**: Search `[AgentConfigPanel_test]` in tracing logs.
 */

import '@testing-library/jest-dom';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import AgentConfigPanel from './AgentConfigPanel';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { event_bus } from '../services/event_bus';
import type { Agent, Agent_Status } from '../types';

// Mock tadpole_os_service
vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        pause_agent: vi.fn().mockResolvedValue({ success: true }),
        resume_agent: vi.fn().mockResolvedValue({ success: true }),
        update_agent: vi.fn().mockResolvedValue({ success: true }),
        get_agent_memory: vi.fn().mockResolvedValue({ entries: [] }),
        save_agent_memory: vi.fn().mockResolvedValue({ success: true }),
        delete_agent_memory: vi.fn().mockResolvedValue({ success: true }),
    }
}));

// Mock event_bus
vi.mock('../services/event_bus', () => ({
    event_bus: {
        emit_log: vi.fn(),
        subscribe_logs: vi.fn(() => () => {}),
        subscribe_traces: vi.fn(() => () => {}),
        get_history: vi.fn(() => []),
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

// Mock Zustand state variables for stores
const mock_skill_store_state = {
    manifests: [],
    scripts: [],
    workflows: [],
    hooks: [],
    mcp_tools: [],
    is_loading: false,
    initialized_skills: true,
    initialized_mcp: true,
    error: null,
    fetch_skills: vi.fn().mockResolvedValue(undefined),
    fetch_mcp_tools: vi.fn().mockResolvedValue(undefined),
};

const mock_provider_store_state = {
    providers: [],
};

const mock_model_store_state = {
    models: [],
};

const mock_role_store_state = {
    roles: {
        CEO: {
            id: 'CEO',
            name: 'CEO',
            department: 'Executive',
            description: 'CEO role description',
            skills: '[]',
            workflows: '[]',
            mcp_tools: '[]',
        }
    },
};

// Mock Zustand stores
vi.mock('../stores/skill_store', () => ({
    use_skill_store: vi.fn((selector) => {
        if (typeof selector === 'function') {
            return selector(mock_skill_store_state);
        }
        return mock_skill_store_state;
    }),
}));

vi.mock('../stores/provider_store', () => ({
    use_provider_store: vi.fn((selector) => {
        if (typeof selector === 'function') {
            return selector(mock_provider_store_state);
        }
        return mock_provider_store_state;
    }),
}));

vi.mock('../stores/model_store', () => ({
    use_model_store: vi.fn((selector) => {
        if (typeof selector === 'function') {
            return selector(mock_model_store_state);
        }
        return mock_model_store_state;
    }),
}));

vi.mock('../stores/role_store', () => ({
    use_role_store: vi.fn((selector) => {
        if (typeof selector === 'function') {
            return selector(mock_role_store_state);
        }
        return mock_role_store_state;
    }),
}));

describe('AgentConfigPanel', () => {
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
        render(<AgentConfigPanel agent={mock_agent} onUpdate={mock_on_update} onClose={mock_on_close} />);
        
        // Name is in an input value
        expect(screen.getByDisplayValue('Test Agent')).toBeInTheDocument();
        // Role is displayed
        expect(screen.getByText('CEO')).toBeInTheDocument();
    });

    it('can pause and resume agent', async () => {
        const { rerender } = render(<AgentConfigPanel agent={mock_agent} onUpdate={mock_on_update} onClose={mock_on_close} />);
        
        // Pause button
        const pause_button = screen.getByLabelText('SUSPEND LINK');
        fireEvent.click(pause_button);

        await waitFor(() => {
            expect(tadpole_os_service.pause_agent).toHaveBeenCalledWith('agent-1');
            expect(mock_on_update).toHaveBeenCalledWith('agent-1', expect.objectContaining({ status: 'suspended' }));
        });

        // Simulating the update from parent
        const suspended_agent = { ...mock_agent, status: 'suspended' as Agent_Status };
        rerender(<AgentConfigPanel agent={suspended_agent} onUpdate={mock_on_update} onClose={mock_on_close} />);

        // Resume button
        const resume_button = screen.getByLabelText('RESUME LINK');
        fireEvent.click(resume_button);

        await waitFor(() => {
            expect(tadpole_os_service.resume_agent).toHaveBeenCalledWith('agent-1');
            expect(mock_on_update).toHaveBeenCalledWith('agent-1', expect.objectContaining({ status: 'idle' }));
        });
    });

    it('reactively synchronizes model and slot changes from parent agent prop', async () => {
        const { rerender } = render(<AgentConfigPanel agent={mock_agent} onUpdate={mock_on_update} onClose={mock_on_close} />);
        
        // Simulating external update of the primary model, active slot and system prompt
        const updated_agent = {
            ...mock_agent,
            model: 'gpt-4-turbo',
            model_config: {
                ...mock_agent.model_config,
                modelId: 'gpt-4-turbo',
                provider: 'openai',
                temperature: 0.9,
                systemPrompt: 'New Prompt'
            }
        } as any;
        
        rerender(<AgentConfigPanel agent={updated_agent} onUpdate={mock_on_update} onClose={mock_on_close} />);
        
        // Assert that the text area with the new system prompt is rendered with the updated value
        expect(screen.getByDisplayValue('New Prompt')).toBeInTheDocument();
    });

    it('should handle vector memory loading failure and alert the user', async () => {
        vi.mocked(tadpole_os_service.get_agent_memory).mockRejectedValueOnce(new Error('API 500 Failure'));
        
        render(<AgentConfigPanel agent={mock_agent} onUpdate={mock_on_update} onClose={mock_on_close} />);
        
        // Toggle to memory tab
        const memory_tab = screen.getByText('agent_config.tab_memory');
        fireEvent.click(memory_tab);

        await waitFor(() => {
            expect(tadpole_os_service.get_agent_memory).toHaveBeenCalledWith('agent-1');
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                text: 'agent_config.memory_load_failed',
                severity: 'error'
            }));
        });
    });

    it('should log and display feedback when saving memory entry succeeds', async () => {
        render(<AgentConfigPanel agent={mock_agent} onUpdate={mock_on_update} onClose={mock_on_close} />);
        
        // Toggle to memory tab
        const memory_tab = screen.getByText('agent_config.tab_memory');
        fireEvent.click(memory_tab);

        // Fill memory input and click save
        const input = screen.getByPlaceholderText('agent_config.placeholder_memory_injection');
        fireEvent.change(input, { target: { value: 'This is a test memory.' } });
        
        const save_button = screen.getByLabelText('agent_config.aria_save_memory');
        fireEvent.click(save_button);

        await waitFor(() => {
            expect(tadpole_os_service.save_agent_memory).toHaveBeenCalledWith('agent-1', 'This is a test memory.');
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                text: 'agent_config.memory_saved',
                severity: 'success'
            }));
        });
    });

    it('should alert when a FileSystem connector is registered', async () => {
        render(<AgentConfigPanel agent={mock_agent} onUpdate={mock_on_update} onClose={mock_on_close} />);
        
        // Toggle to memory tab
        const memory_tab = screen.getByText('agent_config.tab_memory');
        fireEvent.click(memory_tab);

        // Fill path and click add connector
        const input = screen.getByPlaceholderText('memory_section.placeholder_local_path');
        fireEvent.change(input, { target: { value: '/test/path' } });

        const add_button = screen.getByRole('button', { name: 'memory_section.btn_add_source' });
        fireEvent.click(add_button);

        await waitFor(() => {
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                text: 'agent_config.connector_added',
                severity: 'info'
            }));
        });
    });
});

// Metadata: [AgentConfigPanel_test]
