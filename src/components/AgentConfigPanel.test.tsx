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
                scripts: [{ name: 'bash' }],
                manifests: [],
                workflows: [{ name: 'deployment' }],
                mcpTools: [],
                fetchSkills: vi.fn(),
                fetchMcpTools: vi.fn(),
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
    });

    it('renders correctly with an agent', () => {
        render(<AgentConfigPanel agent={mockAgent} onClose={mockOnClose} onUpdate={mockOnUpdate} />);
        
        // Should display the agent name in an input
        const nameInput = screen.getByDisplayValue('Test Agent');
        expect(nameInput).toBeInTheDocument();
        
        // Should display pause button for active agent
        expect(screen.getByText(/Pause/i)).toBeInTheDocument();
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
        const pauseBtn = screen.getByText('Pause');
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
        
        const resumeBtn = screen.getByText('Resume');
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
        
        const input = screen.getByPlaceholderText('Send instruction...');
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
});
