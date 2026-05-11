/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Event hook identity card. 
 * Renders trigger conditions and target actions for system hooks, facilitating rapid status monitoring of reactive loops.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Trigger condition text-overflow, or status indicator pulse desync from active hook state.
 * - **Telemetry Link**: Search for `[Hook_Card]` or `hook_pulse` in tracing.
 */

import React from 'react';
import { Anchor, Shield, Trash2, Edit3, Loader2 } from 'lucide-react';
import { type Hook_Definition } from '../../stores/skill_store';
import { Tooltip } from '../ui/Tooltip';
import { i18n } from '../../i18n';

interface Hook_Card_Props {
    hook: Hook_Definition;
    on_edit: (hook: Hook_Definition) => void;
    on_delete: (name: string) => void;
    is_loading?: boolean;
}

export const Hook_Card: React.FC<Hook_Card_Props> = ({ hook, on_edit, on_delete, is_loading }) => {
    return (
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-4 hover:border-zinc-700 transition-all group">
            <div className="flex justify-between items-start">
                <div className="flex items-center gap-3">
                    <div className="p-2 bg-green-500/10 rounded-lg">
                        <Anchor size={18} className="text-green-500" />
                    </div>
                    <div>
                        <h4 className="font-bold text-zinc-100 group-hover:text-green-400 transition-colors uppercase tracking-tight">{hook.name}</h4>
                        <div className="flex items-center gap-2 mt-0.5">
                            <span className="text-[10px] font-mono text-zinc-600 bg-zinc-950 px-1.5 py-0.5 rounded border border-zinc-800 uppercase">{hook.hook_type}</span>
                            {hook.active && (
                                <span className="flex items-center gap-1 text-[10px] font-bold text-emerald-500 uppercase tracking-tighter">
                                    <Shield size={10} /> {i18n.t('skills.active_label')}
                                </span>
                            )}
                        </div>
                    </div>
                </div>

                <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <Tooltip content={i18n.t('skills.edit_hook_tooltip')}>
                        <button
                            onClick={() => on_edit(hook)}
                            className="p-1.5 text-zinc-500 hover:text-zinc-100 hover:bg-zinc-800 rounded-lg transition-all"
                        >
                            <Edit3 size={14} />
                        </button>
                    </Tooltip>
                    <Tooltip content={i18n.t('skills.delete_hook_tooltip')}>
                        <button
                            onClick={() => on_delete(hook.name)}
                            disabled={is_loading}
                            className="p-1.5 text-zinc-500 hover:text-red-500 hover:bg-red-500/10 rounded-lg transition-all disabled:opacity-50"
                        >
                            {is_loading ? <Loader2 size={14} className="animate-spin" /> : <Trash2 size={14} />}
                        </button>
                    </Tooltip>
                </div>
            </div>

            <p className="text-sm text-zinc-400 line-clamp-2 leading-relaxed">
                {hook.description || i18n.t('skills.no_description')}
            </p>

            <div className="pt-2">
                <div className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest mb-2">{i18n.t('skills.preview_trigger_label')}</div>
                <pre className="text-[10px] font-mono text-zinc-500 bg-black/40 p-2.5 rounded-lg border border-zinc-800/50 truncate">
                    {hook.content.split('\n')[0]?.substring(0, 80) || i18n.t('skills.no_preview_available')}
                </pre>
            </div>
        </div>
    );
};


// Metadata: [Hook_Card]
