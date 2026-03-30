/**
 * @file ScheduledJobs.test.tsx
 * @description Suite for the Continuity Scheduler and Managed Recurring Tasks page.
 * @module Pages/ScheduledJobs
 * @testedBehavior
 * - Job Inventory: Correct rendering of agent vs workflow based scheduled tasks.
 * - Dynamic Controls: Toggling job enabled states and deleting jobs with confirmation.
 * - History Tracing: Expanding jobs to fetch and render execution run history.
 * - Creation Flow: Validating the multi-step form for new scheduled operations.
 * @aiContext
 * - Mocks TadpoleOSService for job CRUD and run history retrieval.
 * - Mocks ResizeObserver and framer-motion AnimatePresence to bypass layout/animation side-effects.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import ScheduledJobs from './ScheduledJobs';
import { TadpoleOSService } from '../services/tadpoleosService';
import { useAgentStore } from '../stores/agentStore';
import { EventBus } from '../services/eventBus';

// Mock Dependencies
vi.mock('../services/tadpoleosService', () => ({
    TadpoleOSService: {
        getScheduledJobs: vi.fn(),
        listContinuityWorkflows: vi.fn(),
        getUnifiedSkills: vi.fn(),
        getScheduledJobRuns: vi.fn(),
        createScheduledJob: vi.fn(),
        updateScheduledJob: vi.fn(),
        deleteScheduledJob: vi.fn()
    }
}));

vi.mock('../stores/agentStore', () => ({
    useAgentStore: Object.assign(vi.fn(), {
        getState: vi.fn(() => ({
            fetchAgents: vi.fn(),
            agents: [
                { id: 'agent-1', name: 'Alpha Agent', role: 'Dev' },
                { id: 'agent-2', name: 'Beta Agent', role: 'Tester' }
            ]
        }))
    })
}));

vi.mock('../services/eventBus', () => ({
    EventBus: {
        emit: vi.fn(),
        subscribe: vi.fn(() => () => { }),
        getHistory: vi.fn(() => []),
    }
}));

// Mock ResizeObserver
global.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
};

// Fix AnimatePresence for testing by mocking framer-motion minimally
vi.mock('framer-motion', async () => {
    const actual = await vi.importActual('framer-motion');
    return {
        ...actual as any,
        AnimatePresence: ({ children }: any) => <>{children}</>,
        motion: {
            div: ({ children, ...props }: any) => <div {...props}>{children}</div>
        }
    };
});

describe('ScheduledJobs Page', () => {
    const mockJobs = [
        {
            id: 'job-1',
            name: 'Daily Summary',
            agent_id: 'agent-1',
            workflow_id: null,
            cron_expr: '0 8 * * *',
            prompt: 'Summarize tasks',
            enabled: true,
            budget_usd: 0.5,
            consecutive_failures: 0,
            max_failures: 3,
            next_run_at: '2026-03-13T08:00:00Z',
            created_at: '2026-03-12T08:00:00Z',
            updated_at: '2026-03-12T08:00:00Z'
        },
        {
            id: 'job-2',
            name: 'Weekly Backup',
            agent_id: '',
            workflow_id: 'wf-1',
            cron_expr: '0 0 * * 0',
            prompt: '',
            enabled: true,
            budget_usd: 1.0,
            consecutive_failures: 1, // Has failed
            max_failures: 3,
            next_run_at: '2026-03-15T00:00:00Z',
            created_at: '2026-03-12T08:00:00Z',
            updated_at: '2026-03-12T08:00:00Z'
        }
    ];

    const mockWorkflows = [
        { id: 'wf-1', name: 'Database Backup Workflow', department: 'Operations', target_agents: [] }
    ];

    const mockRuns = [
        {
            id: 'run-1',
            job_id: 'job-1',
            mission_id: 'mission-abc',
            status: 'completed',
            started_at: '2026-03-12T08:00:00Z',
            completed_at: '2026-03-12T08:05:00Z',
            cost_usd: 0.12,
            error: null
        }
    ];

    beforeEach(() => {
        vi.clearAllMocks();
        
        // Ensure useAgentStore hook returns our mocked agents array directly
        (useAgentStore as unknown as ReturnType<typeof vi.fn>).mockImplementation((selector: any) => {
            const state = useAgentStore.getState();
            return selector(state);
        });

        (TadpoleOSService.getScheduledJobs as any).mockResolvedValue(mockJobs);
        (TadpoleOSService.listContinuityWorkflows as any).mockResolvedValue(mockWorkflows);
        (TadpoleOSService.getUnifiedSkills as any).mockResolvedValue({ scripts: [], manifests: [], workflows: [] });
        (TadpoleOSService.getScheduledJobRuns as any).mockResolvedValue(mockRuns);

        // Required for `confirm` dialogs in test
        global.confirm = vi.fn(() => true);
    });

    it('renders the job list correctly', async () => {
        await act(async () => {
            render(<ScheduledJobs />);
        });

        expect(screen.getByText('Continuity Scheduler')).toBeInTheDocument();
        
        // Asserts job names
        expect(screen.getByText('Daily Summary')).toBeInTheDocument();
        expect(screen.getByText('Weekly Backup')).toBeInTheDocument();

        // Asserts statuses
        expect(screen.getByText('ACTIVE')).toBeInTheDocument();
        expect(screen.getByText('1 FAILS')).toBeInTheDocument();

        // Asserts targets
        expect(screen.getByText('Alpha Agent')).toBeInTheDocument();
        expect(screen.getByText('Database Backup Workflow')).toBeInTheDocument();
    });

    it('toggles job enabled state', async () => {
        await act(async () => {
            render(<ScheduledJobs />);
        });

        const pauseButtons = screen.getAllByRole('button').filter(b => b.innerHTML.includes('lucide-pause') || b.innerHTML.includes('lucide-play'));
        
        await act(async () => {
            fireEvent.click(pauseButtons[0]); // Pause first job
        });

        expect(TadpoleOSService.updateScheduledJob).toHaveBeenCalledWith('job-1', { enabled: false });
        expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({ text: expect.stringContaining('Daily Summary') }));
    });

    it('deletes a job after confirmation', async () => {
        await act(async () => {
            render(<ScheduledJobs />);
        });

        const deleteButtons = screen.getAllByRole('button').filter(b => b.innerHTML.includes('lucide-trash'));
        
        await act(async () => {
            fireEvent.click(deleteButtons[0]);
        });

        // The custom ConfirmDialog should be visible
        expect(screen.getByText('PURGE SCHEDULED JOB')).toBeInTheDocument();
        
        const confirmButton = screen.getByRole('button', { name: /PURGE CONFIGURATION/i });
        
        await act(async () => {
            fireEvent.click(confirmButton);
        });

        expect(TadpoleOSService.deleteScheduledJob).toHaveBeenCalledWith('job-1');
        expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({ text: expect.stringContaining('Daily Summary') }));
    });

    it('expands a job and fetches its run history', async () => {
        await act(async () => {
            render(<ScheduledJobs />);
        });

        const expandButtons = screen.getAllByRole('button').filter(b => b.innerHTML.includes('lucide-chevron-right'));
        
        await act(async () => {
            fireEvent.click(expandButtons[0]); // Expand first job
        });

        expect(TadpoleOSService.getScheduledJobRuns).toHaveBeenCalledWith('job-1');
        
        // The prompt should be visible
        expect(screen.getByText('Mission Prompt')).toBeInTheDocument();
        expect(screen.getByText('Summarize tasks')).toBeInTheDocument();

        // The run history should be visible
        expect(await screen.findByText('COMPLETED')).toBeInTheDocument();
        expect(screen.getByText('$0.1200')).toBeInTheDocument();
    });

    it('can create a new agent-based scheduled job', async () => {
        await act(async () => {
            render(<ScheduledJobs />);
        });

        await act(async () => {
            fireEvent.click(screen.getByRole('button', { name: /New Job/i }));
        });

        expect(screen.getByText('Configure New Job')).toBeInTheDocument();

        // Fill out form
        fireEvent.change(screen.getByLabelText(/Job Name/i), { target: { value: 'New Agent Job' } });
        fireEvent.change(screen.getByLabelText(/Target Agent/i), { target: { value: 'agent-2' } });
        fireEvent.change(screen.getByLabelText(/Cron Expression/i), { target: { value: '0 12 * * *' } });
        fireEvent.change(screen.getByLabelText(/Mission Prompt/i), { target: { value: 'Do the new thing' } });

        await act(async () => {
            fireEvent.click(screen.getByRole('button', { name: /Save Job/i }));
        });

        expect(TadpoleOSService.createScheduledJob).toHaveBeenCalledWith(expect.objectContaining({
            name: 'New Agent Job',
            agent_id: 'agent-2',
            cron_expr: '0 12 * * *',
            prompt: 'Do the new thing'
        }));
        
        expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'success'
        }));
    });

    it('can switch to creating a workflow-based scheduled job', async () => {
        await act(async () => {
            render(<ScheduledJobs />);
        });

        await act(async () => {
            fireEvent.click(screen.getByRole('button', { name: /New Job/i }));
        });

        await act(async () => {
            fireEvent.click(screen.getByText('MULTI-STEP WORKFLOW'));
        });

        // The Mission prompt field should disappear and Workflow select should appear
        expect(screen.queryByLabelText(/Mission Prompt/i)).not.toBeInTheDocument();
        
        fireEvent.change(screen.getByLabelText(/Job Name/i), { target: { value: 'New WF Job' } });
        fireEvent.change(screen.getByLabelText(/Target Workflow/i), { target: { value: 'wf-1' } });

        await act(async () => {
            fireEvent.click(screen.getByRole('button', { name: /Save Job/i }));
        });

        expect(TadpoleOSService.createScheduledJob).toHaveBeenCalledWith(expect.objectContaining({
            name: 'New WF Job',
            workflow_id: 'wf-1'
        }));
    });
});
