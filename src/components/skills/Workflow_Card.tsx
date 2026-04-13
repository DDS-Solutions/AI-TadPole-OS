/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Workflow orchestration identity card. 
 * Renders multi-step workflow logic summaries and facilitates rapid assignment of complex behavioral sequences.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Step count desync, or action button click-through to card body.
 * - **Telemetry Link**: Search for `[Workflow_Card]` or `workflow_assignment` in UI logs.
 */

import React from 'react';
import { Edit2, Users, Trash2 } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Workflow_Definition } from '../../stores/skill_store';

interface Workflow_Card_Props {
    workflow: Workflow_Definition;
    on_edit: (workflow: Workflow_Definition) => void;
    on_assign: (name: string) => void;
    on_delete: (name: string) => void;
}

export const Workflow_Card: React.FC<Workflow_Card_Props> = ({ workflow, on_edit, on_assign, on_delete }) => {
    return (
        <div className="bg-zinc-950 border border-zinc-800 p-5 rounded-xl transition-all duration-300 hover:border-amber-500/30 hover:shadow-[0_0_15px_rgba(245,158,11,0.15)] group relative overflow-hidden shadow-sm">
            <div className="neural-grid opacity-[0.03]" />
            <div className="absolute top-4 right-4 flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity z-20">
                <Tooltip content={i18n.t('skills.tooltip_edit_workflow')} position="top">
                    <button onClick={() => on_edit(workflow)} className="text-zinc-500 hover:text-blue-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded">
                        <Edit2 className="w-3.5 h-3.5" />
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('agent_manager.tooltip_assign')} position="top">
                    <button onClick={() => on_assign(workflow.name)} className="text-zinc-500 hover:text-emerald-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded transition-colors">
                        <Users className="w-3.5 h-3.5" />
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('skills.tooltip_delete_workflow')} position="top">
                    <button onClick={() => on_delete(workflow.name)} className="text-zinc-500 hover:text-red-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded">
                        <Trash2 className="w-3.5 h-3.5" />
                    </button>
                </Tooltip>
            </div>
            <div className="relative z-10 flex flex-col h-full">
                <div className="flex items-center gap-3 mb-3 pr-16 text-zinc-300 font-bold tracking-wide">
                    <div className="w-2 h-2 rounded-full bg-amber-500/30 group-hover:bg-amber-400 group-hover:shadow-[0_0_8px_rgba(245,158,11,0.5)] transition-all shrink-0 mt-0.5"></div>
                    <h3 className="font-mono text-sm">{workflow.name}</h3>
                </div>
                <div className="bg-black/40 border border-zinc-800/50 p-3 rounded text-[11px] text-zinc-400 font-mono whitespace-pre-wrap max-h-64 overflow-y-auto custom-scrollbar flex-1">
                    {workflow.content}
                </div>
            </div>
        </div>
    );
};

