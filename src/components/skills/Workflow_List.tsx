/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: High-density registry for autonomous behavioral sequences. 
 * Orchestrates categorized workflow rendering and facilitates empty-state transitions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: List re-hydration lag, or empty state occlusion.
 * - **Telemetry Link**: Search for `[Workflow_List]` or `load_workflow_catalog` in UI tracing.
 */

import React from 'react';
import { Activity, Plus } from 'lucide-react';
import { Tooltip, Tw_Empty_State } from '../ui';
import { i18n } from '../../i18n';
import type { Workflow_Definition } from '../../stores/skill_store';
import { Workflow_Card } from './Workflow_Card';

interface Workflow_List_Props {
    workflows: Workflow_Definition[];
    search_query: string;
    active_category: 'user' | 'ai';
    on_new_workflow: () => void;
    on_edit_workflow: (workflow: Workflow_Definition) => void;
    on_assign_workflow: (name: string) => void;
    on_delete_workflow: (name: string) => void;
}

export const Workflow_List: React.FC<Workflow_List_Props> = ({
    workflows,
    search_query,
    active_category,
    on_new_workflow,
    on_edit_workflow,
    on_assign_workflow,
    on_delete_workflow
}) => {
    const filtered_workflows = React.useMemo(() => workflows.filter(w => 
        w.category === active_category && 
        (w.name.toLowerCase().includes(search_query.toLowerCase()) || 
         w.content.toLowerCase().includes(search_query.toLowerCase()))
    ), [workflows, active_category, search_query]);

    return (
        <div className="space-y-6">
            <div className="flex justify-between items-center mb-6 sticky top-0 bg-zinc-950/80 backdrop-blur-md pt-2 pb-3 border-b border-zinc-800/50 z-20">
                <h3 className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                    <Activity size={12} className="text-green-500" /> {i18n.t('skills.header_guiding')}
                </h3>
                <Tooltip content={i18n.t('skills.tooltip_new_workflow')} position="bottom">
                    <button
                        onClick={on_new_workflow}
                        className="bg-green-600 hover:bg-green-500 text-white px-3 py-1.5 rounded-lg text-xs font-bold flex items-center gap-2 transition-colors shadow-[0_0_15px_rgba(37,99,235,0.3)]"
                    >
                        <Plus className="w-3.5 h-3.5" /> {i18n.t('skills.btn_new_workflow')}
                    </button>
                </Tooltip>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 relative z-10">
                {filtered_workflows.map(wf => (
                    <Workflow_Card 
                        key={wf.name} 
                        workflow={wf} 
                        on_edit={on_edit_workflow} 
                        on_assign={on_assign_workflow} 
                        on_delete={on_delete_workflow} 
                    />
                ))}
                {filtered_workflows.length === 0 && (
                    <div className="col-span-full">
                        <Tw_Empty_State 
                            title={i18n.t('skills.empty_workflows_title')} 
                            description={i18n.t('skills.empty_workflows_desc')} 
                        />
                    </div>
                )}
            </div>
        </div>
    );
};


// Metadata: [Workflow_List]
