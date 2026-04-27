/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Intelligence Forge item renderer. 
 * Orchestrates inline property editing for model forge catalog entries and facilitates real-time modality switching (Vision/Voice/Reasoning).
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Edit name length overflow, numeric limit validation rejection (negative values), or modality icon mismatch during hot-swaps.
 * - **Telemetry Link**: Search for `[Forge_Item]` or `save_forge_item` in browser logs.
 */

import React, { useState } from 'react';
import { Layers, Activity, Info, Zap, Edit2, Trash2, Check, X } from 'lucide-react';
import { i18n } from '../../i18n';
import { type Model_Entry } from '../../stores/model_store';
import { Tooltip } from '../ui';

/**
 * Forge_Item_Props
 * Defines the interface for the Forge_Item component.
 * Refactored for strict snake_case compliance for backend parity.
 */
interface Forge_Item_Props {
    model: Model_Entry;
    is_editing: boolean;
    on_edit: () => void;
    on_cancel: () => void;
    on_save: (id: string, name: string, prov: string, modality: Model_Entry['modality'], limits: Omit<Model_Entry, 'id' | 'name' | 'provider' | 'modality'>) => void;
    on_delete: () => void;
}

/**
 * Forge_Item
 * An individual model entry within the Intelligence Forge catalog.
 */
export function Forge_Item({ model, is_editing, on_edit, on_cancel, on_save, on_delete }: Forge_Item_Props): React.ReactElement {
    const [edit_name, set_edit_name] = useState(model.name);
    const [edit_modality, set_edit_modality] = useState<Model_Entry['modality']>(model.modality || 'llm');
    const [is_custom_modality, set_is_custom_modality] = useState(!['llm', 'vision', 'voice', 'reasoning'].includes(model.modality || 'llm'));
    const [custom_modality, set_custom_modality] = useState(model.modality || '');
    const [limits, set_limits] = useState({
        rpm: model.rpm || 10,
        tpm: model.tpm || 100000,
        rpd: model.rpd || 1000,
        tpd: model.tpd || 10000000
    });



    if (is_editing) {
        return (
            <div className="p-4 bg-emerald-500/[0.03] space-y-4 animate-in fade-in duration-300">
                <div className="flex gap-3">
                    <input
                        className="bg-zinc-950 border border-emerald-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 flex-1 focus:outline-none focus:border-emerald-500 font-mono"
                        value={edit_name}
                        onChange={e => set_edit_name(e.target.value)}
                        aria-label={i18n.t('provider.forge_name_label')}
                    />
                    <select
                        className="bg-zinc-950 border border-emerald-500/30 rounded-lg px-3 py-1.5 text-[11px] text-zinc-400 focus:outline-none uppercase font-bold"
                        value={is_custom_modality ? 'other' : edit_modality}
                        onChange={e => {
                            if (e.target.value === 'other') {
                                set_is_custom_modality(true);
                            } else {
                                set_is_custom_modality(false);
                                set_edit_modality(e.target.value as Model_Entry['modality']);
                            }
                        }}
                        aria-label={i18n.t('provider.forge_modality_label')}
                    >
                        <option value="llm">{i18n.t('provider.label_modality_llm')}</option>
                        <option value="vision">{i18n.t('provider.label_modality_vision')}</option>
                        <option value="voice">{i18n.t('provider.label_modality_voice')}</option>
                        <option value="reasoning">{i18n.t('provider.label_modality_reasoning')}</option>
                        <option value="other">{i18n.t('provider.label_modality_other')}</option>
                    </select>
                </div>
                {is_custom_modality && (
                    <input
                        className="bg-zinc-950 border border-emerald-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 w-full focus:outline-none focus:border-emerald-500 font-mono"
                        placeholder={i18n.t('provider.placeholder_custom_modality')}
                        value={custom_modality}
                        onChange={e => set_custom_modality(e.target.value as Model_Entry['modality'])}
                        aria-label={i18n.t('provider.custom_modality_label')}
                    />
                )}

                <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-1">
                        <label className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1 flex items-center gap-1.5">
                            {i18n.t('provider.forge_item.label_rpm')}
                            <Tooltip content={i18n.t('provider.forge_item.tooltip_rpm')} position="top">
                                <Info size={9} className="text-zinc-700 hover:text-emerald-400 cursor-help transition-colors" />
                            </Tooltip>
                        </label>
                        <input
                            type="number"
                            min="0"
                            step="1"
                            className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-emerald-500/40 outline-none"
                            value={limits.rpm}
                            onChange={e => set_limits({ ...limits, rpm: Math.max(0, e.target.valueAsNumber || 0) })}
                            aria-label={i18n.t('provider.field_rpm_label')}
                        />
                    </div>
                    <div className="space-y-1">
                        <label className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1 flex items-center gap-1.5">
                            {i18n.t('provider.forge_item.label_tpm')}
                            <Tooltip content={i18n.t('provider.forge_item.tooltip_tpm')} position="top">
                                <Info size={9} className="text-zinc-700 hover:text-emerald-400 cursor-help transition-colors" />
                            </Tooltip>
                        </label>
                        <input
                            type="number"
                            min="0"
                            step="1"
                            className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-emerald-500/40 outline-none"
                            value={limits.tpm}
                            onChange={e => set_limits({ ...limits, tpm: Math.max(0, e.target.valueAsNumber || 0) })}
                            aria-label={i18n.t('provider.field_tpm_label')}
                        />
                    </div>
                    <div className="space-y-1">
                        <label className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('provider.forge_item.label_rpd')}</label>
                        <input
                            type="number"
                            min="0"
                            step="1"
                            className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-emerald-500/40 outline-none"
                            value={limits.rpd}
                            onChange={e => set_limits({ ...limits, rpd: Math.max(0, e.target.valueAsNumber || 0) })}
                            aria-label={i18n.t('provider.field_rpd_label')}
                        />
                    </div>
                    <div className="space-y-1">
                        <label className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('provider.forge_item.label_tpd')}</label>
                        <input
                            type="number"
                            min="0"
                            step="1"
                            className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-emerald-500/40 outline-none"
                            value={limits.tpd}
                            onChange={e => set_limits({ ...limits, tpd: Math.max(0, e.target.valueAsNumber || 0) })}
                            aria-label={i18n.t('provider.field_tpd_label')}
                        />
                    </div>
                </div>

                <div className="flex justify-end gap-2 pt-2 border-t border-zinc-800/50">
                    <Tooltip content={i18n.t('provider.forge_item.tooltip_save')} position="bottom">
                        <button
                            onClick={() => {
                                const final_modality = is_custom_modality ? custom_modality : edit_modality;
                                on_save(model.id, edit_name, model.provider, final_modality as Model_Entry['modality'], limits);
                            }}
                            className="p-1.5 bg-emerald-500/20 text-emerald-400 rounded hover:bg-emerald-500/30 transition-colors"
                            aria-label={i18n.t('provider.aria_save_node')}
                        >
                            <Check size={14} />
                        </button>
                    </Tooltip>
                    <Tooltip content={i18n.t('provider.forge_item.tooltip_cancel')} position="bottom">
                        <button 
                            onClick={on_cancel} 
                            className="p-1.5 bg-zinc-800 text-zinc-500 rounded hover:bg-zinc-700 transition-colors"
                            aria-label={i18n.t('provider.aria_cancel_node')}
                        >
                            <X size={14} />
                        </button>
                    </Tooltip>
                </div>
            </div>
        );
    }

    return (
        <div className="p-3 px-4 flex items-center justify-between group hover:bg-zinc-900/50 transition-all">
            <div className="flex items-center gap-3">
                <div className={`p-1.5 rounded-lg border flex items-center justify-center transition-colors ${model.modality === 'vision' ? 'bg-amber-500/10 border-amber-500/20 text-amber-500' :
                    model.modality === 'voice' ? 'bg-green-500/10 border-green-500/20 text-green-500' :
                        model.modality === 'reasoning' ? 'bg-green-500/10 border-green-500/20 text-green-500' :
                            'bg-zinc-900 border-zinc-800 text-zinc-500 group-hover:text-emerald-500'
                    }`}>
                    {model.modality === 'vision' ? <Activity size={12} /> :
                        model.modality === 'voice' ? <Info size={12} /> :
                            model.modality === 'reasoning' ? <Zap size={12} /> :
                                <Layers size={12} />}
                </div>
                <div className="flex flex-col">
                    <span className="text-xs font-mono font-bold text-zinc-300 group-hover:text-zinc-100 uppercase tracking-tight">
                        {model.name}
                    </span>
                    <div className="flex items-center gap-2 mt-0.5">
                        <span className="text-[10px] text-zinc-600 font-bold uppercase tracking-tighter">
                            {model.modality || 'llm'}
                        </span>
                        <div className="w-1 h-1 rounded-full bg-zinc-800" />
                        <span className="text-[10px] text-zinc-400 font-mono font-bold">
                            {(model.tpm || 100000).toLocaleString()} <span className="text-zinc-600 opacity-80">{i18n.t('common_units.tpm')}</span>
                        </span>
                    </div>
                </div>
            </div>
            <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-all">
                <Tooltip content={i18n.t('provider.forge_item.tooltip_edit')} position="left">
                    <button
                        onClick={on_edit}
                        className="p-1.5 rounded hover:bg-emerald-500/10 text-zinc-700 hover:text-emerald-500"
                    >
                        <Edit2 size={12} />
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('provider.forge_item.tooltip_delete')} position="left">
                    <button
                        onClick={on_delete}
                        className="p-1.5 rounded hover:bg-red-500/10 text-zinc-700 hover:text-red-500"
                    >
                        <Trash2 size={12} />
                    </button>
                </Tooltip>
            </div>
        </div>
    );
}


// Metadata: [Forge_Item]
