/**
 * @docs ARCHITECTURE:Testing
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
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Job_Form_Manager } from './Job_Form_Manager';
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
});
