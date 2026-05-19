/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Job_Table]` in observability traces.
 */

import React from 'react';
import { i18n } from '../../i18n';
import { Job_Table_Row } from './Job_Table_Row';
import type { Scheduled_Job, Scheduled_Job_Run, Workflow_Entry } from '../../services/tadpoleos_service';
import type { Agent } from '../../types';

interface JobTableProps {
    jobs: Scheduled_Job[];
    agents: Agent[];
    workflows: Workflow_Entry[];
    expanded_job: string | null;
    runs_map: Record<string, Scheduled_Job_Run[]>;
    toggle_expand: (id: string) => void;
    toggle_enable: (job: Scheduled_Job) => void;
    handle_edit: (job: Scheduled_Job) => void;
    delete_job: (id: string, name: string) => void;
}

export const Job_Table: React.FC<JobTableProps> = ({
    jobs,
    agents,
    workflows,
    expanded_job,
    runs_map,
    toggle_expand,
    toggle_enable,
    handle_edit,
    delete_job
}) => {
    return (
        <div className="bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-xl overflow-hidden shadow-2xl">
            <table className="w-full text-left text-sm">
                <thead className="bg-[color:var(--color-surface)] border-b border-[color:var(--color-border)] text-zinc-400 font-mono text-xs uppercase">
                    <tr>
                        <th className="px-6 py-4 font-medium tracking-wider w-10"></th>
                        <th className="px-6 py-4 font-medium tracking-wider">{i18n.t('scheduled_jobs.th_job_name')}</th>
                        <th className="px-6 py-4 font-medium tracking-wider">{i18n.t('scheduled_jobs.th_target')}</th>
                        <th className="px-6 py-4 font-medium tracking-wider">{i18n.t('scheduled_jobs.th_schedule')}</th>
                        <th className="px-6 py-4 font-medium tracking-wider">{i18n.t('scheduled_jobs.th_next_run')}</th>
                        <th className="px-6 py-4 font-medium tracking-wider">{i18n.t('scheduled_jobs.th_status')}</th>
                        <th className="px-6 py-4 font-medium tracking-wider text-right">{i18n.t('scheduled_jobs.th_actions')}</th>
                    </tr>
                </thead>
                <tbody className="divide-y divide-zinc-800">
                    {jobs.map((job) => (
                        <Job_Table_Row 
                            key={job.id}
                            job={job}
                            agents={agents}
                            workflows={workflows}
                            is_expanded={expanded_job === job.id}
                            runs={runs_map[job.id]}
                            toggle_expand={toggle_expand}
                            toggle_enable={toggle_enable}
                            handle_edit={handle_edit}
                            delete_job={delete_job}
                        />
                    ))}
                    {jobs.length === 0 && (
                        <tr>
                            <td colSpan={7} className="px-6 py-12 text-center text-zinc-500 font-mono text-sm">
                                {i18n.t('scheduled_jobs.no_jobs_deployed')}
                            </td>
                        </tr>
                    )}
                </tbody>
            </table>
        </div>
    );
};

// Metadata: [Job_Table]
