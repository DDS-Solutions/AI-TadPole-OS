import { useState } from 'react';
import { Edit2, Trash2, Check, X } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { ModelEntry, ProviderConfig } from '../../stores/providerStore';

interface ModelRowProps {
    model: ModelEntry;
    isEditing: boolean;
    onEdit: () => void;
    onSave: (name: string, prov: string, modality: ModelEntry['modality'], limits: Record<string, number>) => void;
    onCancel: () => void;
    onDelete: () => void;
    providers: ProviderConfig[];
}

export function ModelRow({ 
    model, 
    isEditing, 
    onEdit, 
    onSave, 
    onCancel, 
    onDelete, 
    providers 
}: ModelRowProps) {
    const [editName, setEditName] = useState(model.name);
    const [editProv, setEditProv] = useState(model.provider);
    const [editModality, setEditModality] = useState<ModelEntry['modality']>(model.modality || 'llm');
    const [isCustomModality, setIsCustomModality] = useState(!['llm', 'vision', 'voice', 'reasoning'].includes(model.modality || 'llm'));
    const [customModality, setCustomModality] = useState(model.modality || '');
    const [limits, setLimits] = useState({
        rpm: model.rpm || 10,
        tpm: model.tpm || 100000,
        rpd: model.rpd || 1000,
        tpd: model.tpd || 10000000
    });
    const [showLimits, setShowLimits] = useState(false);

    if (isEditing) {
        return (
            <>
                <tr className="bg-blue-500/[0.03]">
                    <td className="px-8 py-4">
                        <input
                            className="bg-zinc-950 border border-blue-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 w-full focus:outline-none focus:border-blue-500 font-mono"
                            value={editName}
                            onChange={e => setEditName(e.target.value)}
                        />
                    </td>
                    <td className="px-8 py-4">
                        <select
                            className="bg-zinc-950 border border-blue-500/30 rounded-lg px-3 py-1.5 text-[10px] text-zinc-400 w-full focus:outline-none cursor-pointer uppercase font-bold"
                            value={isCustomModality ? 'other' : editModality}
                            onChange={e => {
                                if (e.target.value === 'other') {
                                    setIsCustomModality(true);
                                } else {
                                    setIsCustomModality(false);
                                    setEditModality(e.target.value as ModelEntry['modality']);
                                }
                            }}
                        >
                            <option value="llm">{i18n.t('provider.label_modality_llm')}</option>
                            <option value="vision">{i18n.t('provider.label_modality_vision')}</option>
                            <option value="voice">{i18n.t('provider.label_modality_voice')}</option>
                            <option value="reasoning">{i18n.t('provider.label_modality_reasoning')}</option>
                            <option value="other">{i18n.t('provider.label_modality_other')}</option>
                        </select>
                        {isCustomModality && (
                            <input
                                className="mt-2 bg-zinc-950 border border-blue-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 w-full focus:outline-none focus:border-blue-500 font-mono"
                                placeholder={i18n.t('provider.label_modality_other')}
                                value={customModality}
                                onChange={e => setCustomModality(e.target.value)}
                            />
                        )}
                    </td>
                    <td className="px-8 py-4">
                        <select
                            className="bg-zinc-950 border border-blue-500/30 rounded-lg px-3 py-1.5 text-[10px] text-zinc-400 w-full focus:outline-none cursor-pointer uppercase font-bold"
                            value={editProv}
                            onChange={e => setEditProv(e.target.value)}
                        >
                            {providers.map(p => <option key={p.id} value={p.id}>{p.name.toUpperCase()}</option>)}
                        </select>
                    </td>
                    <td className="px-8 py-4 text-right">
                        <div className="flex items-center justify-end gap-3">
                            <button
                                onClick={() => {
                                    const finalModality = isCustomModality ? customModality : editModality;
                                    onSave(editName, editProv, finalModality, limits);
                                }}
                                aria-label={i18n.t('provider.forge_item.tooltip_save')}
                                className="p-1.5 bg-blue-500/20 text-blue-400 rounded hover:bg-blue-500/30 transition-all"
                            >
                                <Check size={14} />
                            </button>
                            <button onClick={onCancel} aria-label={i18n.t('provider.forge_item.tooltip_cancel')} className="p-1.5 bg-zinc-800 text-zinc-500 rounded hover:bg-zinc-700 transition-all"><X size={14} /></button>
                        </div>
                    </td>
                </tr>
                <tr className="bg-blue-500/[0.02]">
                    <td colSpan={4} className="px-12 py-5 border-b border-blue-500/10">
                        <div className="grid grid-cols-4 gap-8">
                            <div className="space-y-1.5">
                                <label className="text-[9px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('model_manager.row.req_min')}</label>
                                <input
                                    type="number"
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-xs text-zinc-400 font-mono focus:outline-none focus:border-blue-500/50"
                                    value={limits.rpm}
                                    onChange={e => setLimits({ ...limits, rpm: parseInt(e.target.value) || 0 })}
                                />
                            </div>
                            <div className="space-y-1.5">
                                <label className="text-[9px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('model_manager.row.tkn_min')}</label>
                                <input
                                    type="number"
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-xs text-zinc-400 font-mono focus:outline-none focus:border-blue-500/50"
                                    value={limits.tpm}
                                    onChange={e => setLimits({ ...limits, tpm: parseInt(e.target.value) || 0 })}
                                />
                            </div>
                            <div className="space-y-1.5">
                                <label className="text-[9px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('model_manager.row.req_day')}</label>
                                <input
                                    type="number"
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-xs text-zinc-400 font-mono focus:outline-none focus:border-blue-500/50"
                                    value={limits.rpd}
                                    onChange={e => setLimits({ ...limits, rpd: parseInt(e.target.value) || 0 })}
                                />
                            </div>
                            <div className="space-y-1.5">
                                <label className="text-[9px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('model_manager.row.tkn_day')}</label>
                                <input
                                    type="number"
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-xs text-zinc-400 font-mono focus:outline-none focus:border-blue-500/50"
                                    value={limits.tpd}
                                    onChange={e => setLimits({ ...limits, tpd: parseInt(e.target.value) || 0 })}
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
                        <div className="w-1.5 h-1.5 rounded-full bg-blue-500/40 group-hover:bg-blue-400 transition-colors shadow-[0_0_5px_rgba(59,130,246,0.2)]" />
                        <div className="flex flex-col">
                            <span className="text-xs font-bold text-zinc-200 uppercase tracking-tight">{model.name}</span>
                            <button
                                onClick={() => setShowLimits(!showLimits)}
                                className="text-[11px] text-zinc-600 hover:text-blue-400 font-bold mt-1 transition-colors flex items-center gap-1 uppercase tracking-widest"
                                aria-expanded={showLimits}
                                aria-controls={`limits-${model.id}`}
                            >
                                {showLimits ? i18n.t('model_manager.row.hide_limits') : i18n.t('model_manager.row.show_limits')}
                            </button>
                        </div>
                    </div>
                </td>
                <td className="px-8 py-5">
                    <span className={`text-[9px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-full border ${model.modality === 'vision' ? 'bg-amber-500/10 border-amber-500/20 text-amber-500' :
                        model.modality === 'voice' ? 'bg-blue-500/10 border-blue-500/20 text-blue-500' :
                            model.modality === 'reasoning' ? 'bg-blue-500/10 border-blue-500/20 text-blue-500' :
                                'bg-zinc-800/50 border-white/5 text-zinc-500'
                        }`}>
                        {model.modality || 'llm'}
                    </span>
                </td>
                <td className="px-8 py-5">
                    <span className="text-[11px] font-bold font-mono text-zinc-500 uppercase tracking-tighter bg-zinc-900 border border-white/5 px-2 py-0.5 rounded">
                        {model.provider}
                    </span>
                </td>
                <td className="px-8 py-5 text-right">
                    <div className="flex items-center justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all transform translate-x-2 group-hover:translate-x-0">
                        <Tooltip content={i18n.t('model_manager.row.tooltip_edit')} position="top">
                            <button onClick={onEdit} aria-label={i18n.t('model_manager.row.tooltip_edit')} className="p-1.5 rounded-md hover:bg-zinc-800 text-zinc-600 hover:text-zinc-100 transition-colors">
                                <Edit2 size={13} />
                            </button>
                        </Tooltip>
                        <Tooltip content={i18n.t('model_manager.row.tooltip_delete')} position="top">
                            <button
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onDelete();
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
            {showLimits && (
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
