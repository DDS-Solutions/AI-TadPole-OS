import React from 'react';
import { Code, AlertTriangle } from 'lucide-react';
import { i18n } from '../../i18n';
import type { Skill_Definition } from '../../stores/skill_store';

interface Skill_Edit_Modal_Props {
    is_open: boolean;
    on_close: () => void;
    editing_skill: Partial<Skill_Definition>;
    set_editing_skill: (skill: Partial<Skill_Definition>) => void;
    schema_error: string | null;
    set_schema_error: (error: string | null) => void;
    skill_save_error: string | null;
    is_saving: boolean;
    on_save: () => void;
}

export const Skill_Edit_Modal: React.FC<Skill_Edit_Modal_Props> = ({
    is_open,
    on_close,
    editing_skill,
    set_editing_skill,
    schema_error,
    set_schema_error,
    skill_save_error,
    is_saving,
    on_save
}) => {
    if (!is_open) return null;

    return (
        <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-4">
            <div className="bg-zinc-950 border border-zinc-800 w-full max-w-2xl rounded-2xl shadow-2xl flex flex-col max-h-[90vh] relative overflow-hidden">
                <div className="neural-grid opacity-10" />
                <div className="p-5 border-b border-zinc-800 flex justify-between items-center shrink-0 relative z-10 bg-zinc-950/50">
                    <h2 className="text-lg font-bold text-zinc-100 flex items-center gap-2">
                        <Code className="text-blue-500" /> {editing_skill?.name ? i18n.t('skills.modal_edit_skill') : i18n.t('skills.modal_create_skill')}
                    </h2>
                    <button onClick={on_close} className="text-zinc-500 hover:text-zinc-300 p-1">✕</button>
                </div>
                <div className="p-6 overflow-y-auto space-y-5 custom-scrollbar relative z-10 bg-zinc-950/80">
                    <div>
                        <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('skills.label_skill_name')}</label>
                        <input
                            type="text"
                            value={editing_skill.name || ''}
                            onChange={e => set_editing_skill({ ...editing_skill, name: e.target.value.replace(/\s+/g, '_') })}
                            className="w-full bg-zinc-900 border border-zinc-800 rounded p-3 text-zinc-200 font-mono text-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all placeholder:text-zinc-700"
                            placeholder="fetch_twitter_data"
                        />
                    </div>
                    <div>
                        <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('skills.label_llm_desc')}</label>
                        <textarea
                            value={editing_skill.description || ''}
                            onChange={e => set_editing_skill({ ...editing_skill, description: e.target.value })}
                            className="w-full bg-zinc-900 border border-zinc-800 rounded p-3 text-zinc-300 focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all resize-y min-h-[80px] text-sm placeholder:text-zinc-700"
                            placeholder="Scrapes a twitter profile..."
                        />
                    </div>
                    <div>
                        <label className="block text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">{i18n.t('skills.label_exec_cmd')}</label>
                        <input
                            type="text"
                            value={editing_skill.execution_command || ''}
                            onChange={e => set_editing_skill({ ...editing_skill, execution_command: e.target.value })}
                            className="w-full bg-zinc-900 border border-zinc-800 rounded p-3 text-emerald-400 font-mono text-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 outline-none transition-all placeholder:text-zinc-700"
                            placeholder="python scripts/fetch.py"
                        />
                        <p className="mt-1.5 text-[9px] text-zinc-600 font-mono leading-relaxed">{i18n.t('skills.hint_exec_cmd')}</p>
                    </div>
                    <div>
                        <label className="flex items-center justify-between text-[10px] text-zinc-500 font-bold mb-2 uppercase tracking-[0.1em]">
                            <span>{i18n.t('skills.label_params_schema')}</span>
                            {schema_error && <p className="mt-1 text-[10px] text-red-400 font-bold">{i18n.t('skills.error_invalid_json')}: {schema_error}</p>}
                        </label>
                        <textarea
                            value={typeof editing_skill.schema === 'string' ? editing_skill.schema : JSON.stringify(editing_skill.schema, null, 2)}
                            onChange={e => {
                                try {
                                    const val = JSON.parse(e.target.value);
                                    set_editing_skill({ ...editing_skill, schema: val });
                                    set_schema_error(null);
                                } catch {
                                    // eslint-disable-next-line @typescript-eslint/no-explicit-any
                                    set_editing_skill({ ...editing_skill, schema: e.target.value as any });
                                    set_schema_error("Invalid JSON");
                                }
                            }}
                            className={`w-full bg-zinc-900 border rounded p-3 font-mono text-xs focus:ring-1 outline-none transition-all resize-y min-h-[150px] custom-scrollbar ${schema_error ? 'border-red-500/50 text-red-400 focus:border-red-500 focus:ring-red-500/50' : 'border-zinc-800 text-zinc-300 focus:border-blue-500 focus:ring-blue-500/50'}`}
                            spellCheck="false"
                        />
                    </div>
                </div>
                <div className="p-5 border-t border-zinc-800 flex justify-end gap-3 shrink-0 relative z-10 bg-zinc-950/90 items-center">
                    {skill_save_error && <div className="text-xs text-red-400 font-mono mr-auto flex items-center gap-1"><AlertTriangle className="w-3.5 h-3.5" /> {skill_save_error}</div>}
                    <button onClick={on_close} className="px-5 py-2 text-xs font-bold text-zinc-500 hover:text-zinc-100 transition-colors">{i18n.t('skills.btn_cancel')}</button>
                    <button onClick={on_save} disabled={is_saving || !!schema_error || !editing_skill.name || !editing_skill.execution_command} className="bg-blue-600 hover:bg-blue-500 text-white px-6 py-2 rounded-lg text-xs font-bold transition-colors shadow-lg shadow-blue-500/20 disabled:opacity-50">{is_saving ? i18n.t('skills.btn_saving') : i18n.t('skills.btn_save_skill')}</button>
                </div>
            </div>
        </div>
    );
};
