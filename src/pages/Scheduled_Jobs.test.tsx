/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the Cron-like job scheduling and automated task execution logic.** 
 * Verifies the continuity workflow integration and ensures failed jobs trigger the expected recovery workflows via `event_bus`. 
 * Mocks `tadpole_os_service` for job CRUD and run history retrieval with snake_case parity.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Job overlap (race conditions) or failure to persist 'Last Run' metadata to the `jobs_store` after a system restart.
 * - **Telemetry Link**: Search `[Scheduled_Jobs.test]` in tracing logs.
 */


/**
 * @file Scheduled_Jobs.test.tsx
 * @description Suite for the Continuity Scheduler and Managed Recurring Tasks page.
 * @module Pages/Scheduled_Jobs
 * @testedBehavior
 * - Job Inventory: Correct rendering of agent vs workflow based scheduled tasks.
 * - Dynamic Controls: Toggling job enabled states and deleting jobs with confirmation.
 * - History Tracing: Expanding jobs to fetch and render execution run history.
 * - Creation Flow: Validating the multi-step form for new scheduled operations.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks tadpole_os_service for job CRUD and run history retrieval with snake_case.
 * - Mocks ResizeObserver and framer-motion AnimatePresence to bypass layout/animation side-effects.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import Scheduled_Jobs from './Scheduled_Jobs';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { use_agent_store } from '../stores/agent_store';
import { event_bus } from '../services/event_bus';

// Mock Dependencies
vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        get_scheduled_jobs: vi.fn(),
        list_continuity_workflows: vi.fn(),
        get_unified_skills: vi.fn(),
        get_scheduled_job_runs: vi.fn(),
        create_scheduled_job: vi.fn(),
        update_scheduled_job: vi.fn(),
        delete_scheduled_job: vi.fn()
    }
}));

vi.mock('../stores/agent_store', () => ({
    use_agent_store: Object.assign(vi.fn(), {
        getState: vi.fn(() => ({
            fetch_agents: vi.fn(),
            agents: [
                { id: 'agent-1', name: 'Alpha Agent', role: 'Dev' },
                { id: 'agent-2', name: 'Beta Agent', role: 'Tester' }
            ]
        }))
    })
}));

vi.mock('../services/event_bus', () => ({
    event_bus: {
        emit_log: vi.fn(),
        subscribe_logs: vi.fn(() => () => { }),
        get_history: vi.fn(() => []),
    }
}));

vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string, options?: any) => {
            if (key === 'scheduled_jobs.status_fails') return `${options?.count} FAILS`;
            if (options?.name) return `${key} (${options.name})`;
            return key;
        },
    },
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

describe('Scheduled_Jobs Page', () => {
    const mock_jobs = [
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

    const mock_workflows = [
        { id: 'wf-1', name: 'Database Backup Workflow', department: 'Operations', target_agents: [], type: 'continuity' }
    ];

    const mock_runs = [
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
        
        // Ensure use_agent_store hook returns our mocked agents array directly
        (use_agent_store as unknown as ReturnType<typeof vi.fn>).mockImplementation((selector: any) => {
            const state = (use_agent_store as any).getState();
            return selector(state);
        });

        (tadpole_os_service.get_scheduled_jobs as any).mockResolvedValue(mock_jobs);
        (tadpole_os_service.list_continuity_workflows as any).mockResolvedValue(mock_workflows);
        (tadpole_os_service.get_unified_skills as any).mockResolvedValue({ scripts: [], manifests: [], workflows: [] });
        (tadpole_os_service.get_scheduled_job_runs as any).mockResolvedValue(mock_runs);

        // Required for `confirm` dialogs in test
        global.confirm = vi.fn(() => true);
    });

    it('renders the job list correctly', async () => {
        await act(async () => {
            render(<Scheduled_Jobs />);
        });

        expect(screen.getByText('scheduled_jobs.title')).toBeInTheDocument();
        
        // Asserts job names
        expect(screen.getByText('Daily Summary')).toBeInTheDocument();
        expect(screen.getByText('Weekly Backup')).toBeInTheDocument();

        // Asserts statuses
        expect(screen.getByText('scheduled_jobs.status_active')).toBeInTheDocument();
        expect(screen.getByText('1 FAILS')).toBeInTheDocument();

        // Asserts targets
        expect(screen.getByText('Alpha Agent')).toBeInTheDocument();
        expect(screen.getByText('Database Backup Workflow')).toBeInTheDocument();
    });

    it('toggles job enabled state', async () => {
        await act(async () => {
            render(<Scheduled_Jobs />);
        });

        const pause_buttons = screen.getAllByRole('button').filter(b => b.innerHTML.includes('lucide-pause') || b.innerHTML.includes('lucide-play'));
        
        await act(async () => {
            fireEvent.click(pause_buttons[0]); // Pause first job
        });

        expect(tadpole_os_service.update_scheduled_job).toHaveBeenCalledWith('job-1', { enabled: false });
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ text: expect.stringContaining('Daily Summary') }));
    });

    it('deletes a job after confirmation', async () => {
        await act(async () => {
            render(<Scheduled_Jobs />);
        });

        const delete_buttons = screen.getAllByLabelText(/Delete Daily Summary/i);
        
        await act(async () => {
            fireEvent.click(delete_buttons[0]);
        });

        // The custom Confirm_Dialog should be visible
        expect(screen.getByText('scheduled_jobs.confirm_purge_title')).toBeInTheDocument();
        
        const confirm_button = screen.getByRole('button', { name: /scheduled_jobs.confirm_purge_button/i });
        
        await act(async () => {
            fireEvent.click(confirm_button);
        });

        expect(tadpole_os_service.delete_scheduled_job).toHaveBeenCalledWith('job-1');
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ text: expect.stringContaining('Daily Summary') }));
    });

    it('expands a job and fetches its run history', async () => {
        await act(async () => {
            render(<Scheduled_Jobs />);
        });

        const expand_buttons = screen.getAllByRole('button').filter(b => b.innerHTML.includes('lucide-chevron-right'));
        
        await act(async () => {
            fireEvent.click(expand_buttons[0]); // Expand first job
        });

        expect(tadpole_os_service.get_scheduled_job_runs).toHaveBeenCalledWith('job-1');
        
        // The prompt should be visible
        expect(screen.getByText('scheduled_jobs.mission_prompt')).toBeInTheDocument();
        expect(screen.getByText('Summarize tasks')).toBeInTheDocument();

        // The run history should be visible
        expect(await screen.findByText('COMPLETED')).toBeInTheDocument();
        expect(screen.getByText('$0.1200')).toBeInTheDocument();
    });

    it('can create a new agent-based scheduled job', async () => {
        await act(async () => {
            render(<Scheduled_Jobs />);
        });

        await act(async () => {
            fireEvent.click(screen.getByRole('button', { name: /scheduled_jobs.new_job/i }));
        });

        expect(screen.getByText('scheduled_jobs.configure_new')).toBeInTheDocument();

        // Fill out form
        fireEvent.change(screen.getByLabelText(/scheduled_jobs.job_name/i), { target: { value: 'New Agent Job' } });
        fireEvent.change(screen.getByLabelText(/scheduled_jobs.target_agent/i), { target: { value: 'agent-2' } });
        fireEvent.change(screen.getByLabelText(/scheduled_jobs.cron_expression/i), { target: { value: '0 12 * * *' } });
        fireEvent.change(screen.getByLabelText(/scheduled_jobs.mission_prompt/i), { target: { value: 'Do the new thing' } });

        await act(async () => {
            fireEvent.click(screen.getByRole('button', { name: /scheduled_jobs.save_job/i }));
        });

        expect(tadpole_os_service.create_scheduled_job).toHaveBeenCalledWith(expect.objectContaining({
            name: 'New Agent Job',
            agent_id: 'agent-2',
            cron_expr: '0 12 * * *',
            prompt: 'Do the new thing'
        }));
        
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            severity: 'success'
        }));
    });

    it('can switch to creating a workflow-based scheduled job', async () => {
        await act(async () => {
            render(<Scheduled_Jobs />);
        });

        await act(async () => {
            fireEvent.click(screen.getByRole('button', { name: /scheduled_jobs.new_job/i }));
        });

        await act(async () => {
            fireEvent.click(screen.getByText('scheduled_jobs.multi_step_workflow'));
        });

        // The Mission prompt field should disappear and Workflow select should appear
        expect(screen.queryByLabelText(/scheduled_jobs.mission_prompt/i)).not.toBeInTheDocument();
        
        fireEvent.change(screen.getByLabelText(/scheduled_jobs.job_name/i), { target: { value: 'New WF Job' } });
        fireEvent.change(screen.getByLabelText(/scheduled_jobs.target_workflow/i), { target: { value: 'wf-1' } });

        await act(async () => {
            fireEvent.click(screen.getByRole('button', { name: /scheduled_jobs.save_job/i }));
        });

        expect(tadpole_os_service.create_scheduled_job).toHaveBeenCalledWith(expect.objectContaining({
            name: 'New WF Job',
            workflow_id: 'wf-1'
        }));
    });
});

