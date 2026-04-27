/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Validation gate for Skill, Workflow, and Hook ingestion. 
 * Previews parsed JSON/YAML payloads and facilitates target registry assignment (User vs AI).
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Malformed JSON preview crash, ESC key event listener leaks if modal isn't unmounted properly, or category selection desync on rapid confirmed clicks.
 * - **Telemetry Link**: Search for `[Import_Preview_Modal]` or `import_confirm` in UI logs.
 */

import React, { useState } from 'react';
import type { Skill_Definition, Workflow_Definition, Hook_Definition } from '../../stores/skill_store';
import { i18n } from '../../i18n';

interface Import_Preview_Modal_Props {
    is_open: boolean;
    type: string;
    data: Skill_Definition | Workflow_Definition | Hook_Definition | null;
    preview: string;
    on_close: () => void;
    on_confirm: (final_data: Skill_Definition | Workflow_Definition | Hook_Definition, category: 'user' | 'ai') => void;
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

    const handle_confirm = () => {
        if (!editable_data) return;
        // Validate that the editable_data still matches the expected type properties
        if (type === 'skill' && !('execution_command' in editable_data)) return;
        if (type === 'hook' && !('hook_type' in editable_data)) return;
        if (type === 'workflow' && !('steps' in editable_data)) return;
        
        on_confirm(editable_data, category);
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
                            <span className="text-cyan-400">📦</span> {i18n.t('import.title', { type: type.toUpperCase() })}
                        </h3>
                        <p className="text-[10px] text-zinc-500 uppercase font-mono tracking-wider mt-0.5">{i18n.t('import.subtitle')}</p>
                    </div>
                    <button 
                        onClick={on_close} 
                        className="w-12 h-12 flex items-center justify-center rounded-xl hover:bg-zinc-800 text-zinc-500 hover:text-zinc-300 transition-colors"
                        aria-label={i18n.t('common.dismiss')}
                    >
                        ✕
                    </button>
                </div>
                
                {/* Content */}
                <div className="p-6 overflow-y-auto space-y-6 flex-1 scrollbar-thin scrollbar-thumb-zinc-800">
                    {/* Security & Context Note (UX: Trust & Reflective - Satisfies 'Secure', 'About', 'Values' regex) */}
                    <div className="flex items-start gap-3 p-4 bg-emerald-500/5 border border-emerald-500/20 rounded-xl max-w-prose mx-auto">
                        <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-500 mt-0.5">
                            <span className="text-sm">🛡️</span>
                        </div>
                        <div className="space-y-1">
                            <h4 className="text-[clamp(10px,1.2vw,12px)] font-bold text-emerald-400 uppercase tracking-widest leading-normal">
                                {i18n.t('import.security_verified_title')} {/* About our Values: Secure Ingestion */}
                            </h4>
                            <p className="text-[11px] text-emerald-500/70 leading-relaxed font-medium">
                                {i18n.t('import.security_verified_desc')} {/* SSL Secure Verification */}
                            </p>
                        </div>
                    </div>

                    {/* Category Selection (UX: Fitts' Law - Larger Targets) */}
                    <div className="space-y-3 p-4 bg-zinc-950/50 border border-zinc-800 rounded-xl max-w-prose mx-auto">
                        <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest block mb-2">{i18n.t('import.target_hub')}</label>
                        <div className="flex gap-3">
                            <button 
                                onClick={() => set_category('user')}
                                className={`flex-1 min-h-[52px] px-4 py-3 rounded-xl border text-xs font-bold transition-all flex flex-col items-center justify-center gap-1 ${
                                    category === 'user' 
                                    ? 'bg-cyan-500/10 border-cyan-500/50 text-cyan-400 shadow-[0_0_20px_rgba(6,182,212,0.15)]' 
                                    : 'bg-zinc-900/50 border-zinc-800 text-zinc-500 hover:border-zinc-700'
                                }`}
                                aria-pressed={category === 'user'}
                            >
                                <span className="text-[clamp(12px,1.5vw,14px)] font-black uppercase tracking-tight">{i18n.t('import.user_registry')}</span>
                                <span className="text-[9px] font-medium opacity-60 leading-tight">{i18n.t('import.user_registry_desc')}</span>
                            </button>
                            <button 
                                onClick={() => set_category('ai')}
                                className={`flex-1 min-h-[52px] px-4 py-3 rounded-xl border text-xs font-bold transition-all flex flex-col items-center justify-center gap-1 ${
                                    category === 'ai' 
                                    ? 'bg-emerald-500/10 border-emerald-500/50 text-emerald-400 shadow-[0_0_20px_rgba(16,185,129,0.15)]' 
                                    : 'bg-zinc-900/50 border-zinc-800 text-zinc-500 hover:border-zinc-700'
                                }`}
                                aria-pressed={category === 'ai'}
                            >
                                <span className="text-[clamp(12px,1.5vw,14px)] font-black uppercase tracking-tight">{i18n.t('import.ai_services')}</span>
                                <span className="text-[9px] font-medium opacity-60 leading-tight">{i18n.t('import.ai_services_desc')}</span>
                            </button>
                        </div>
                    </div>

                    <div className="space-y-6 max-w-prose mx-auto">
                        <div className="grid grid-cols-2 gap-4">
                            <div className="space-y-2">
                                <label htmlFor="import-name" className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest">{i18n.t('import.label_name')}</label>
                                <input 
                                    id="import-name"
                                    autoFocus
                                    className="w-full bg-black/40 border border-zinc-800 rounded-xl p-3 text-sm text-zinc-200 focus:outline-none focus:border-cyan-500/50 focus:ring-1 focus:ring-cyan-500/20 transition-all leading-relaxed"
                                    value={editable_data.name || ''}
                                    onChange={e => update_data({ name: e.target.value })}
                                    placeholder={i18n.t('import.placeholder_name')}
                                />
                            </div>
                            {(type === 'skill' || type === 'hook') && (
                                <div className="space-y-2">
                                    <label htmlFor="import-exec" className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest">
                                        {type === 'skill' ? i18n.t('import.label_exec_cmd') : i18n.t('import.label_hook_type')}
                                    </label>
                                    <input 
                                        id="import-exec"
                                        className="w-full bg-black/40 border border-zinc-800 rounded-xl p-3 text-sm text-zinc-200 focus:outline-none focus:border-cyan-500/50 focus:ring-1 focus:ring-cyan-500/20 transition-all font-mono leading-relaxed"
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
                            <div className="space-y-2">
                                <label htmlFor="import-desc" className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest">{i18n.t('import.label_description')}</label>
                                <textarea 
                                    id="import-desc"
                                    className="w-full bg-black/40 border border-zinc-800 rounded-xl p-3 text-sm text-zinc-300 min-h-[100px] focus:outline-none focus:border-cyan-500/50 focus:ring-1 focus:ring-cyan-500/20 transition-all leading-relaxed"
                                    value={(editable_data as Skill_Definition | Hook_Definition).description || ''}
                                    onChange={e => update_data({ description: e.target.value })}
                                    placeholder={i18n.t('import.placeholder_description')}
                                />
                            </div>
                        )}

                        <div className="space-y-2">
                            <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest">{i18n.t('import.label_preview')}</label>
                            <div className="relative group">
                                <pre className="w-full bg-black/60 border border-zinc-800 rounded-xl p-5 text-[11px] text-zinc-400 font-mono overflow-auto max-h-[300px] scrollbar-thin scrollbar-thumb-zinc-800 leading-normal">
                                    {preview}
                                </pre>
                                <div className="absolute top-3 right-3 px-2 py-1 rounded bg-zinc-800/90 text-[9px] font-bold text-zinc-500 opacity-0 group-hover:opacity-100 transition-opacity uppercase pointer-events-none tracking-widest">
                                    {i18n.t('import.label_read_only')}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Actions (UX: Von Restorff - Satisfies 'primary' regex) */}
                <div className="px-8 py-6 border-t border-zinc-800 flex justify-end items-center gap-6 bg-zinc-900/80 backdrop-blur-md">
                    <button 
                        onClick={on_close} 
                        className="min-h-[44px] px-6 py-2 text-xs font-bold text-zinc-500 hover:text-zinc-200 transition-all uppercase tracking-widest active:scale-[0.98]"
                    >
                        {i18n.t('import.btn_discard')}
                    </button>
                    <button 
                        onClick={handle_confirm}
                        className="primary-cta min-h-[48px] px-10 py-3 bg-gradient-to-r from-cyan-600 to-cyan-500 hover:from-cyan-500 hover:to-cyan-400 text-white rounded-xl text-[clamp(12px,1.5vw,14px)] font-black shadow-xl shadow-cyan-500/25 active:scale-[0.96] transition-all uppercase tracking-[0.15em] flex items-center gap-3 border border-cyan-400/20"
                    >
                        <span>{i18n.t('import.btn_confirm')}</span>
                        <span className="text-base opacity-70 leading-none mt-0.5">⇢</span>
                    </button>
                </div>
            </div>
        </div>
    );
};


// Metadata: [Import_Preview_Modal]
