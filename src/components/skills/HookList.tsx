import React from 'react';
import { Shield, Plus } from 'lucide-react';
import { i18n } from '../../i18n';
import type { HookDefinition } from '../../stores/skillStore';
import { HookCard } from './HookCard';

interface HookListProps {
    hooks: HookDefinition[];
    searchQuery: string;
    activeCategory: 'user' | 'ai';
    onNewHook: () => void;
    onEditHook: (hook: HookDefinition) => void;
    onDeleteHook: (name: string) => void;
}

export const HookList: React.FC<HookListProps> = ({
    hooks,
    searchQuery,
    activeCategory,
    onNewHook,
    onEditHook,
    onDeleteHook
}) => {
    const filteredHooks = hooks.filter(h => 
        h.category === activeCategory && 
        (h.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
         h.description.toLowerCase().includes(searchQuery.toLowerCase()))
    );

    return (
        <div className="space-y-6">
            <div className="flex justify-between items-center mb-6 sticky top-0 bg-zinc-950/80 backdrop-blur-md pt-2 pb-3 border-b border-zinc-800/50 z-20">
                <h3 className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                    <Shield size={12} className="text-emerald-500" /> {i18n.t('skills.header_hooks')}
                </h3>
                <button
                    onClick={onNewHook}
                    className="bg-emerald-600 hover:bg-emerald-500 text-white px-3 py-1.5 rounded-lg text-xs font-bold flex items-center gap-2 transition-colors shadow-[0_0_15px_rgba(16,185,129,0.3)]"
                >
                    <Plus className="w-3.5 h-3.5" /> Register Hook
                </button>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 relative z-10">
                {filteredHooks.map(hook => (
                    <HookCard 
                        key={hook.name} 
                        hook={hook} 
                        onEdit={onEditHook} 
                        onDelete={onDeleteHook} 
                    />
                ))}
                {filteredHooks.length === 0 && (
                    <div className="col-span-full py-12 flex flex-col items-center justify-center border border-dashed border-zinc-800 rounded-2xl bg-zinc-950/20">
                        <Shield size={32} className="text-zinc-800 mb-4" />
                        <p className="text-zinc-500 text-sm font-mono">No lifecycle hooks registered in this sector.</p>
                    </div>
                )}
            </div>
        </div>
    );
};
