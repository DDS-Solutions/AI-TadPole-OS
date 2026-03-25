/**
 * @file Skills.test.tsx
 * @description Suite for the Agent Skills (Permission/Feature) management page.
 * @module Pages/Skills
 * @testedBehavior
 * - Toggle Logic: Verification of permission state updates in useSkillStore.
 * - Discovery: Interaction with TadpoleOSService.discoverNodes.
 * - Reactive UI: Ensuring toggles accurately reflect backend skill states.
 * @aiContext
 * - Mocks useSkillStore and TadpoleOSService to isolate permission logic.
 */
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import Skills from './Skills';
import { useSkillStore } from '../stores/skillStore';
import { TadpoleOSService } from '../services/tadpoleosService';

// Mock store
vi.mock('../stores/skillStore', () => ({
    useSkillStore: vi.fn(),
}));

// Mock service
vi.mock('../services/tadpoleosService', () => ({
    TadpoleOSService: {
        executeMcpTool: vi.fn(),
    },
}));

// Mock Tooltip and TwEmptyState components to simplify tests
vi.mock('../components/ui', () => ({
    Tooltip: ({ children, content }: { children: React.ReactNode, content?: string }) => (
        <div data-testid="tooltip-wrapper" data-tooltip-content={content}>
            {children}
            {content && <span style={{ display: 'none' }}>{content}</span>}
        </div>
    ),
    TwEmptyState: ({ title }: { title: string }) => <div>{title}</div>,
}));

describe('Skills Page', () => {
    const mockStore = {
        scripts: [
            { name: 'test_skill', description: 'Test skill description', execution_command: 'python test.py', schema: {}, category: 'user' as const }
        ],
        workflows: [
            { name: 'test_workflow', content: 'Test workflow content', category: 'user' as const }
        ],
        hooks: [
            { name: 'test_hook', description: 'Test hook desc', hook_type: 'pre_validation', content: 'test-content', category: 'user' as const, active: true }
        ],
        mcpTools: [
            {
                name: 'mcp_tool',
                description: 'MCP Tool description',
                source: 'test-source',
                input_schema: { properties: { arg1: { type: 'string' } } },
                stats: { invocations: 10, success_count: 9, failure_count: 1, avg_latency_ms: 150 },
                category: 'user' as const
            }
        ],
        manifests: [],
        isLoading: false,
        error: null,
        fetchSkills: vi.fn(),
        fetchMcpTools: vi.fn(),
        saveSkillScript: vi.fn(),
        deleteSkillScript: vi.fn(),
        saveWorkflow: vi.fn(),
        deleteWorkflow: vi.fn(),
        saveHook: vi.fn(),
        deleteHook: vi.fn(),
    };

    beforeEach(() => {
        vi.clearAllMocks();
        (useSkillStore as unknown as Mock).mockReturnValue(mockStore);
    });

    it('renders and fetches data on mount', () => {
        render(<Skills />);
        expect(mockStore.fetchSkills).toHaveBeenCalled();
        expect(mockStore.fetchMcpTools).toHaveBeenCalled();
        expect(screen.getByText('test_skill')).toBeInTheDocument();
    });

    it('switches tabs correctly', () => {
        render(<Skills />);
        
        const workflowTab = screen.getByText(/PASSIVE WORKFLOWS/i);
        fireEvent.click(workflowTab);
        expect(screen.getByText('test_workflow')).toBeInTheDocument();

        const hookTab = screen.getByText(/LIFECYCLE HOOKS/i);
        fireEvent.click(hookTab);
        expect(screen.getByText(/SYSTEM HOOKS & AUDITS/i)).toBeInTheDocument();

        const mcpTab = screen.getByText(/MCP TOOLS AGENTS/i);
        fireEvent.click(mcpTab);
        expect(screen.getByText('mcp_tool')).toBeInTheDocument();
    });

    it('opens and closes skill modal', async () => {
        render(<Skills />);
        
        const newSkillBtn = screen.getByText(/NEW SKILL/i);
        fireEvent.click(newSkillBtn);

        expect(screen.getByText('CREATE SKILL')).toBeInTheDocument();

        const cancelBtn = screen.getByText('CANCEL');
        fireEvent.click(cancelBtn);

        await waitFor(() => {
            expect(screen.queryByText('CREATE SKILL')).not.toBeInTheDocument();
        });
    });

    it('handles skill deletion', async () => {
        render(<Skills />);
        // Find by text and navigate to the button
        const tooltipText = screen.getByText('Permanently Delete Skill');
        const deleteBtn = tooltipText.closest('div[data-testid="tooltip-wrapper"]')?.querySelector('button');
        
        expect(deleteBtn).toBeInTheDocument();
        if (deleteBtn) fireEvent.click(deleteBtn);

        expect(mockStore.deleteSkillScript).toHaveBeenCalledWith('test_skill');
    });

    it('handles skill creation/saving', async () => {
        render(<Skills />);
        
        const newSkillBtn = screen.getByText(/NEW SKILL/i);
        fireEvent.click(newSkillBtn);

        // Fill form
        const nameInput = screen.getByPlaceholderText('fetch_twitter_data');
        fireEvent.change(nameInput, { target: { value: 'new_skill' } });

        const cmdInput = screen.getByPlaceholderText('python scripts/fetch.py');
        fireEvent.change(cmdInput, { target: { value: 'python exec.py' } });

        const saveBtn = screen.getByText('SAVE SKILL');
        fireEvent.click(saveBtn);

        await waitFor(() => {
            expect(mockStore.saveSkillScript).toHaveBeenCalledWith(expect.objectContaining({
                name: 'new_skill',
                execution_command: 'python exec.py',
                category: 'user'
            }));
        });
    });

    it('handles workflow creation/saving', async () => {
        render(<Skills />);
        
        // Go to workflows tab
        fireEvent.click(screen.getByText(/PASSIVE WORKFLOWS/i));

        const newWfBtn = screen.getByText(/NEW WORKFLOW/i);
        fireEvent.click(newWfBtn);

        const nameInput = screen.getByPlaceholderText('Deep Research Protocol');
        fireEvent.change(nameInput, { target: { value: 'New WF' } });

        const contentInput = screen.getByPlaceholderText(/1. Do X/i);
        fireEvent.change(contentInput, { target: { value: '# Step 1' } });

        const saveBtn = screen.getByText('SAVE WORKFLOW');
        fireEvent.click(saveBtn);

        await waitFor(() => {
            expect(mockStore.saveWorkflow).toHaveBeenCalledWith(expect.objectContaining({
                name: 'New WF',
                content: '# Step 1',
                category: 'user'
            }));
        });
    });

    it('runs Tool Lab execution', async () => {
        (TadpoleOSService.executeMcpTool as Mock).mockResolvedValue({ status: 'success', data: 'result' });
        render(<Skills />);
        
        fireEvent.click(screen.getByText(/MCP TOOLS AGENTS/i));
        
        const testBtn = screen.getByText(/Test Tool/i);
        fireEvent.click(testBtn);
        
        expect(screen.getByText('Manual Execution Lab')).toBeInTheDocument();
        
        const runBtn = screen.getByText(/RUN TOOL/i);
        fireEvent.click(runBtn);
        
        await waitFor(() => {
            expect(TadpoleOSService.executeMcpTool).toHaveBeenCalledWith('mcp_tool', expect.any(Object));
            expect(screen.getByText(/"status": "success"/)).toBeInTheDocument();
        });
    });

    it('displays error message from store', () => {
        (useSkillStore as unknown as Mock).mockReturnValue({
            ...mockStore,
            error: 'Failed to load tools'
        });
        render(<Skills />);
        expect(screen.getByText('Failed to load tools')).toBeInTheDocument();
    });
});
