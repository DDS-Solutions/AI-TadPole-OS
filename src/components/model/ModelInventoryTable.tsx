import { Cpu, Plus, Filter } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { ModelEntry, ProviderConfig } from '../../stores/providerStore';
import { ModelRow } from './ModelRow';

interface ModelInventoryTableProps {
    models: ModelEntry[];
    modalityFilter: 'all' | ModelEntry['modality'];
    onSetModalityFilter: (filter: 'all' | ModelEntry['modality']) => void;
    onAddNode: () => void;
    editingId: string | null;
    onEditNode: (id: string | null) => void;
    onSaveNode: (id: string, name: string, prov: string, modality: ModelEntry['modality'], limits: Record<string, number>) => void;
    onDeleteNode: (id: string, name: string) => void;
    providers: ProviderConfig[];
    children?: React.ReactNode;
}

export function ModelInventoryTable({
    models,
    modalityFilter,
    onSetModalityFilter,
    onAddNode,
    editingId,
    onEditNode,
    onSaveNode,
    onDeleteNode,
    providers,
    children
}: ModelInventoryTableProps) {
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
                            onClick={onAddNode}
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
                                value={modalityFilter}
                                onChange={(e) => onSetModalityFilter(e.target.value as 'all' | ModelEntry['modality'])}
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
                            <ModelRow
                                key={m.id + (editingId === m.id ? '-editing' : '')}
                                model={m}
                                isEditing={editingId === m.id}
                                onEdit={() => onEditNode(m.id)}
                                onSave={(name, prov, modality, limits) => onSaveNode(m.id, name, prov, modality, limits)}
                                onCancel={() => onEditNode(null)}
                                onDelete={() => onDeleteNode(m.id, m.name)}
                                providers={providers}
                            />
                        ))}
                    </tbody>
                </table>
            </div>
        </section>
    );
}
