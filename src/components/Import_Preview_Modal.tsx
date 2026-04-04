import React, { useState } from 'react';
import type { Skill_Definition, Workflow_Definition, Hook_Definition } from '../stores/skill_store';
import { i18n } from '../i18n';

interface Import_Preview_Modal_Props {
    is_open: boolean;
    type: string;
    data: Skill_Definition | Workflow_Definition | Hook_Definition | null;
    preview: string;
    on_close: () => void;
    on_confirm: (final_data: Skill_Definition | Workflow_Definition | Hook_Definition, category: string) => void;
}

/**
 * Import_Preview_Modal
 * Provides a structured preview of parsed skills, workflows, or hooks.
 * Allows users to edit fields and choose the target category (User vs AI).
 */
export const Import_Preview_Modal: React.FC<Import_Preview_Modal_Props> = ({
    is_open,
    type,
    data,
    preview,
    on_close,
    on_confirm,
}) => {
    const [category, set_category] = useState<'user' | 'ai'>('user');
    const [editable_data, set_editable_data] = useState<Skill_Definition | Workflow_Definition | Hook_Definition | null>(data);

    if (!is_open || !data || !editable_data) return null;

    const handle_backdrop_click = (e: React.MouseEvent) => {
        if (e.target === e.currentTarget) on_close();
    };

    const update_data = (updates: Partial<Skill_Definition & Workflow_Definition & Hook_Definition>) => {
        set_editable_data(prev => prev ? ({ ...prev, ...updates } as Skill_Definition | Workflow_Definition | Hook_Definition) : null);
    };

    return (
        <div 
            className="fixed inset-0 z-[10000] flex items-center justify-center bg-black/60 backdrop-blur-sm animate-in fade-in duration-200"
            onClick={handle_backdrop_click}
            onKeyDown={(e) => e.key === 'Escape' && on_close()}
            role="dialog"
            aria-modal="true"
            aria-labelledby="import-modal-title"
            tabIndex={-1}
        >
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl shadow-2xl w-[95%] max-w-2xl max-h-[85vh] flex flex-col overflow-hidden animate-in zoom-in-95 duration-200">
                {/* Header */}
                <div className="px-6 py-4 border-b border-zinc-800 flex justify-between items-center bg-zinc-900/50 backdrop-blur-xl">
                    <div>
                        <h3 id="import-modal-title" className="text-lg font-bold text-zinc-100 flex items-center gap-2">
                            <span className="text-indigo-400">📦</span> {i18n.t('import.title', { type: type.toUpperCase() })}
                        </h3>
                        <p className="text-[10px] text-zinc-500 uppercase font-mono tracking-wider mt-0.5">{i18n.t('import.subtitle')}</p>
                    </div>
                    <button 
                        onClick={on_close} 
                        className="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-zinc-800 text-zinc-500 hover:text-zinc-300 transition-colors"
                        aria-label={i18n.t('common.dismiss')}
                    >
                        ✕
                    </button>
                </div>
                
                {/* Content */}
                <div className="p-6 overflow-y-auto space-y-6 flex-1 scrollbar-thin scrollbar-thumb-zinc-800">
                    {/* Category Selection */}
                    <div className="space-y-3 p-4 bg-zinc-950/50 border border-zinc-800 rounded-lg">
                        <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest">{i18n.t('import.target_hub')}</label>
                        <div className="flex gap-2">
                            <button 
                                onClick={() => set_category('user')}
                                className={`flex-1 px-4 py-3 rounded-lg border text-xs font-bold transition-all flex flex-col items-center gap-1 ${
                                    category === 'user' 
                                    ? 'bg-indigo-500/10 border-indigo-500/50 text-indigo-400 shadow-[0_0_15px_rgba(99,102,241,0.1)]' 
                                    : 'bg-zinc-900/50 border-zinc-800 text-zinc-500 hover:border-zinc-700'
                                }`}
                                aria-pressed={category === 'user'}
                            >
                                <span>{i18n.t('import.user_registry')}</span>
                                <span className="text-[9px] font-normal opacity-60">{i18n.t('import.user_registry_desc')}</span>
                            </button>
                            <button 
                                onClick={() => set_category('ai')}
                                className={`flex-1 px-4 py-3 rounded-lg border text-xs font-bold transition-all flex flex-col items-center gap-1 ${
                                    category === 'ai' 
                                    ? 'bg-emerald-500/10 border-emerald-500/50 text-emerald-400 shadow-[0_0_15px_rgba(16,185,129,0.1)]' 
                                    : 'bg-zinc-900/50 border-zinc-800 text-zinc-500 hover:border-zinc-700'
                                }`}
                                aria-pressed={category === 'ai'}
                            >
                                <span>{i18n.t('import.ai_services')}</span>
                                <span className="text-[9px] font-normal opacity-60">{i18n.t('import.ai_services_desc')}</span>
                            </button>
                        </div>
                    </div>

                    <div className="space-y-4">
                        <div className="grid grid-cols-2 gap-4">
                            <div className="space-y-1.5">
                                <label htmlFor="import-name" className="text-[10px] font-bold text-zinc-500 uppercase tracking-wider">{i18n.t('import.label_name')}</label>
                                <input 
                                    id="import-name"
                                    className="w-full bg-black/40 border border-zinc-800 rounded-lg p-2.5 text-sm text-zinc-200 focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/20 transition-all"
                                    value={editable_data.name || ''}
                                    onChange={e => update_data({ name: e.target.value })}
                                    placeholder={i18n.t('import.placeholder_name')}
                                />
                            </div>
                            {(type === 'skill' || type === 'hook') && (
                                <div className="space-y-1.5">
                                    <label htmlFor="import-exec" className="text-[10px] font-bold text-zinc-500 uppercase tracking-wider">
                                        {type === 'skill' ? i18n.t('import.label_exec_cmd') : i18n.t('import.label_hook_type')}
                                    </label>
                                    <input 
                                        id="import-exec"
                                        className="w-full bg-black/40 border border-zinc-800 rounded-lg p-2.5 text-sm text-zinc-200 focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/20 transition-all font-mono"
                                        value={type === 'skill' ? ((editable_data as Skill_Definition).execution_command || '') : ((editable_data as Hook_Definition).hook_type || '')}
                                        onChange={e => update_data({
                                            [type === 'skill' ? 'execution_command' : 'hook_type']: e.target.value
                                        })}
                                        placeholder={type === 'skill' ? i18n.t('import.placeholder_exec_cmd') : i18n.t('import.placeholder_hook_type')}
                                    />
                                </div>
                            )}
                        </div>

                        {(type === 'skill' || type === 'hook') && (
                            <div className="space-y-1.5">
                                <label htmlFor="import-desc" className="text-[10px] font-bold text-zinc-500 uppercase tracking-wider">{i18n.t('import.label_description')}</label>
                                <textarea 
                                    id="import-desc"
                                    className="w-full bg-black/40 border border-zinc-800 rounded-lg p-2.5 text-sm text-zinc-300 min-h-[80px] focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/20 transition-all"
                                    value={(editable_data as Skill_Definition | Hook_Definition).description || ''}
                                    onChange={e => update_data({ description: e.target.value })}
                                    placeholder={i18n.t('import.placeholder_description')}
                                />
                            </div>
                        )}

                        <div className="space-y-1.5">
                            <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-wider">{i18n.t('import.label_preview')}</label>
                            <div className="relative group">
                                <pre className="w-full bg-black/60 border border-zinc-800 rounded-lg p-4 text-[11px] text-zinc-400 font-mono overflow-auto max-h-[250px] scrollbar-thin scrollbar-thumb-zinc-800">
                                    {preview}
                                </pre>
                                <div className="absolute top-2 right-2 px-2 py-1 rounded bg-zinc-800/80 text-[9px] font-bold text-zinc-500 opacity-0 group-hover:opacity-100 transition-opacity uppercase pointer-events-none">
                                    {i18n.t('import.label_read_only')}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Actions */}
                <div className="px-6 py-5 border-t border-zinc-800 flex justify-end items-center gap-4 bg-zinc-900/50">
                    <button 
                        onClick={on_close} 
                        className="px-4 py-2 text-xs font-bold text-zinc-500 hover:text-zinc-200 transition-colors uppercase tracking-widest"
                    >
                        {i18n.t('import.btn_discard')}
                    </button>
                    <button 
                        onClick={() => on_confirm(editable_data, category)}
                        className="px-8 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-bold shadow-lg shadow-indigo-500/20 active:scale-[0.98] transition-all uppercase tracking-widest flex items-center gap-2"
                    >
                        <span>{i18n.t('import.btn_confirm')}</span>
                        <span className="text-[10px] opacity-70">⇢</span>
                    </button>
                </div>
            </div>
        </div>
    );
};
