import { Database, Plus } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Provider_Config, Model_Entry } from '../../stores/provider_store';
import { Provider_Card } from './Provider_Card';

interface ProviderGridProps {
    providers: Provider_Config[];
    models: Model_Entry[];
    selected_provider_id: string | null;
    on_select_provider: (id: string | null) => void;
    on_delete_provider: (id: string, name: string) => void;
    on_add_provider: () => void;
    is_adding_provider: boolean;
    children?: React.ReactNode;
}

export function Provider_Grid({
    providers,
    models,
    selected_provider_id,
    on_select_provider,
    on_delete_provider,
    on_add_provider,
    is_adding_provider,
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
                        onClick={on_add_provider}
                        disabled={providers.length >= 25 || is_adding_provider}
                        className="flex items-center gap-1.5 text-[10px] font-bold text-emerald-500 hover:bg-emerald-500/5 px-2 py-1 rounded-md border border-emerald-500/10 transition-colors uppercase tracking-widest disabled:opacity-30"
                    >
                        <Plus size={12} /> {i18n.t('model_manager.grid.btn_add')}
                    </button>
                </Tooltip>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                {providers.map(p => (
                    <Provider_Card
                        key={p.id}
                        provider={p}
                        is_selected={selected_provider_id === p.id}
                        on_select={(id: string) => on_select_provider(selected_provider_id === id ? null : id)}
                        on_delete={on_delete_provider}
                        models_count={models.filter(m => m.provider === p.id).length}
                    />
                ))}
                {children}
            </div>
        </section>
    );
}
