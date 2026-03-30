import React from 'react';
import { Zap, Terminal, FileText } from 'lucide-react';
import { Tooltip } from '../ui';
import { useDropdownStore } from '../../stores/dropdownStore';
import { getModelColor } from '../../utils/modelUtils';
import { i18n } from '../../i18n';
import type { Agent } from '../../types';

interface NodeStatsProps {
    agent: Agent;
    onSkillTrigger?: (agentId: string, skill: string, slot?: 1 | 2 | 3) => void;
}

export const NodeStats: React.FC<NodeStatsProps> = ({ agent, onSkillTrigger }) => {
    const toggle = useDropdownStore(s => s.toggle);
    const isSkillDropdownOpen = useDropdownStore(s => s.openId === agent.id && s.openType === 'skill');

    const modelSlots = [
        { label: i18n.t('agent_card.label_primary'), model: agent.model, config: agent.modelConfig },
        { label: i18n.t('agent_card.label_secondary'), model: agent.model2, config: agent.modelConfig2 },
        { label: i18n.t('agent_card.label_tertiary'), model: agent.model3, config: agent.modelConfig3 },
    ].filter(s => s.model);

    return (
        <div className={`grid grid-cols-[1fr_min-content] gap-2 items-center px-1 py-1 border-b border-zinc-800/30 relative ${isSkillDropdownOpen ? 'z-50' : 'z-30'}`}>
            <div className="flex items-center gap-2 min-w-0 overflow-hidden">
                <div className="relative shrink-0" role="presentation" onClick={(e) => e.stopPropagation()} onKeyDown={(e) => e.stopPropagation()}>
                    <Tooltip content={i18n.t('agent_card.tooltip_skills')} position="top">
                        <button
                            onClick={() => toggle(agent.id, 'skill')}
                            className={`p-1 rounded hover:bg-zinc-800 transition-colors ${isSkillDropdownOpen ? 'text-blue-400 bg-blue-900/10' : 'text-zinc-600 hover:text-zinc-300'}`}
                        >
                            <Zap size={14} fill={isSkillDropdownOpen ? "currentColor" : "none"} />
                        </button>
                    </Tooltip>

                    {isSkillDropdownOpen && (
                        <div className="absolute top-full left-0 mt-2 w-64 bg-zinc-950/95 backdrop-blur-md border border-zinc-800 rounded-lg shadow-[0_0_30px_rgba(0,0,0,0.5)] z-[110] overflow-hidden animate-in fade-in zoom-in-95 duration-100 max-h-80 overflow-y-auto custom-scrollbar relative">
                            <div className="neural-grid opacity-10 pointer-events-none" />
                            <div className="relative z-10 w-full flex flex-col">
                                {modelSlots.map((slot, idx) => {
                                    // Robust skill lookup: prioritize slot config, then agent level arrays for Primary slot only
                                    const skills = slot.config?.skills ?? (idx === 0 ? agent.skills : []) ?? [];
                                    const workflows = slot.config?.workflows ?? (idx === 0 ? agent.workflows : []) ?? [];
                                    if (!skills.length && !workflows.length) return null;

                                    return (
                                        <div key={slot.label} className={idx > 0 ? 'border-t border-zinc-800' : ''}>
                                            <div className="px-3 py-1.5 text-[10px] uppercase font-bold tracking-widest bg-zinc-950/60 border-b border-zinc-800/50 flex items-center justify-between gap-2">
                                                <span className="text-zinc-500">{slot.label}</span>
                                                <span className={`font-mono truncate ${getModelColor(slot.model || '').split(' ').find(c => c.startsWith('text-')) || 'text-zinc-400'}`}>
                                                    {slot.model}
                                                </span>
                                            </div>

                                            {skills.length > 0 && (
                                                <React.Fragment key={`${slot.label}-skills`}>
                                                    <div className="px-4 py-1.5 text-[9px] uppercase font-bold text-zinc-600 tracking-[0.2em] flex items-center gap-1.5 bg-zinc-950/80">
                                                        <Terminal size={10} className="text-blue-500" /> {i18n.t('agent_card.title_active_skills')}
                                                    </div>
                                                    {skills.map((skill) => (
                                                        <button 
                                                            key={`${slot.label}-s-${skill}`} 
                                                            onClick={() => onSkillTrigger?.(agent.id, skill, (idx + 1) as 1 | 2 | 3)}
                                                            className="w-full text-left px-4 py-2 text-xs text-zinc-300 hover:bg-zinc-900 border-l-2 border-transparent hover:border-emerald-500 hover:text-white transition-all flex items-center gap-2 group font-mono outline-none focus:bg-zinc-900 focus:border-emerald-500 focus:text-white"
                                                            aria-label={`${i18n.t('node.trigger_skill_aria')}: ${skill}`}
                                                        >
                                                            <div className="w-1.5 h-1.5 rounded-full bg-emerald-500/30 group-hover:bg-emerald-400 group-hover:shadow-[0_0_8px_rgba(16,185,129,0.5)] transition-all"></div>
                                                            <span className="truncate">{skill}</span>
                                                        </button>
                                                    ))}
                                                </React.Fragment>
                                            )}

                                            {workflows.length > 0 && (
                                                <React.Fragment key={`${slot.label}-workflows`}>
                                                    <div className="px-4 py-1.5 mt-1 border-t border-zinc-800/30 text-[9px] uppercase font-bold text-zinc-600 tracking-[0.2em] flex items-center gap-1.5 bg-zinc-950/80">
                                                        <FileText size={10} className="text-blue-400" /> {i18n.t('agent_card.title_passive_workflows')}
                                                    </div>
                                                    {workflows.map((wf) => (
                                                        <button key={`${slot.label}-w-${wf}`} onClick={() => onSkillTrigger?.(agent.id, wf, (idx + 1) as 1 | 2 | 3)}
                                                            className="w-full text-left px-4 py-2 text-xs text-zinc-300 hover:bg-zinc-900 border-l-2 border-transparent hover:border-amber-500 hover:text-white transition-all flex items-center gap-2 group font-mono">
                                                            <div className="w-1.5 h-1.5 rounded-full bg-amber-500/30 group-hover:bg-amber-400 group-hover:shadow-[0_0_8px_rgba(245,158,11,0.5)] transition-all"></div>
                                                            <span className="truncate">{wf}</span>
                                                        </button>
                                                    ))}
                                                </React.Fragment>
                                            )}
                                        </div>
                                    );
                                })}
                            </div>
                        </div>
                    )}
                </div>

                <span className="text-[10px] uppercase font-bold text-zinc-500 tracking-wider truncate">{i18n.t('agent_card.label_skills_workflows')}</span>
            </div>

            <div className="flex items-center gap-3 shrink-0">
                <div className="flex items-center gap-1 font-mono text-xs text-emerald-500/80">
                    ${(agent.costUsd || 0).toFixed(3)}
                </div>
                <div className="w-px h-2.5 bg-zinc-800" />
                <div className="flex items-center gap-1 font-mono text-xs text-zinc-500">
                    {(agent.tokensUsed / 1000).toFixed(1)}k
                </div>
            </div>
        </div>
    );
};
