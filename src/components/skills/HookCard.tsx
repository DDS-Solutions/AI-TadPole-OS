import React from 'react';
import { Edit2, Trash2, Shield } from 'lucide-react';
import type { HookDefinition } from '../../stores/skillStore';

interface HookCardProps {
    hook: HookDefinition;
    onEdit: (hook: HookDefinition) => void;
    onDelete: (name: string) => void;
}

export const HookCard: React.FC<HookCardProps> = ({ hook, onEdit, onDelete }) => {
    return (
        <div key={hook.name} className={`bg-zinc-950 border border-zinc-800 p-5 rounded-xl transition-all duration-300 hover:border-emerald-500/30 hover:shadow-[0_0_15px_rgba(16,185,129,0.15)] group relative overflow-hidden shadow-sm shadow-black/40 ${hook.active ? '' : 'opacity-60 grayscale'}`}>
            <div className="neural-grid opacity-[0.03]" />
            <div className="absolute top-4 right-4 flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity z-20">
                <button 
                    onClick={() => onEdit(hook)}
                    className="text-zinc-500 hover:text-blue-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded transition-colors"
                >
                    <Edit2 className="w-3.5 h-3.5" />
                </button>
                <button 
                    onClick={() => onDelete(hook.name)}
                    className="text-zinc-500 hover:text-red-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded transition-colors"
                >
                    <Trash2 className="w-3.5 h-3.5" />
                </button>
            </div>
            <div className="relative z-10 flex flex-col h-full">
                <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-2 text-zinc-300 font-bold tracking-wide">
                        <div className={`w-2 h-2 rounded-full ${hook.active ? 'bg-emerald-500 animate-pulse' : 'bg-zinc-600'} shrink-0`}></div>
                        <h3 className="font-mono text-sm">{hook.name}</h3>
                    </div>
                    <span className="text-[9px] font-bold text-zinc-500 uppercase tracking-widest">{hook.hook_type.replace('_', ' ')}</span>
                </div>
                <p className="text-zinc-500 text-xs font-mono mb-4 leading-relaxed line-clamp-2 h-8">{hook.description}</p>
                <div className="mt-auto pt-4 border-t border-zinc-900 flex items-center justify-between">
                    <span className={`text-[9px] px-2 py-0.5 rounded border font-bold uppercase ${hook.active ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20' : 'bg-zinc-800 text-zinc-500 border-zinc-700'}`}>
                        {hook.active ? 'Active' : 'Bypassed'}
                    </span>
                    <Shield size={12} className="text-zinc-700" />
                </div>
            </div>
        </div>
    );
};
