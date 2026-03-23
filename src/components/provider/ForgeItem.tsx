import { useState } from 'react';
import { Layers, Activity, Info, Zap, Edit2, Trash2, Check, X } from 'lucide-react';
import { i18n } from '../../i18n';
import { type ModelEntry } from '../../stores/providerStore';
import { Tooltip } from '../ui';

export function ForgeItem({ model, isEditing, onEdit, onCancel, onSave, onDelete }: {
    model: ModelEntry;
    isEditing: boolean;
    onEdit: () => void;
    onCancel: () => void;
    onSave: (id: string, name: string, prov: string, modality: ModelEntry['modality'], limits: Omit<ModelEntry, 'id' | 'name' | 'provider' | 'modality'>) => void;
    onDelete: () => void;
}): React.ReactElement {
    const [editName, setEditName] = useState(model.name);
    const [editModality, setEditModality] = useState<ModelEntry['modality']>(model.modality || 'llm');
    const [isCustomModality, setIsCustomModality] = useState(!['llm', 'vision', 'voice', 'reasoning'].includes(model.modality || 'llm'));
    const [customModality, setCustomModality] = useState(model.modality || '');
    const [limits, setLimits] = useState({
        rpm: model.rpm || 10,
        tpm: model.tpm || 100000,
        rpd: model.rpd || 1000,
        tpd: model.tpd || 10000000
    });

    const [prevIsEditing, setPrevIsEditing] = useState(isEditing);
    if (isEditing !== prevIsEditing) {
        setPrevIsEditing(isEditing);
        if (isEditing) {
            setEditName(model.name);
            setEditModality(model.modality || 'llm');
            const custom = !['llm', 'vision', 'voice', 'reasoning'].includes(model.modality || 'llm');
            setIsCustomModality(custom);
            setCustomModality(model.modality || '');
            setLimits({
                rpm: model.rpm || 10,
                tpm: model.tpm || 100000,
                rpd: model.rpd || 1000,
                tpd: model.tpd || 10000000
            });
        }
    }

    if (isEditing) {
        return (
            <div className="p-4 bg-emerald-500/[0.03] space-y-4 animate-in fade-in duration-300">
                <div className="flex gap-3">
                    <input
                        className="bg-zinc-950 border border-emerald-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 flex-1 focus:outline-none focus:border-emerald-500 font-mono"
                        value={editName}
                        onChange={e => setEditName(e.target.value)}
                        aria-label={i18n.t('provider.forge_name_label')}
                    />
                    <select
                        className="bg-zinc-950 border border-emerald-500/30 rounded-lg px-3 py-1.5 text-[11px] text-zinc-400 focus:outline-none uppercase font-bold"
                        value={isCustomModality ? 'other' : editModality}
                        onChange={e => {
                            if (e.target.value === 'other') {
                                setIsCustomModality(true);
                            } else {
                                setIsCustomModality(false);
                                setEditModality(e.target.value as ModelEntry['modality']);
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
                {isCustomModality && (
                    <input
                        className="bg-zinc-950 border border-emerald-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 w-full focus:outline-none focus:border-emerald-500 font-mono"
                        placeholder={i18n.t('provider.placeholder_custom_modality')}
                        value={customModality}
                        onChange={e => setCustomModality(e.target.value)}
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
                            className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-emerald-500/40 outline-none"
                            value={limits.rpm}
                            onChange={e => setLimits({ ...limits, rpm: parseInt(e.target.value) || 0 })}
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
                            className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-emerald-500/40 outline-none"
                            value={limits.tpm}
                            onChange={e => setLimits({ ...limits, tpm: parseInt(e.target.value) || 0 })}
                            aria-label={i18n.t('provider.field_tpm_label')}
                        />
                    </div>
                    <div className="space-y-1">
                        <label className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('provider.forge_item.label_rpd')}</label>
                        <input
                            type="number"
                            className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-emerald-500/40 outline-none"
                            value={limits.rpd}
                            onChange={e => setLimits({ ...limits, rpd: parseInt(e.target.value) || 0 })}
                            aria-label={i18n.t('provider.field_rpd_label')}
                        />
                    </div>
                    <div className="space-y-1">
                        <label className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('provider.forge_item.label_tpd')}</label>
                        <input
                            type="number"
                            className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-emerald-500/40 outline-none"
                            value={limits.tpd}
                            onChange={e => setLimits({ ...limits, tpd: parseInt(e.target.value) || 0 })}
                            aria-label={i18n.t('provider.field_tpd_label')}
                        />
                    </div>
                </div>

                <div className="flex justify-end gap-2 pt-2 border-t border-zinc-800/50">
                    <Tooltip content={i18n.t('provider.forge_item.tooltip_save')} position="bottom">
                        <button
                            onClick={() => {
                                const finalModality = isCustomModality ? customModality : editModality;
                                onSave(model.id, editName, model.provider, finalModality, limits);
                            }}
                            className="p-1.5 bg-emerald-500/20 text-emerald-400 rounded hover:bg-emerald-500/30 transition-colors"
                            aria-label={i18n.t('provider.aria_save_node')}
                        >
                            <Check size={14} />
                        </button>
                    </Tooltip>
                    <Tooltip content={i18n.t('provider.forge_item.tooltip_cancel')} position="bottom">
                        <button 
                            onClick={onCancel} 
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
                    model.modality === 'voice' ? 'bg-blue-500/10 border-blue-500/20 text-blue-500' :
                        model.modality === 'reasoning' ? 'bg-blue-500/10 border-blue-500/20 text-blue-500' :
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
                            {(model.tpm || 100000).toLocaleString()} <span className="text-zinc-600 opacity-80">TPM</span>
                        </span>
                    </div>
                </div>
            </div>
            <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-all">
                <Tooltip content={i18n.t('provider.forge_item.tooltip_edit')} position="left">
                    <button
                        onClick={onEdit}
                        className="p-1.5 rounded hover:bg-emerald-500/10 text-zinc-700 hover:text-emerald-500"
                    >
                        <Edit2 size={12} />
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('provider.forge_item.tooltip_delete')} position="left">
                    <button
                        onClick={onDelete}
                        className="p-1.5 rounded hover:bg-red-500/10 text-zinc-700 hover:text-red-500"
                    >
                        <Trash2 size={12} />
                    </button>
                </Tooltip>
            </div>
        </div>
    );
}
