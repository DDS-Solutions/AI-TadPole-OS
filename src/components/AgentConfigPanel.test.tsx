import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import AgentConfigPanel from './AgentConfigPanel';
import { TadpoleOSService } from '../services/tadpoleosService';
import { useProviderStore } from '../stores/providerStore';
import { useModelStore } from '../stores/modelStore';
import { useRoleStore } from '../stores/roleStore';
import { useSkillStore } from '../stores/skillStore';
import { useMemoryStore } from '../stores/memoryStore';
import { EventBus } from '../services/eventBus';
import type { Agent } from '../types';

// Mock dependencies
vi.mock('../services/tadpoleosService', () => ({
    TadpoleOSService: {
        getUnifiedSkills: vi.fn(),
        pauseAgent: vi.fn(),
        resumeAgent: vi.fn(),
        sendCommand: vi.fn(),
        getAgentMemory: vi.fn(),
        saveAgentMemory: vi.fn(),
        deleteAgentMemory: vi.fn(),
    }
}));

vi.mock('../services/eventBus', () => ({
    EventBus: {
        emit: vi.fn(),
        on: vi.fn(),
        off: vi.fn(),
    }
}));

vi.mock('../stores/providerStore', () => ({
    useProviderStore: vi.fn(),
}));

vi.mock('../stores/modelStore', () => ({
    useModelStore: vi.fn(),
}));

vi.mock('../stores/roleStore', () => ({
    useRoleStore: vi.fn(),
}));

vi.mock('../stores/skillStore', () => ({
    useSkillStore: vi.fn(),
}));

vi.mock('../stores/memoryStore', () => ({
    useMemoryStore: vi.fn(),
}));

const mockAgent = {
    id: 'agent-1',
    name: 'Test Agent',
    role: 'dev',
    status: 'active',
    model: 'gemini-1.5-flash',
    modelConfig: {
        modelId: 'gemini-1.5-flash',
        provider: 'google',
        temperature: 0.7,
        systemPrompt: 'You are a standard agent',
        skills: [],
        workflows: []
    },
    workflows: [],
    department: 'Engineering',
    tokensUsed: 0,
    category: 'ai',
} as Agent;

describe('AgentConfigPanel', () => {
    const mockOnClose = vi.fn();
    const mockOnUpdate = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();

        // Setup default store mocks
        (useProviderStore as unknown as ReturnType<typeof vi.fn>).mockImplementation((selector: (state: any) => any) => {
            const state = {
                providers: [{ id: 'google', name: 'Google' }, { id: 'openai', name: 'OpenAI' }],
            };
            return selector ? selector(state) : state;
        });

        (useModelStore as unknown as ReturnType<typeof vi.fn>).mockImplementation((selector: (state: any) => any) => {
            const state = {
                models: [
                    { id: 'gemini-1.5-flash', name: 'Gemini 1.5 Flash', provider: 'google' },
                    { id: 'gpt-4o', name: 'GPT-4o', provider: 'openai' }
                ],
            };
            return selector ? selector(state) : state;
        });

        (useRoleStore as unknown as ReturnType<typeof vi.fn>).mockImplementation((selector: (state: any) => any) => {
            const state = {
                roles: { dev: {}, pm: {} },
                addRole: vi.fn(),
            };
            return selector ? selector(state) : state;
        });

        (useSkillStore as unknown as ReturnType<typeof vi.fn>).mockImplementation((selector: (state: any) => any) => {
            const state = {
                scripts: [{ name: 'bash', description: 'Terminal access' }],
                manifests: [{ name: 'reasoning', display_name: 'Core Reasoning', description: 'Deep think' }],
                workflows: [{ name: 'deployment', description: 'Full deploy' }],
                mcpTools: [{ name: 'google_search', description: 'Search the web' }],
                fetchSkills: vi.fn(),
                fetchMcpTools: vi.fn(),
                isLoading: false,
            };
            if (selector) return selector(state);
            return state;
        });

        (useMemoryStore as unknown as ReturnType<typeof vi.fn>).mockImplementation(() => ({
            memories: [],
            isLoading: false,
            fetchMemories: vi.fn(),
            deleteMemory: vi.fn(),
            saveMemory: vi.fn(),
        }));

        (TadpoleOSService.getUnifiedSkills as unknown as ReturnType<typeof vi.fn>).mockResolvedValue({ scripts: [], manifests: [], workflows: [] });
        (TadpoleOSService.getAgentMemory as unknown as ReturnType<typeof vi.fn>).mockResolvedValue({ entries: [] });
    });

    it('renders correctly with an agent', () => {
        render(<AgentConfigPanel agent={mockAgent} onClose={mockOnClose} onUpdate={mockOnUpdate} />);
        
        // Should display the agent name in an input
        const nameInput = screen.getByDisplayValue('Test Agent');
        expect(nameInput).toBeInTheDocument();
        
        // Should display pause button for active agent
        expect(screen.getByLabelText(/SUSPEND LINK/i)).toBeInTheDocument();
    });

    it('calls onClose when close button is clicked', () => {
        render(<AgentConfigPanel agent={mockAgent} onClose={mockOnClose} onUpdate={mockOnUpdate} />);
        
        const backdrop = document.querySelector('.bg-black\\/60');
        if (backdrop) {
            fireEvent.click(backdrop);
            expect(mockOnClose).toHaveBeenCalled();
        }
    });

    it('changes to Memory tab', async () => {
        render(<AgentConfigPanel agent={mockAgent} onClose={mockOnClose} onUpdate={mockOnUpdate} />);
        
        const memoryTabBtn = screen.getByText(/LONG-TERM MEMORY/i);
        await act(async () => {
            fireEvent.click(memoryTabBtn);
        });
        
        expect(screen.getByText(/LANCEDB VECTOR STORE/i)).toBeInTheDocument();
    });

    it('can pause and resume agent', async () => {
        (TadpoleOSService.pauseAgent as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(true);
        (TadpoleOSService.resumeAgent as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(true);

        const { rerender } = render(<AgentConfigPanel agent={mockAgent} onClose={mockOnClose} onUpdate={mockOnUpdate} />);
        
        // Pause
        const pauseBtn = screen.getByLabelText('SUSPEND LINK');
        await act(async () => {
            fireEvent.click(pauseBtn);
        });
        
        // We wait for the promise to resolve and onUpdate to be called
        await vi.waitFor(() => {
            expect(TadpoleOSService.pauseAgent).toHaveBeenCalledWith('agent-1');
            expect(mockOnUpdate).toHaveBeenCalledWith('agent-1', { status: 'idle' });
        });

        // Rerender with idle status to test resume
        await act(async () => {
            rerender(<AgentConfigPanel agent={{ ...mockAgent, status: 'idle' }} onClose={mockOnClose} onUpdate={mockOnUpdate} />);
        });
        
        const resumeBtn = screen.getByLabelText('RESUME LINK');
        await act(async () => {
            fireEvent.click(resumeBtn);
        });
        
        await vi.waitFor(() => {
            expect(TadpoleOSService.resumeAgent).toHaveBeenCalledWith('agent-1');
            expect(mockOnUpdate).toHaveBeenCalledWith('agent-1', { status: 'active' });
        });
    });

    it('can send direct messages to the agent', async () => {
        (TadpoleOSService.sendCommand as any).mockResolvedValue(true);
        
        render(<AgentConfigPanel agent={mockAgent} onClose={mockOnClose} onUpdate={mockOnUpdate} />);
        
        const input = screen.getByPlaceholderText('TRANSMIT NEURAL INSTRUCTION...');
        fireEvent.change(input, { target: { value: 'Hello agent' } });
        
        // Send button is hard to query by text, but it's next to the input. We can use keydown Enter
        await act(async () => {
            fireEvent.keyDown(input, { key: 'Enter', code: 'Enter' });
        });
        
        await vi.waitFor(() => {
            expect(TadpoleOSService.sendCommand).toHaveBeenCalledWith('agent-1', 'Hello agent', expect.any(String), expect.any(String));
            expect(EventBus.emit).toHaveBeenCalled();
        });
    });

    it('renders all types of available skills (manifests, scripts, mcpTools)', () => {
        render(<AgentConfigPanel agent={mockAgent} onClose={mockOnClose} onUpdate={mockOnUpdate} />);
        
        // Check for manifest (uses display_name if available)
        expect(screen.getByText('Core Reasoning')).toBeInTheDocument();
        
        // Check for script
        expect(screen.getByText('bash')).toBeInTheDocument();
        expect(screen.getByText('Terminal access')).toBeInTheDocument();
        
        // Check for mcpTool
        expect(screen.getByText('google_search')).toBeInTheDocument();
        expect(screen.getByText('MCP')).toBeInTheDocument();
    });

    it('does not fetch if already loading', () => {
        const mockFetchSkills = vi.fn();
        const mockFetchMcpTools = vi.fn();

        (useSkillStore as unknown as ReturnType<typeof vi.fn>).mockImplementation((selector: (state: any) => any) => {
            const state = {
                manifests: [],
                mcpTools: [],
                scripts: [],
                workflows: [],
                fetchSkills: mockFetchSkills,
                fetchMcpTools: mockFetchMcpTools,
                isLoading: true, // Simulate already loading
            };
            if (selector) return selector(state);
            return state;
        });

        render(<AgentConfigPanel agent={mockAgent} onClose={mockOnClose} onUpdate={mockOnUpdate} />);

        expect(mockFetchSkills).not.toHaveBeenCalled();
        expect(mockFetchMcpTools).not.toHaveBeenCalled();
    });

    it('can toggle neural oversight in Governance tab', async () => {
        render(<AgentConfigPanel agent={mockAgent} onClose={mockOnClose} onUpdate={mockOnUpdate} />);
        
        // 1. Switch to Governance tab
        const govTabBtn = screen.getByText(/BUDGET & OVERSIGHT/i);
        await act(async () => {
            fireEvent.click(govTabBtn);
        });
        
        // 2. Locate and click the oversight toggle
        // The label is i18n.t('agent_config.label_oversight_gate') -> "NEURAL OVERSIGHT GATE"
        expect(screen.getByText(/NEURAL OVERSIGHT GATE/i)).toBeInTheDocument();
        
        // Find the toggle button - it's the only custom rounded-full button in the section
        const buttons = screen.getAllByRole('button');
        const govToggle = buttons.find(b => b.className.includes('rounded-full'));
        
        if (govToggle) {
            await act(async () => {
                fireEvent.click(govToggle);
            });
            
            // 3. Click Save (COMMIT CONFIGURATION)
            const saveBtn = screen.getByText(/COMMIT CONFIGURATION/i);
            await act(async () => {
                fireEvent.click(saveBtn);
            });
            
            expect(mockOnUpdate).toHaveBeenCalledWith('agent-1', expect.objectContaining({
                requiresOversight: true
            }));
        }
    });

    it('can update budget USD in Governance tab', async () => {
        render(<AgentConfigPanel agent={mockAgent} onClose={mockOnClose} onUpdate={mockOnUpdate} />);
        
        // 1. Switch to Governance tab
        const govTabBtn = screen.getByText(/BUDGET & OVERSIGHT/i);
        await act(async () => {
            fireEvent.click(govTabBtn);
        });
        
        // 2. Find the budget input
        const budgetInput = screen.getByDisplayValue('0');
        
        await act(async () => {
            fireEvent.change(budgetInput, { target: { value: '50.50' } });
            fireEvent.blur(budgetInput);
        });

        // 3. Click Save (COMMIT CONFIGURATION)
        const saveBtn = screen.getByText(/COMMIT CONFIGURATION/i);
        await act(async () => {
            fireEvent.click(saveBtn);
        });
        
        expect(mockOnUpdate).toHaveBeenCalledWith('agent-1', expect.objectContaining({
            budgetUsd: 50.5
        }));
    });
});
