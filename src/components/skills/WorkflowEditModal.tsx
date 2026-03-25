import React from 'react';
import { FileText, AlertTriangle } from 'lucide-react';
import { i18n } from '../../i18n';
import type { WorkflowDefinition } from '../../stores/skillStore';

interface WorkflowEditModalProps {
    isOpen: boolean;
    onClose: () => void;
    editingWf: Partial<WorkflowDefinition>;
    setEditingWf: (wf: Partial<WorkflowDefinition>) => void;
    wfSaveError: string | null;
    isSaving: boolean;
    onSave: () => void;
}

export const WorkflowEditModal: React.FC<WorkflowEditModalProps> = ({
    isOpen,
    onClose,
    editingWf,
    setEditingWf,
    wfSaveError,
    isSaving,
    onSave
}) => {
    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-4">
            <div className="bg-zinc-950 border border-zinc-800 w-full max-w-4xl rounded-2xl shadow-2xl flex flex-col max-h-[90vh] relative overflow-hidden">
                <div className="neural-grid opacity-10" />
                <div className="p-5 border-b border-zinc-800 flex justify-between items-center shrink-0 relative z-10 bg-zinc-950/50">
                    <h2 className="text-lg font-bold text-zinc-100 flex items-center gap-2">
                        <FileText className="text-blue-500" /> {editingWf?.name ? i18n.t('skills.modal_edit_workflow') : i18n.t('skills.modal_create_workflow')}
                    </h2>
                    <button onClick={onClose} className="text-zinc-500 hover:text-zinc-300 p-1">✕</button>
                </div>
                <div className="p-6 overflow-y-auto space-y-5 custom-scrollbar flex-1 relative z-10 bg-zinc-950/80">
                    <div>
                        <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('skills.label_workflow_name')}</label>
                        <input
                            type="text"
                            value={editingWf.name || ''}
                            onChange={e => setEditingWf({ ...editingWf, name: e.target.value })}
                            className="w-full bg-zinc-900 border border-zinc-800 rounded p-3 text-blue-300 font-mono text-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all placeholder:text-zinc-700"
                            placeholder="Deep Research Protocol"
                        />
                    </div>
                    <div className="flex-1 flex flex-col h-full min-h-[400px]">
                        <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('skills.label_markdown_content')}</label>
                        <textarea
                            value={editingWf.content || ''}
                            onChange={e => setEditingWf({ ...editingWf, content: e.target.value })}
                            className="flex-1 w-full bg-zinc-900 border border-zinc-800 rounded p-4 text-zinc-300 font-mono text-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all resize-none custom-scrollbar placeholder:text-zinc-700"
                            placeholder="# Protocol Overview\n1. Do X\n2. Do Y"
                            spellCheck="false"
                        />
                    </div>
                </div>
                <div className="p-5 border-t border-zinc-800 flex justify-end gap-3 shrink-0 relative z-10 bg-zinc-950/90 items-center">
                    {wfSaveError && <div className="text-xs text-red-400 font-mono mr-auto flex items-center gap-1"><AlertTriangle className="w-3.5 h-3.5" /> {wfSaveError}</div>}
                    <button onClick={onClose} className="px-5 py-2 text-xs font-bold text-zinc-500 hover:text-zinc-100 transition-colors">{i18n.t('skills.btn_cancel')}</button>
                    <button onClick={onSave} disabled={isSaving || !editingWf.name || !editingWf.content} className="bg-blue-600 hover:bg-blue-500 text-white px-6 py-2 rounded-lg text-xs font-bold transition-colors shadow-lg shadow-blue-500/20 disabled:opacity-50">{isSaving ? i18n.t('skills.btn_saving') : i18n.t('skills.btn_save_workflow')}</button>
                </div>
            </div>
        </div>
    );
};
