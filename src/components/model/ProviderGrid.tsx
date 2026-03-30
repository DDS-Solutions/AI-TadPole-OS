import { Database, Plus } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { ProviderConfig, ModelEntry } from '../../stores/providerStore';
import { ProviderCard } from './ProviderCard';

interface ProviderGridProps {
    providers: ProviderConfig[];
    models: ModelEntry[];
    selectedProviderId: string | null;
    onSelectProvider: (id: string | null) => void;
    onDeleteProvider: (id: string, name: string) => void;
    onAddProvider: () => void;
    isAddingProvider: boolean;
    children?: React.ReactNode;
}

export function ProviderGrid({
    providers,
    models,
    selectedProviderId,
    onSelectProvider,
    onDeleteProvider,
    onAddProvider,
    isAddingProvider,
    children
}: ProviderGridProps) {
    return (
        <section className="space-y-6">
            <div className="flex items-center justify-between">
                <h2 className="text-[11px] font-bold text-zinc-500 uppercase tracking-[0.3em] flex items-center gap-2">
                    <Database size={12} className="text-emerald-500" />
                    {i18n.t('model_manager.grid.title')}
                </h2>
                <Tooltip content={i18n.t('model_manager.grid.tooltip_add')} position="left">
                    <button
                        onClick={onAddProvider}
                        disabled={providers.length >= 25 || isAddingProvider}
                        className="flex items-center gap-1.5 text-[10px] font-bold text-emerald-500 hover:bg-emerald-500/5 px-2 py-1 rounded-md border border-emerald-500/10 transition-colors uppercase tracking-widest disabled:opacity-30"
                    >
                        <Plus size={12} /> {i18n.t('model_manager.grid.btn_add')}
                    </button>
                </Tooltip>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                {providers.map(p => (
                    <ProviderCard
                        key={p.id}
                        provider={p}
                        isSelected={selectedProviderId === p.id}
                        onSelect={(id) => onSelectProvider(selectedProviderId === id ? null : id)}
                        onDelete={onDeleteProvider}
                        modelsCount={models.filter(m => m.provider === p.id).length}
                    />
                ))}
                {children}
            </div>
        </section>
    );
}
