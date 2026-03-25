import { Trash2, Sliders } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { ProviderConfig } from '../../stores/providerStore';

interface ProviderCardProps {
    provider: ProviderConfig;
    isSelected: boolean;
    onSelect: (id: string) => void;
    onDelete: (id: string, name: string) => void;
    modelsCount: number;
}

export function ProviderCard({
    provider,
    isSelected,
    onSelect,
    onDelete,
    modelsCount
}: ProviderCardProps) {
    return (
        <div
            role="button"
            tabIndex={0}
            onClick={() => onSelect(provider.id)}
            onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && onSelect(provider.id)}
            className={`group p-5 bg-zinc-900/40 border rounded-2xl transition-all duration-300 relative overflow-hidden flex flex-col items-start gap-3 hover:border-emerald-500/40 hover:bg-emerald-500/[0.02] cursor-pointer ${isSelected ? 'border-emerald-500 bg-emerald-500/5' : 'border-zinc-800'}`}
            aria-label={i18n.t('model_manager.aria_manage_provider', { name: provider.name })}
            aria-pressed={isSelected}
        >
            <div className="flex items-center justify-between w-full relative z-10">
                <div className="text-2xl italic">{provider.icon}</div>
                <div className="flex items-center gap-2">
                    <button
                        onClick={(e) => {
                            e.stopPropagation();
                            e.preventDefault();
                            onDelete(provider.id, provider.name);
                        }}
                        aria-label={i18n.t('model_manager.aria_terminate_provider', { name: provider.name })}
                        className="p-2 -m-0.5 hover:bg-red-500/20 text-zinc-400 hover:text-red-400 rounded-lg transition-all active:scale-90 flex items-center justify-center relative z-[100] border border-transparent hover:border-red-500/30"
                    >
                        <Trash2 size={18} />
                    </button>
                    <Tooltip content={i18n.t('model_manager.grid.tooltip_configure')} position="top">
                        <div className="p-1.5 text-zinc-700 group-hover:text-emerald-400 transition-colors">
                            <Sliders size={14} />
                        </div>
                    </Tooltip>
                </div>
            </div>
            
            <Tooltip content={i18n.t('model_manager.grid.tooltip_manage', { name: provider.name })} position="bottom" className="w-full">
                <div className="space-y-0.5 text-left relative z-10">
                    <h3 className="font-bold text-zinc-100 text-sm tracking-tight">{provider.name}</h3>
                    <p className="text-[11px] font-mono text-zinc-600 uppercase group-hover:text-zinc-400 transition-colors">
                        {i18n.t('model_manager.grid.protocol_nodes', { protocol: provider.protocol || 'API', count: modelsCount })}
                    </p>
                </div>
            </Tooltip>

            {isSelected && (
                <div className="absolute top-0 right-0 w-12 h-12 bg-emerald-500/10 blur-xl rounded-full pointer-events-none" />
            )}
        </div>
    );
}
