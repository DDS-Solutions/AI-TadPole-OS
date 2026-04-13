/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Identity card for LLM Providers. 
 * Renders service status indicators, node counts, and facilitates transition to the granular `Provider_Config_Panel`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Status pulse desync if backend health-check is delayed, or card detachment race condition.
 * - **Telemetry Link**: Search for `[Provider_Card]` or `provider_pulse` in UI tracing.
 */

import { Trash2, Sliders } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Provider_Config } from '../../stores/provider_store';

interface Provider_Card_Props {
    provider: Provider_Config;
    is_selected: boolean;
    on_select: (id: string) => void;
    on_delete: (id: string, name: string) => void;
    models_count: number;
}

export function Provider_Card({
    provider,
    is_selected,
    on_select,
    on_delete,
    models_count
}: Provider_Card_Props) {
    return (
        <div
            role="button"
            tabIndex={0}
            onClick={() => on_select(provider.id)}
            onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && on_select(provider.id)}
            className={`group p-5 bg-zinc-900/40 border rounded-2xl transition-all duration-300 relative overflow-hidden flex flex-col items-start gap-3 hover:border-emerald-500/40 hover:bg-emerald-500/[0.02] cursor-pointer ${is_selected ? 'border-emerald-500 bg-emerald-500/5' : 'border-zinc-800'}`}
            aria-label={i18n.t('model_manager.aria_manage_provider', { name: provider.name })}
            aria-pressed={is_selected}
        >
            <div className="flex items-center justify-between w-full relative z-10">
                <div className="text-2xl italic">{provider.icon}</div>
                <div className="flex items-center gap-2">
                    <button
                        onClick={(e) => {
                            e.stopPropagation();
                            e.preventDefault();
                            on_delete(provider.id, provider.name);
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
            
            <Tooltip content={i18n.t('model_manager.grid.tooltip_manage', { name: provider.name })} position="bottom" class_name="w-full">
                <div className="space-y-0.5 text-left relative z-10">
                    <h3 className="font-bold text-zinc-100 text-sm tracking-tight">{provider.name}</h3>
                    <p className="text-[11px] font-mono text-zinc-600 uppercase group-hover:text-zinc-400 transition-colors">
                        {i18n.t('model_manager.grid.protocol_nodes', { protocol: provider.protocol || 'API', count: models_count })}
                    </p>
                </div>
            </Tooltip>

            {is_selected && (
                <div className="absolute top-0 right-0 w-12 h-12 bg-emerald-500/10 blur-xl rounded-full pointer-events-none" />
            )}
        </div>
    );
}

