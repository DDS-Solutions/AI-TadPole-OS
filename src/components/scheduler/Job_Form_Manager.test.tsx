/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Assist Note
 * - Keep fixtures isolated and assert the public behavior of the extracted boundaries.
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Scheduler / Job_Form_Manager.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Job_Form_Manager } from './Job_Form_Manager';
import { i18n } from '../../i18n';
import type { Job_Config_State } from '../../hooks/useScheduledJobs';

describe('Job_Form_Manager Component', () => {
    const mock_config: Job_Config_State = {
        name: 'Weekly Backup',
        agent_id: 'agent-1',
        workflow_id: null,
        prompt: 'Run weekly database snapshot',
        cron_expr: '0 0 * * 0',
        budget_usd: 0.50,
        max_failures: 3
    };

    it('renders form inputs with current job configuration values', () => {
        render(
            <Job_Form_Manager
                job_type="agent"
                set_job_type={vi.fn()}
                current_job_config={mock_config}
                set_job_config={vi.fn()}
                target_search=""
                set_target_search={vi.fn()}
                filtered_agents={[{ id: 'agent-1', name: 'Scout Agent', role: 'Security' }]}
                filtered_workflows={[]}
                editing_job_id={null}
                on_cancel={vi.fn()}
                handle_job_submit={vi.fn()}
            />
        );

        expect(screen.getByDisplayValue('Weekly Backup')).toBeDefined();
        expect(screen.getByDisplayValue('Run weekly database snapshot')).toBeDefined();
        expect(screen.getByDisplayValue('0 0 * * 0')).toBeDefined();
    });

    it('triggers submit handler when form is submitted', () => {
        const submit_mock = vi.fn((e) => { e.preventDefault(); return Promise.resolve(); });
        render(
            <Job_Form_Manager
                job_type="agent"
                set_job_type={vi.fn()}
                current_job_config={mock_config}
                set_job_config={vi.fn()}
                target_search=""
                set_target_search={vi.fn()}
                filtered_agents={[{ id: 'agent-1', name: 'Scout Agent', role: 'Security' }]}
                filtered_workflows={[]}
                editing_job_id="job-123"
                on_cancel={vi.fn()}
                handle_job_submit={submit_mock}
            />
        );

        const form = screen.getByDisplayValue('Weekly Backup').closest('form');
        expect(form).toBeDefined();
        fireEvent.submit(form!);
        expect(submit_mock).toHaveBeenCalled();
    });

    it('renders the schedule helper mini panel and live schedule preview', () => {
        render(
            <Job_Form_Manager
                job_type="agent"
                set_job_type={vi.fn()}
                current_job_config={mock_config}
                set_job_config={vi.fn()}
                target_search=""
                set_target_search={vi.fn()}
                filtered_agents={[{ id: 'agent-1', name: 'Scout Agent', role: 'Security' }]}
                filtered_workflows={[]}
                editing_job_id={null}
                on_cancel={vi.fn()}
                handle_job_submit={vi.fn()}
            />
        );

        // Schedule Helper panel is rendered
        expect(screen.getByText(i18n.t('scheduled_jobs.schedule_config'))).toBeDefined();
        // Estimated schedule preview is rendered
        expect(screen.getByText(/Estimated Schedule/i)).toBeDefined();
        expect(screen.getByText(/Runs weekly on Sunday/i)).toBeDefined();
    });

    it('updates cron configuration when Schedule Helper emits new cron', () => {
        const set_config_mock = vi.fn();
        render(
            <Job_Form_Manager
                job_type="agent"
                set_job_type={vi.fn()}
                current_job_config={mock_config}
                set_job_config={set_config_mock}
                target_search=""
                set_target_search={vi.fn()}
                filtered_agents={[{ id: 'agent-1', name: 'Scout Agent', role: 'Security' }]}
                filtered_workflows={[]}
                editing_job_id={null}
                on_cancel={vi.fn()}
                handle_job_submit={vi.fn()}
            />
        );

        const interval_input = screen.getByLabelText('Every');
        fireEvent.change(interval_input, { target: { value: '45' } });

        expect(set_config_mock).toHaveBeenCalled();
    });
});
