/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Sub-module**: Swarm intelligence catalog and model forge. 
 * Orchestrates the addition and modification of model endpoints (GPT-4o, Claude 3.5, etc.) linked to the current provider.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Model limit reached, invalid model schema, or unauthorized model access (403).
 * - **Telemetry Link**: Search for `add_model` or `edit_model` in `model_store` logs.
 */

import React, { useState } from 'react';
import { Cpu, Plus, Check, X, Info } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import { type Model_Entry } from '../../stores/model_store';
import { Forge_Item } from './Forge_Item';
import { MODEL_OPTIONS } from '../../data/models';
import { resolve_provider } from '../../utils/model_utils';

/**
 * Model_Forge_Props
 * Defines the props for the Model_Forge component.
 */
interface Model_Forge_Props {
    /** Current provider ID */
    provider_id: string;
    /** Filtered models belonging to this provider */
    provider_models: Model_Entry[];
    /** Callback to add a new model node */
    add_model: (name: string, provider: string, modality: string, limits: Partial<Model_Entry>) => void;
    /** Callback to edit an existing model node */
    edit_model: (id: string, name: string, provider: string, modality: string, limits: Partial<Model_Entry>) => void;
    /** Callback to remove a model node */
    delete_model: (id: string) => void;
}

/**
 * Model_Forge
 * An interactive catalog manager for defining and calibrating provider-specific AI infrastructure nodes.
 */
export function Model_Forge({
    provider_id,
    provider_models,
    add_model,
    edit_model,
    delete_model
}: Model_Forge_Props): React.ReactElement {
    const [is_forge_adding, set_is_forge_adding] = useState(false);
    const [forge_new_model, set_forge_new_model] = useState({
        name: '',
        modality: 'llm' as Model_Entry['modality'],
        rpm: 10,
        tpm: 100000,
        rpd: 1000,
        tpd: 10000000
    });
    const [is_forge_custom_modality, set_is_forge_custom_modality] = useState(false);
    const [forge_custom_modality, set_forge_custom_modality] = useState('');
    const [editing_model_id, set_editing_model_id] = useState<string | null>(null);

    const handle_forge_add = (): void => {
        if (!forge_new_model.name.trim()) return;
        const final_modality = is_forge_custom_modality ? forge_custom_modality : forge_new_model.modality;
        add_model(forge_new_model.name, provider_id, final_modality, {
            rpm: forge_new_model.rpm,
            tpm: forge_new_model.tpm,
            rpd: forge_new_model.rpd,
            tpd: forge_new_model.tpd
        });
        set_forge_new_model({ name: '', modality: 'llm', rpm: 10, tpm: 100000, rpd: 1000, tpd: 10000000 });
        set_is_forge_custom_modality(false);
        set_forge_custom_modality('');
        set_is_forge_adding(false);
    };

    return (
        <section className="space-y-4">
            <div className="flex items-center justify-between">
                <h3 className="text-[11px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                    <Cpu size={12} className="text-blue-400/50" />
                    {i18n.t('provider.intelligence_forge')}
                </h3>
                <Tooltip content={i18n.t('provider.add_node_tooltip')} position="left">
                    <button
                        onClick={() => set_is_forge_adding(true)}
                        className="flex items-center gap-1.5 text-[11px] font-bold text-blue-400 hover:bg-blue-400/10 px-3 py-1.5 rounded-lg border border-blue-400/20 transition-all uppercase tracking-widest"
                        aria-label={i18n.t('provider.aria_add_node')}
                    >
                        <Plus size={12} /> {i18n.t('provider.add_node')}
                    </button>
                </Tooltip>
            </div>

            <div className="bg-zinc-950/50 border border-zinc-800 rounded-2xl overflow-hidden divide-y divide-zinc-800/50">
                {is_forge_adding && (
                    <div className="p-4 bg-blue-500/5 space-y-4 animate-in fade-in slide-in-from-top-2 duration-300">
                        <div className="flex gap-3">
                            <input
                                id="forge-model-name"
                                className="bg-zinc-950 border border-blue-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 flex-1 focus:outline-none focus:border-blue-500 font-mono"
                                placeholder={i18n.t('provider.placeholder_forge_model')}
                                autoFocus
                                list="model-inventory-suggestions"
                                value={forge_new_model.name}
                                onChange={e => set_forge_new_model({ ...forge_new_model, name: e.target.value })}
                                aria-label={i18n.t('provider.forge_name_label')}
                            />
                            <datalist id="model-inventory-suggestions">
                                {MODEL_OPTIONS
                                    .filter(opt => resolve_provider(opt) === provider_id)
                                    .map(opt => (
                                        <option key={opt} value={opt} />
                                    ))}
                            </datalist>
                            <select
                                className="bg-zinc-950 border border-blue-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 focus:outline-none cursor-pointer font-mono"
                                value={is_forge_custom_modality ? 'other' : forge_new_model.modality}
                                onChange={e => {
                                    if (e.target.value === 'other') {
                                        set_is_forge_custom_modality(true);
                                    } else {
                                        set_is_forge_custom_modality(false);
                                        set_forge_new_model({ ...forge_new_model, modality: e.target.value as Model_Entry['modality'] });
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
                        {is_forge_custom_modality && (
                            <input
                                id="forge-custom-modality"
                                className="bg-zinc-950 border border-blue-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 w-full focus:outline-none focus:border-blue-500 font-mono"
                                placeholder={i18n.t('provider.placeholder_custom_modality')}
                                value={forge_custom_modality}
                                onChange={e => set_forge_custom_modality(e.target.value)}
                                aria-label={i18n.t('provider.custom_modality_label')}
                            />
                        )}
                        <div className="grid grid-cols-2 gap-4">
                            <div className="space-y-1">
                                <label htmlFor="forge-rpm" className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1 flex items-center gap-1.5">
                                    {i18n.t('provider.forge_item.label_rpm')}
                                    <Tooltip content={i18n.t('provider.rpm_tooltip')} position="top">
                                        <Info size={9} className="text-zinc-700 hover:text-blue-400 cursor-help transition-colors" />
                                    </Tooltip>
                                </label>
                                <input
                                    id="forge-rpm"
                                    type="number"
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-blue-500/40 outline-none"
                                    value={forge_new_model.rpm}
                                    onChange={e => set_forge_new_model({ ...forge_new_model, rpm: parseInt(e.target.value) || 0 })}
                                    aria-label={i18n.t('provider.field_rpm_label')}
                                />
                            </div>
                            <div className="space-y-1">
                                <label htmlFor="forge-tpm" className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1 flex items-center gap-1.5">
                                    {i18n.t('provider.forge_item.label_tpm')}
                                    <Tooltip content={i18n.t('provider.tpm_tooltip')} position="top">
                                        <Info size={9} className="text-zinc-700 hover:text-blue-400 cursor-help transition-colors" />
                                    </Tooltip>
                                </label>
                                <input
                                    type="number"
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-blue-500/40 outline-none"
                                    value={forge_new_model.tpm}
                                    onChange={e => set_forge_new_model({ ...forge_new_model, tpm: parseInt(e.target.value) || 0 })}
                                    aria-label={i18n.t('provider.field_tpm_label')}
                                />
                            </div>
                            <div className="space-y-1">
                                <label className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('provider.field_rpd')}</label>
                                <input
                                    type="number"
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-blue-500/40 outline-none"
                                    value={forge_new_model.rpd}
                                    onChange={e => set_forge_new_model({ ...forge_new_model, rpd: parseInt(e.target.value) || 0 })}
                                    aria-label={i18n.t('provider.field_rpd_label')}
                                />
                            </div>
                            <div className="space-y-1">
                                <label className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('provider.field_tpd')}</label>
                                <input
                                    type="number"
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-blue-500/40 outline-none"
                                    value={forge_new_model.tpd}
                                    onChange={e => set_forge_new_model({ ...forge_new_model, tpd: parseInt(e.target.value) || 0 })}
                                    aria-label={i18n.t('provider.field_tpd_label')}
                                />
                            </div>
                        </div>
                        <div className="flex justify-end gap-2 pt-2 border-t border-zinc-800/50">
                            <Tooltip content={i18n.t('provider.commit_node_tooltip')} position="bottom">
                                <button onClick={handle_forge_add} className="p-1.5 bg-blue-500/20 text-blue-400 rounded hover:bg-blue-500/30 transition-colors">
                                    <Check size={16} />
                                </button>
                            </Tooltip>
                            <Tooltip content={i18n.t('provider.cancel_node_tooltip')} position="bottom">
                                <button onClick={() => set_is_forge_adding(false)} className="p-1.5 bg-zinc-800 text-zinc-500 rounded hover:bg-zinc-700 transition-colors">
                                    <X size={16} />
                                </button>
                            </Tooltip>
                        </div>
                    </div>
                )}

                {provider_models.length === 0 ? (
                    <div className="p-8 text-center">
                        <div className="inline-flex p-3 rounded-full bg-zinc-900 border border-zinc-800 text-zinc-600 mb-3">
                            <Cpu size={24} />
                        </div>
                        <p className="text-xs text-zinc-500 font-mono uppercase tracking-widest">{i18n.t('provider.no_nodes')}</p>
                    </div>
                ) : (
                    provider_models.map((m) => (
                        <Forge_Item
                            key={m.id}
                            model={m}
                            is_editing={editing_model_id === m.id}
                            on_edit={() => set_editing_model_id(m.id)}
                            on_cancel={() => set_editing_model_id(null)}
                            on_save={(id, name, prov, modality, limits) => {
                                edit_model(id, name, prov, modality, limits);
                                set_editing_model_id(null);
                            }}
                            on_delete={() => delete_model(m.id)}
                        />
                    ))
                )}
            </div>
        </section>
    );
}
