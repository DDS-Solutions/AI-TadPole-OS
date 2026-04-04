import { Cpu, Plus, Filter } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Model_Entry, Provider_Config } from '../../stores/provider_store';
import { Model_Row } from './Model_Row';

interface Model_Inventory_Table_Props {
    models: Model_Entry[];
    modality_filter: 'all' | Model_Entry['modality'];
    on_set_modality_filter: (filter: 'all' | Model_Entry['modality']) => void;
    on_add_node: () => void;
    editing_id: string | null;
    on_edit_node: (id: string | null) => void;
    on_save_node: (id: string, name: string, prov: string, modality: Model_Entry['modality'], limits: Record<string, number>) => void;
    on_delete_node: (id: string, name: string) => void;
    providers: Provider_Config[];
    children?: React.ReactNode;
}

export function Model_Inventory_Table({
    models,
    modality_filter,
    on_set_modality_filter,
    on_add_node,
    editing_id,
    on_edit_node,
    on_save_node,
    on_delete_node,
    providers,
    children
}: Model_Inventory_Table_Props) {
    return (
        <section className="space-y-6">
            <div className="flex items-center justify-between">
                <h2 className="text-[11px] font-bold text-zinc-500 uppercase tracking-[0.3em] flex items-center gap-2">
                    <Cpu size={12} className="text-blue-500" />
                    {i18n.t('model_manager.inventory.title')}
                </h2>
                <div className="flex items-center gap-3">
                    <Tooltip content={i18n.t('model_manager.inventory.tooltip_provision')} position="top">
                        <button
                            onClick={on_add_node}
                            aria-label={i18n.t('model_manager.inventory.btn_add')}
                            className="flex items-center gap-1.5 text-[10px] font-bold text-blue-500 hover:bg-blue-500/5 px-2 py-1 rounded-md border border-blue-500/10 transition-colors uppercase tracking-widest"
                        >
                            <Plus size={12} /> {i18n.t('model_manager.inventory.btn_add')}
                        </button>
                    </Tooltip>
                    <Tooltip content={i18n.t('model_manager.inventory.tooltip_filter')} position="top">
                        <div className="flex items-center gap-1 bg-zinc-900 border border-zinc-800 rounded-lg px-2 py-1 cursor-help">
                            <Filter size={10} className="text-zinc-600" />
                            <select
                                value={modality_filter}
                                onChange={(e) => on_set_modality_filter(e.target.value as 'all' | Model_Entry['modality'])}
                                className="bg-transparent border-none text-[9px] font-bold text-zinc-400 uppercase focus:ring-0 cursor-pointer"
                                aria-label={i18n.t('model_manager.aria_filter_modality')}
                            >
                                <option value="all">{i18n.t('model_manager.inventory.filter_all')}</option>
                                <option value="llm">{i18n.t('provider.label_modality_llm')}</option>
                                <option value="vision">{i18n.t('provider.label_modality_vision')}</option>
                                <option value="voice">{i18n.t('provider.label_modality_voice')}</option>
                                <option value="reasoning">{i18n.t('provider.label_modality_reasoning')}</option>
                            </select>
                        </div>
                    </Tooltip>
                </div>
            </div>

            <div className="bg-zinc-900/40 backdrop-blur-xl border border-zinc-800 rounded-3xl overflow-hidden shadow-2xl relative">
                <table className="w-full text-left text-sm border-collapse">
                    <thead className="bg-zinc-950/80 backdrop-blur text-zinc-500 text-[9px] uppercase tracking-[0.2em] border-b border-zinc-800">
                        <tr>
                            <th className="px-8 py-5 font-bold text-[11px]">{i18n.t('model_manager.inventory.col_identity')}</th>
                            <th className="px-8 py-5 font-bold text-[11px]">{i18n.t('model_manager.inventory.col_modality')}</th>
                            <th className="px-8 py-5 font-bold text-[11px]">{i18n.t('model_manager.inventory.col_provider')}</th>
                            <th className="px-8 py-5 font-bold text-right text-[11px]">{i18n.t('model_manager.inventory.col_actions')}</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-zinc-800/30 font-mono text-[11px]">
                        {children}
                        {models.map((m) => (
                            <Model_Row
                                key={m.id + (editing_id === m.id ? '-editing' : '')}
                                model={m}
                                is_editing={editing_id === m.id}
                                on_edit={() => on_edit_node(m.id)}
                                on_save={(name, prov, modality, limits) => on_save_node(m.id, name, prov, modality, limits)}
                                on_cancel={() => on_edit_node(null)}
                                on_delete={() => on_delete_node(m.id, m.name)}
                                providers={providers}
                            />
                        ))}
                    </tbody>
                </table>
            </div>
        </section>
    );
}
