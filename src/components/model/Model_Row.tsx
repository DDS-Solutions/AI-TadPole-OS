/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Granular model configuration node. 
 * Facilitates real-time parameter tweaking (RPM/TPM) and status toggling (Active/Benched) with immediate store persistence.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Inline edit sync failure if validation rejects a numeric limit, or modality icon mismatch.
 * - **Telemetry Link**: Search for `[Model_Row]` or `update_model_limit` in service logs.
 */

import { useState, useEffect } from 'react';
import { Edit2, Trash2, Check, X } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Model_Entry, Provider_Config } from '../../stores/provider_store';
import { ModalityBadge } from './Modality_Badge';

interface Model_Row_Props {
    model: Model_Entry;
    is_editing: boolean;
    on_edit: () => void;
    on_save: (name: string, prov: string, modality: Model_Entry['modality'], limits: Record<string, number>) => void;
    on_cancel: () => void;
    on_delete: () => void;
    providers: Provider_Config[];
}

export function Model_Row({ 
    model, 
    is_editing, 
    on_edit, 
    on_save, 
    on_cancel, 
    on_delete, 
    providers 
}: Model_Row_Props) {
    const [edit_state, set_edit_state] = useState({
        name: model.name,
        provider: model.provider,
        modality: model.modality || 'llm',
        is_custom_modality: !['llm', 'vision', 'voice', 'reasoning'].includes(model.modality || 'llm'),
        custom_modality: model.modality || '',
        limits: {
            rpm: model.rpm || 10,
            tpm: model.tpm || 100000,
            rpd: model.rpd || 1000,
            tpd: model.tpd || 10000000
        }
    });
    const [show_limits, set_show_limits] = useState(false);

    // Resets form state when edit mode is entered so edits always start from the
    // latest persisted model values. This is the canonical "controlled form reset"
    // pattern — the effect fires once on the is_editing→true transition.
    useEffect(() => {
        if (is_editing) {
            // eslint-disable-next-line react-hooks/set-state-in-effect
            set_edit_state({
                name: model.name,
                provider: model.provider,
                modality: model.modality || 'llm',
                is_custom_modality: !['llm', 'vision', 'voice', 'reasoning'].includes(model.modality || 'llm'),
                custom_modality: model.modality || '',
                limits: {
                    rpm: model.rpm || 10,
                    tpm: model.tpm || 100000,
                    rpd: model.rpd || 1000,
                    tpd: model.tpd || 10000000
                }
            });
        }
    }, [is_editing, model]);

    if (is_editing) {
        return (
            <>
                <tr className="bg-green-500/[0.03]">
                    <td className="px-8 py-4">
                        <input
                            className="bg-zinc-950 border border-green-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 w-full focus:outline-none focus:border-green-500 font-mono"
                            value={edit_state.name}
                            onChange={e => set_edit_state({ ...edit_state, name: e.target.value })}
                        />
                    </td>
                    <td className="px-8 py-4">
                        <select
                            className="bg-zinc-950 border border-green-500/30 rounded-lg px-3 py-1.5 text-[10px] text-zinc-400 w-full focus:outline-none cursor-pointer uppercase font-bold"
                            value={edit_state.is_custom_modality ? 'other' : edit_state.modality}
                            onChange={e => {
                                if (e.target.value === 'other') {
                                    set_edit_state({ ...edit_state, is_custom_modality: true });
                                } else {
                                    set_edit_state({ ...edit_state, is_custom_modality: false, modality: e.target.value as Model_Entry['modality'] });
                                }
                            }}
                        >
                            <option value="llm">{i18n.t('provider.label_modality_llm')}</option>
                            <option value="vision">{i18n.t('provider.label_modality_vision')}</option>
                            <option value="voice">{i18n.t('provider.label_modality_voice')}</option>
                            <option value="reasoning">{i18n.t('provider.label_modality_reasoning')}</option>
                            <option value="other">{i18n.t('provider.label_modality_other')}</option>
                        </select>
                        {edit_state.is_custom_modality && (
                            <input
                                className="mt-2 bg-zinc-950 border border-green-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 w-full focus:outline-none focus:border-green-500 font-mono"
                                placeholder={i18n.t('provider.label_modality_other')}
                                value={edit_state.custom_modality}
                                onChange={e => set_edit_state({ ...edit_state, custom_modality: e.target.value as Model_Entry['modality'] })}
                            />
                        )}
                    </td>
                    <td className="px-8 py-4">
                        <select
                            className="bg-zinc-950 border border-green-500/30 rounded-lg px-3 py-1.5 text-[10px] text-zinc-400 w-full focus:outline-none cursor-pointer uppercase font-bold"
                            value={edit_state.provider}
                            onChange={e => set_edit_state({ ...edit_state, provider: e.target.value })}
                        >
                            {providers.map(p => <option key={p.id} value={p.id}>{p.name.toUpperCase()}</option>)}
                        </select>
                    </td>
                    <td className="px-8 py-4 text-right">
                        <div className="flex items-center justify-end gap-3">
                            <button
                                onClick={() => {
                                    const final_modality = (edit_state.is_custom_modality ? edit_state.custom_modality : edit_state.modality) as Model_Entry['modality'];
                                    on_save(edit_state.name, edit_state.provider, final_modality, edit_state.limits);
                                }}
                                aria-label={i18n.t('provider.forge_item.tooltip_save')}
                                className="p-1.5 bg-green-500/20 text-green-400 rounded hover:bg-green-500/30 transition-all"
                            >
                                <Check size={14} />
                            </button>
                            <button onClick={on_cancel} aria-label={i18n.t('provider.forge_item.tooltip_cancel')} className="p-1.5 bg-zinc-800 text-zinc-500 rounded hover:bg-zinc-700 transition-all"><X size={14} /></button>
                        </div>
                    </td>
                </tr>
                <tr className="bg-green-500/[0.02]">
                    <td colSpan={4} className="px-12 py-5 border-b border-green-500/10">
                        <div className="grid grid-cols-4 gap-8">
                            <div className="space-y-1.5">
                                <label className="text-[9px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('model_manager.row.req_min')}</label>
                                <input
                                    type="number"
                                    min="0"
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-xs text-zinc-400 font-mono focus:outline-none focus:border-green-500/50"
                                    value={edit_state.limits.rpm}
                                    onChange={e => set_edit_state({ ...edit_state, limits: { ...edit_state.limits, rpm: parseInt(e.target.value) || 0 } })}
                                />
                            </div>
                            <div className="space-y-1.5">
                                <label className="text-[9px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('model_manager.row.tkn_min')}</label>
                                <input
                                    type="number"
                                    min="0"
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-xs text-zinc-400 font-mono focus:outline-none focus:border-green-500/50"
                                    value={edit_state.limits.tpm}
                                    onChange={e => set_edit_state({ ...edit_state, limits: { ...edit_state.limits, tpm: parseInt(e.target.value) || 0 } })}
                                />
                            </div>
                            <div className="space-y-1.5">
                                <label className="text-[9px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('model_manager.row.req_day')}</label>
                                <input
                                    type="number"
                                    min="0"
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-xs text-zinc-400 font-mono focus:outline-none focus:border-green-500/50"
                                    value={edit_state.limits.rpd}
                                    onChange={e => set_edit_state({ ...edit_state, limits: { ...edit_state.limits, rpd: parseInt(e.target.value) || 0 } })}
                                />
                            </div>
                            <div className="space-y-1.5">
                                <label className="text-[9px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('model_manager.row.tkn_day')}</label>
                                <input
                                    type="number"
                                    min="0"
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-xs text-zinc-400 font-mono focus:outline-none focus:border-green-500/50"
                                    value={edit_state.limits.tpd}
                                    onChange={e => set_edit_state({ ...edit_state, limits: { ...edit_state.limits, tpd: parseInt(e.target.value) || 0 } })}
                                />
                            </div>
                        </div>
                    </td>
                </tr>
            </>
        );
    }

    return (
        <>
            <tr className="hover:bg-zinc-900/60 transition-all group border-b border-zinc-800/20 last:border-none">
                <td className="px-8 py-5">
                    <div className="flex items-center gap-3">
                        <div className="w-1.5 h-1.5 rounded-full bg-green-500/40 group-hover:bg-green-400 transition-colors shadow-[0_0_5px_rgba(59,130,246,0.2)]" />
                        <div className="flex flex-col">
                            <span className="text-xs font-bold text-zinc-200 uppercase tracking-tight">{model.name}</span>
                            <button
                                onClick={() => set_show_limits(!show_limits)}
                                className="text-[11px] text-zinc-600 hover:text-green-400 font-bold mt-1 transition-colors flex items-center gap-1 uppercase tracking-widest"
                                aria-expanded={show_limits}
                                aria-controls={`limits-${model.id}`}
                            >
                                {show_limits ? i18n.t('model_manager.row.hide_limits') : i18n.t('model_manager.row.show_limits')}
                            </button>
                        </div>
                    </div>
                </td>
                <td className="px-8 py-5">
                    <ModalityBadge modality={model.modality || 'llm'} />
                </td>
                <td className="px-8 py-5">
                    <span className="text-[11px] font-bold font-mono text-zinc-500 uppercase tracking-tighter bg-zinc-900 border border-white/5 px-2 py-0.5 rounded">
                        {model.provider}
                    </span>
                </td>
                <td className="px-8 py-5 text-right">
                    <div className="flex items-center justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all transform translate-x-2 group-hover:translate-x-0">
                        <Tooltip content={i18n.t('model_manager.row.tooltip_edit')} position="top">
                            <button onClick={on_edit} aria-label={i18n.t('model_manager.row.tooltip_edit')} className="p-1.5 rounded-md hover:bg-zinc-800 text-zinc-600 hover:text-zinc-100 transition-colors">
                                <Edit2 size={13} />
                            </button>
                        </Tooltip>
                        <Tooltip content={i18n.t('model_manager.row.tooltip_delete')} position="top">
                            <button
                                onClick={(e) => {
                                    e.stopPropagation();
                                    on_delete();
                                }}
                                aria-label={i18n.t('model_manager.row.tooltip_delete')}
                                className="p-1.5 rounded-lg hover:bg-red-500/10 text-zinc-500 hover:text-red-400 transition-all active:scale-95"
                            >
                                <Trash2 size={16} />
                            </button>
                        </Tooltip>
                    </div>
                </td>
            </tr>
            {show_limits && (
                <tr id={`limits-${model.id}`} className="bg-zinc-950/80 border-b border-zinc-900/40">
                    <td colSpan={4} className="px-12 py-5 animate-in slide-in-from-top-1 duration-200">
                        <div className="grid grid-cols-4 gap-8">
                            <div className="space-y-1">
                                <div className="text-[8px] font-bold text-zinc-600 uppercase tracking-[0.2em]">{i18n.t('model_manager.row.req_min')}</div>
                                <div className="text-xs font-mono text-zinc-400">{model.rpm || 10}</div>
                            </div>
                            <div className="space-y-1">
                                <div className="text-[8px] font-bold text-zinc-600 uppercase tracking-[0.2em]">{i18n.t('model_manager.row.tkn_min')}</div>
                                <div className="text-xs font-mono text-zinc-400">{(model.tpm || 100000).toLocaleString()}</div>
                            </div>
                            <div className="space-y-1">
                                <div className="text-[8px] font-bold text-zinc-600 uppercase tracking-[0.2em]">{i18n.t('model_manager.row.req_day')}</div>
                                <div className="text-xs font-mono text-zinc-400">{(model.rpd || 1000).toLocaleString()}</div>
                            </div>
                            <div className="space-y-1">
                                <div className="text-[8px] font-bold text-zinc-600 uppercase tracking-[0.2em]">{i18n.t('model_manager.row.tkn_day')}</div>
                                <div className="text-xs font-mono text-zinc-400">{(model.tpd || 10000000).toLocaleString()}</div>
                            </div>
                        </div>
                    </td>
                </tr>
            )}
        </>
    );
}


// Metadata: [Model_Row]
