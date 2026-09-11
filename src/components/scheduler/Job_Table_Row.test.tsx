/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Scheduler / Job_Table_Row.test
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
import { render, screen } from '@testing-library/react';
import { Job_Table_Row } from './Job_Table_Row';
import type { Scheduled_Job } from '../../services/tadpoleos_service';

describe('Job_Table_Row Component', () => {
    const mock_job: Scheduled_Job = {
        id: 'job-01',
        name: 'Nightly Sync',
        agent_id: 'agent-1',
        cron_expr: '0 0 * * *',
        enabled: true,
        budget_usd: 0.20,
        max_failures: 3,
        total_runs: 5,
        failed_runs: 0
    } as any;

    it('renders job details and action buttons', () => {
        const toggle_enable_mock = vi.fn();
        const handle_edit_mock = vi.fn();
        const delete_job_mock = vi.fn();

        render(
            <table>
                <tbody>
                    <Job_Table_Row
                        job={mock_job}
                        agents={[{ id: 'agent-1', name: 'Sync Agent' } as any]}
                        workflows={[]}
                        is_expanded={false}
                        runs={[]}
                        toggle_expand={vi.fn()}
                        toggle_enable={toggle_enable_mock}
                        handle_edit={handle_edit_mock}
                        delete_job={delete_job_mock}
                    />
                </tbody>
            </table>
        );

        expect(screen.getByText('Nightly Sync')).toBeDefined();
        expect(screen.getByText('0 0 * * *')).toBeDefined();
        expect(screen.getByText('Sync Agent')).toBeDefined();
    });
});
