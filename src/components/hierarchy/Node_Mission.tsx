import React from 'react';
import { Link } from 'react-router-dom';
import { Target, AlertTriangle, Shield, Users } from 'lucide-react';
import { i18n } from '../../i18n';
import type { Agent } from '../../types';

interface Node_Mission_Props {
    agent: Agent;
    is_active?: boolean;
    mission_objective?: string;
}

export const Node_Mission: React.FC<Node_Mission_Props> = ({ agent, is_active, mission_objective }) => {
    const has_mission = mission_objective || agent.active_mission;

    return (
        <div className="min-h-[26px] py-0.5 flex items-center justify-between z-20">
            {has_mission ? (
                <Link to="/missions" className={`
                    text-[10px] font-bold px-2 py-0.5 rounded-md border flex items-start gap-1 hover:brightness-125 transition-all no-underline cursor-pointer max-h-[60px] overflow-y-auto custom-scrollbar
                    ${is_active ? 'text-emerald-400 border-emerald-500/50 bg-emerald-950/30' :
                        agent.active_mission?.is_degraded ? 'text-amber-400 border-amber-900/50 bg-amber-950/30' :
                            agent.active_mission?.priority === 'high' ? 'text-red-400 border-red-900/50 bg-red-950/30' :
                                agent.active_mission?.priority === 'medium' ? 'text-amber-400 border-amber-900/50 bg-amber-950/30' :
                                    'text-emerald-400 border-emerald-900/50 bg-emerald-950/30'}
                `}>
                    {agent.active_mission?.is_degraded ? (
                        <AlertTriangle size={8} className="fill-current mt-1 shrink-0" />
                    ) : (
                        <Target size={8} className="animate-pulse fill-current mt-1 shrink-0" />
                    )}
                    <span className="uppercase tracking-tighter break-words leading-tight">
                        {agent.active_mission?.is_degraded ? i18n.t('agent_card.prefix_degraded') : i18n.t('agent_card.prefix_mission')}
                        {mission_objective || agent.active_mission?.objective}
                    </span>
                </Link>
            ) : (
                <div className="text-[10px] font-bold px-2 py-0.5 rounded-md border border-zinc-800/30 bg-zinc-900/5 text-zinc-600/50 flex items-center gap-1 italic">
                    <Shield size={8} className="opacity-20" />
                    <span className="uppercase tracking-tighter">{i18n.t('agent_card.no_mission')}</span>
                </div>
            )}

            {agent.workspace_path?.includes('shared') && (
                <div className="flex items-center gap-1 text-[10px] font-mono text-blue-400/60 uppercase animate-in fade-in slide-in-from-right-2">
                    <Users size={10} strokeWidth={3} />
                    <span>{i18n.t('agent_card.label_cluster_hub')}</span>
                </div>
            )}
        </div>
    );
};
