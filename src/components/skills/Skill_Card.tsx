import React from 'react';
import { Edit2, Users, Trash2, Terminal } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Skill_Definition } from '../../stores/skill_store';

interface Skill_Card_Props {
    skill: Skill_Definition;
    on_edit: (skill: Skill_Definition) => void;
    on_assign: (name: string) => void;
    on_delete: (name: string) => void;
}

export const Skill_Card: React.FC<Skill_Card_Props> = ({ skill, on_edit, on_assign, on_delete }) => {
    return (
        <div className="bg-zinc-950 border border-zinc-800 p-5 rounded-xl transition-all duration-300 hover:border-emerald-500/30 hover:shadow-[0_0_15px_rgba(16,185,129,0.15)] group relative overflow-hidden shadow-sm">
            <div className="neural-grid opacity-[0.03]" />
            <div className="absolute top-4 right-4 flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity z-20">
                <Tooltip content={i18n.t('skills.tooltip_edit_skill')} position="top">
                    <button onClick={() => on_edit(skill)} className="text-zinc-500 hover:text-blue-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded">
                        <Edit2 className="w-3.5 h-3.5" />
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('agent_manager.tooltip_assign')} position="top">
                    <button onClick={() => on_assign(skill.name)} className="text-zinc-500 hover:text-emerald-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded transition-colors">
                        <Users className="w-3.5 h-3.5" />
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('skills.tooltip_delete_skill')} position="top">
                    <button onClick={() => on_delete(skill.name)} className="text-zinc-500 hover:text-red-400 bg-zinc-900 hover:bg-zinc-800 p-1.5 rounded">
                        <Trash2 className="w-3.5 h-3.5" />
                    </button>
                </Tooltip>
            </div>
            <div className="relative z-10">
                <div className="flex items-center gap-3 mb-2 pr-16 text-zinc-300 font-bold tracking-wide">
                    <div className="w-2 h-2 rounded-full bg-emerald-500/30 group-hover:bg-emerald-400 group-hover:shadow-[0_0_8px_rgba(16,185,129,0.5)] transition-all shrink-0 mt-0.5"></div>
                    <h3 className="font-mono text-sm">{skill.name}</h3>
                </div>
                <p className="text-zinc-500 text-xs line-clamp-2 mb-4 h-8 leading-relaxed font-mono">{skill.description}</p>
                <div className="bg-black/40 border border-zinc-800/50 p-2.5 rounded font-mono text-[10px] text-zinc-300 flex items-center gap-2 overflow-x-auto">
                    <Terminal className="w-3 h-3 flex-shrink-0 text-zinc-500" />
                    <span className="whitespace-nowrap">{skill.execution_command}</span>
                </div>
            </div>
        </div>
    );
};
