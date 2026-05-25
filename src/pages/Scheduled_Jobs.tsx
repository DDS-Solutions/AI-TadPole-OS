/**
 * @docs ARCHITECTURE:UI-Pages
 * 
 * ### AI Assist Note
 * **Scheduled_Jobs Page**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Scheduled_Jobs]` in observability traces.
 */

import React from 'react';
import { Clock, Plus } from 'lucide-react';
import { useScheduledJobs } from '../hooks/useScheduledJobs';
import { Job_Form_Manager } from '../components/scheduler/Job_Form_Manager';
import { Job_Table } from '../components/scheduler/Job_Table';
import { Tooltip } from '../components/ui';
import { Confirm_Dialog } from '../components/ui/Confirm_Dialog';
import { i18n } from '../i18n';

/**
 * Scheduled_Jobs Page
 * 
 * ### 🗓️ Administrative Interface
 * Manages automated agent tasks and workflow triggers. Features:
 * - **Cron Scheduling**: Standard UNIX cron expression mapping.
 * - **Budget Guard**: Per-job budget enforcement.
 * - **Run History**: Real-time state management of previous execution results.
 * 
 * Refactored to Container-Presentational pattern.
 */
const Scheduled_Jobs: React.FC = () => {
    const {
        jobs,
        workflows,
        is_loading,
        expanded_job,
        runs_map,
        confirm_delete,
        is_creating,
        editing_job_id,
        job_type,
        target_search,
        job_config,
        filtered_agents,
        filtered_workflows,
        set_is_creating,
        set_job_type,
        set_target_search,
        set_job_config,
        set_confirm_delete,
        toggle_expand,
        toggle_enable,
        handle_edit,
        delete_job,
        handle_confirm_delete,
        handle_job_submit,
        reset_form
    } = useScheduledJobs();

    if (is_loading) {
        return <div className="p-8 text-zinc-500 flex items-center gap-2"><Clock className="animate-pulse" /> {i18n.t('scheduled_jobs.loading')}</div>;
    }

    return (
        <div className="p-8 space-y-8 min-h-screen bg-[color:var(--color-background)] text-zinc-100">
            {/* GEO Optimization: Structured Data & Semantic Header */}
            <script type="application/ld+json">
                {JSON.stringify({
                    "@context": "https://schema.org",
                    "@type": "Service",
                    "name": "Tadpole OS Automation Scheduler",
                    "description": "Enterprise-grade cron-style scheduler for autonomous agent missions and workflow triggers.",
                    "provider": { "@type": "Organization", "name": "Sovereign Engineering" },
                    "author": { "@type": "Person", "name": "Agent of Nine" }
                })}
            </script>

            <header className="flex justify-between items-end">
                <Tooltip content={i18n.t('scheduled_jobs.header_tooltip')} position="right">
                    <div>
                        <h1 className="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-white to-zinc-500 cursor-help">
                            {i18n.t('scheduled_jobs.title')}
                        </h1>
                    </div>
                </Tooltip>
                <div className="flex gap-3">
                    <button
                        onClick={() => {
                            reset_form();
                            set_is_creating(true);
                        }}
                        className="px-4 py-2 bg-green-600 hover:bg-green-500 text-white font-bold rounded-lg text-[10px] transition-all flex items-center gap-2 shadow-lg shadow-green-500/20 uppercase tracking-widest"
                    >
                        <Plus size={14} /> {i18n.t('scheduled_jobs.new_job')}
                    </button>
                </div>
            </header>

            {is_creating && (
                <Job_Form_Manager 
                    job_type={job_type}
                    set_job_type={set_job_type}
                    current_job_config={job_config}
                    set_job_config={set_job_config}
                    target_search={target_search}
                    set_target_search={set_target_search}
                    filtered_agents={filtered_agents}
                    filtered_workflows={filtered_workflows}
                    editing_job_id={editing_job_id}
                    on_cancel={reset_form}
                    handle_job_submit={handle_job_submit}
                />
            )}

            <Job_Table 
                jobs={jobs}
                agents={filtered_agents}
                workflows={workflows}
                expanded_job={expanded_job}
                runs_map={runs_map}
                toggle_expand={toggle_expand}
                toggle_enable={toggle_enable}
                handle_edit={handle_edit}
                delete_job={delete_job}
            />

            <Confirm_Dialog
                is_open={!!confirm_delete}
                title={i18n.t('scheduled_jobs.confirm_purge_title')}
                message={i18n.t('scheduled_jobs.confirm_purge_message', { name: confirm_delete?.name ?? '' })}
                confirm_label={i18n.t('scheduled_jobs.confirm_purge_button')}
                on_confirm={handle_confirm_delete}
                on_cancel={() => set_confirm_delete(null)}
                variant="danger"
            />
        </div>
    );
};

export default Scheduled_Jobs;

// Metadata: [Scheduled_Jobs]

// Metadata: [Scheduled_Jobs]
