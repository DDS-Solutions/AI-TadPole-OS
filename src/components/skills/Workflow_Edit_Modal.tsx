/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Visual designer for autonomous agent workflows. 
 * Orchestrates multi-step sequence definitions, delay management, and persistence via the `workflow_store`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Circular logic detection failure, or step order corruption during rapid re-ordering.
 * - **Telemetry Link**: Search for `[Workflow_Edit_Modal]` or `save_workflow_sequence` in browser logs.
 */

import React from 'react';
import { FileText, AlertTriangle } from 'lucide-react';
import { i18n } from '../../i18n';
import type { Workflow_Definition } from '../../stores/skill_store';

interface Workflow_Edit_Modal_Props {
    is_open: boolean;
    on_close: () => void;
    editing_wf: Partial<Workflow_Definition>;
    set_editing_wf: (wf: Partial<Workflow_Definition>) => void;
    wf_save_error: string | null;
    is_saving: boolean;
    on_save: () => void;
}

export const Workflow_Edit_Modal: React.FC<Workflow_Edit_Modal_Props> = ({
    is_open,
    on_close,
    editing_wf,
    set_editing_wf,
    wf_save_error,
    is_saving,
    on_save
}) => {
    if (!is_open) return null;

    return (
        <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-4">
            <div className="bg-zinc-950 border border-zinc-800 w-full max-w-4xl rounded-2xl shadow-2xl flex flex-col max-h-[90vh] relative overflow-hidden">
                <div className="neural-grid opacity-10" />
                <div className="p-5 border-b border-zinc-800 flex justify-between items-center shrink-0 relative z-10 bg-zinc-950/50">
                    <h2 className="text-lg font-bold text-zinc-100 flex items-center gap-2">
                        <FileText className="text-blue-500" /> {editing_wf?.name ? i18n.t('skills.modal_edit_workflow') : i18n.t('skills.modal_create_workflow')}
                    </h2>
                    <button onClick={on_close} className="text-zinc-500 hover:text-zinc-300 p-1">✕</button>
                </div>
                <div className="p-6 overflow-y-auto space-y-5 custom-scrollbar flex-1 relative z-10 bg-zinc-950/80">
                    <div>
                        <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('skills.label_workflow_name')}</label>
                        <input
                            type="text"
                            value={editing_wf.name || ''}
                            onChange={e => set_editing_wf({ ...editing_wf, name: e.target.value })}
                            className="w-full bg-zinc-900 border border-zinc-800 rounded p-3 text-blue-300 font-mono text-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all placeholder:text-zinc-700"
                            placeholder="Deep Research Protocol"
                        />
                    </div>
                    <div className="flex-1 flex flex-col h-full min-h-[400px]">
                        <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('skills.label_markdown_content')}</label>
                        <textarea
                            value={editing_wf.content || ''}
                            onChange={e => set_editing_wf({ ...editing_wf, content: e.target.value })}
                            className="flex-1 w-full bg-zinc-900 border border-zinc-800 rounded p-4 text-zinc-300 font-mono text-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all resize-none custom-scrollbar placeholder:text-zinc-700"
                            placeholder="# Protocol Overview\n1. Do X\n2. Do Y"
                            spellCheck="false"
                        />
                    </div>
                </div>
                <div className="p-5 border-t border-zinc-800 flex justify-end gap-3 shrink-0 relative z-10 bg-zinc-950/90 items-center">
                    {wf_save_error && <div className="text-xs text-red-400 font-mono mr-auto flex items-center gap-1"><AlertTriangle className="w-3.5 h-3.5" /> {wf_save_error}</div>}
                    <button onClick={on_close} className="px-5 py-2 text-xs font-bold text-zinc-500 hover:text-zinc-100 transition-colors">{i18n.t('skills.btn_cancel')}</button>
                    <button onClick={on_save} disabled={is_saving || !editing_wf.name || !editing_wf.content} className="bg-blue-600 hover:bg-blue-500 text-white px-6 py-2 rounded-lg text-xs font-bold transition-colors shadow-lg shadow-blue-500/20 disabled:opacity-50">{is_saving ? i18n.t('skills.btn_saving') : i18n.t('skills.btn_save_workflow')}</button>
                </div>
            </div>
        </div>
    );
};

