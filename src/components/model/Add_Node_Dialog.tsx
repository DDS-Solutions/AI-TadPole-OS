/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Modal interface for injecting new model nodes into the provider inventory. 
 * Orchestrates form validation for RPM/TPM limits and modality categorization (LLM, Vision, etc.).
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Provider selection desync, numeric limit overflow, or modality categorization fallback to 'other' if unhandled.
 * - **Telemetry Link**: Search for `[Add_Node_Dialog]` or `confirm_add_node` in UI logs.
 */

import { Check, X } from 'lucide-react';
import { i18n } from '../../i18n';
import type { Model_Entry, Provider_Config } from '../../stores/provider_store';

interface Add_Node_Dialog_Props {
    new_node: {
        name: string;
        provider: string;
        modality: Model_Entry['modality'];
        rpm: number;
        tpm: number;
        rpd: number;
        tpd: number;
    };
    on_update_new_node: (updates: Partial<Add_Node_Dialog_Props['new_node']>) => void;
    on_confirm: () => void;
    on_cancel: () => void;
    is_custom_modality: boolean;
    on_set_is_custom_modality: (val: boolean) => void;
    custom_modality: string;
    on_set_custom_modality: (val: string) => void;
    providers: Provider_Config[];
}

export function Add_Node_Dialog({
    new_node,
    on_update_new_node,
    on_confirm,
    on_cancel,
    is_custom_modality,
    on_set_is_custom_modality,
    custom_modality,
    on_set_custom_modality,
    providers
}: Add_Node_Dialog_Props) {
    return (
        <>
            <tr className="bg-blue-500/5 animate-in slide-in-from-top-2 duration-300">
                <td className="px-8 py-4">
                    <label htmlFor="new-node-name" className="sr-only">{i18n.t('model_manager.inventory.col_identity')}</label>
                    <input
                        id="new-node-name"
                        className="bg-zinc-950 border border-blue-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 w-full focus:outline-none focus:border-blue-500 font-mono"
                        placeholder={i18n.t('model_manager.add_node.placeholder_name')}
                        value={new_node.name}
                        onChange={e => on_update_new_node({ name: e.target.value })}
                    />
                </td>
                <td className="px-8 py-4">
                    <select
                        className="bg-zinc-950 border border-blue-500/30 rounded-lg px-3 py-1.5 text-[10px] text-zinc-400 w-full focus:outline-none cursor-pointer uppercase font-bold"
                        value={is_custom_modality ? 'other' : new_node.modality}
                        onChange={e => {
                            if (e.target.value === 'other') {
                                on_set_is_custom_modality(true);
                            } else {
                                on_set_is_custom_modality(false);
                                on_update_new_node({ modality: e.target.value as Model_Entry['modality'] });
                            }
                        }}
                    >
                        <option value="llm">{i18n.t('provider.label_modality_llm')}</option>
                        <option value="vision">{i18n.t('provider.label_modality_vision')}</option>
                        <option value="voice">{i18n.t('provider.label_modality_voice')}</option>
                        <option value="reasoning">{i18n.t('provider.label_modality_reasoning')}</option>
                        <option value="other">{i18n.t('provider.label_modality_other')}</option>
                    </select>
                    {is_custom_modality && (
                        <input
                            className="mt-2 bg-zinc-950 border border-blue-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 w-full focus:outline-none focus:border-blue-500 font-mono"
                            placeholder={i18n.t('model_manager.add_node.placeholder_custom_modality')}
                            value={custom_modality}
                            onChange={e => on_set_custom_modality(e.target.value)}
                        />
                    )}
                </td>
                <td className="px-8 py-4">
                    <select
                        className="bg-zinc-950 border border-blue-500/30 rounded-lg px-3 py-1.5 text-[10px] text-zinc-400 w-full focus:outline-none cursor-pointer uppercase font-bold"
                        value={new_node.provider}
                        onChange={e => on_update_new_node({ provider: e.target.value })}
                    >
                        {providers.map(p => <option key={p.id} value={p.id}>{p.name.toUpperCase()}</option>)}
                    </select>
                </td>
                <td className="px-8 py-4 text-right">
                    <div className="flex items-center justify-end gap-3">
                        <button
                            onClick={on_confirm}
                            aria-label="Confirm Add Node"
                            className="p-1.5 bg-blue-500 text-white rounded hover:bg-blue-400 transition-all shadow-lg shadow-blue-500/20"
                        >
                            <Check size={14} />
                        </button>
                        <button onClick={on_cancel} className="p-1.5 bg-zinc-800 text-zinc-500 rounded hover:bg-zinc-700 transition-all"><X size={14} /></button>
                    </div>
                </td>
            </tr>
            <tr className="bg-blue-500/[0.02] border-b border-blue-500/10">
                <td colSpan={4} className="px-12 py-5">
                    <div className="grid grid-cols-4 gap-8">
                        <div className="space-y-1.5">
                            <label className="text-[9px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('model_manager.add_node.label_rpm')}</label>
                            <input
                                type="number"
                                className="w-full bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-xs text-zinc-400 font-mono focus:outline-none focus:border-blue-500/50"
                                value={new_node.rpm}
                                onChange={e => on_update_new_node({ rpm: parseInt(e.target.value) || 0 })}
                            />
                        </div>
                        <div className="space-y-1.5">
                            <label className="text-[9px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('model_manager.add_node.label_tpm')}</label>
                            <input
                                type="number"
                                className="w-full bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-xs text-zinc-400 font-mono focus:outline-none focus:border-blue-500/50"
                                value={new_node.tpm}
                                onChange={e => on_update_new_node({ tpm: parseInt(e.target.value) || 0 })}
                            />
                        </div>
                        <div className="space-y-1.5">
                            <label className="text-[9px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('model_manager.add_node.label_rpd')}</label>
                            <input
                                type="number"
                                className="w-full bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-xs text-zinc-400 font-mono focus:outline-none focus:border-blue-500/50"
                                value={new_node.rpd}
                                onChange={e => on_update_new_node({ rpd: parseInt(e.target.value) || 0 })}
                            />
                        </div>
                        <div className="space-y-1.5">
                            <label className="text-[9px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('model_manager.add_node.label_tpd')}</label>
                            <input
                                type="number"
                                className="w-full bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-xs text-zinc-400 font-mono focus:outline-none focus:border-blue-500/50"
                                value={new_node.tpd}
                                onChange={e => on_update_new_node({ tpd: parseInt(e.target.value) || 0 })}
                            />
                        </div>
                    </div>
                </td>
            </tr>
        </>
    );
}

