import React from 'react';
import { Crown, Plus, Trash2, Users } from 'lucide-react';
import { Tooltip, TwEmptyState } from '../ui';
import type { Agent } from '../../types';
import type { MissionCluster } from '../../stores/workspaceStore';
import { i18n } from '../../i18n';

interface AgentTeamViewProps {
    activeCluster: MissionCluster;
    agents: Agent[];
    availableAgents: Agent[];
    onAssign: (id: string) => void;
    onUnassign: (id: string) => void;
    onSetAlpha: (id: string) => void;
}

export const AgentTeamView: React.FC<AgentTeamViewProps> = ({
    activeCluster,
    agents,
    availableAgents,
    onAssign,
    onUnassign,
    onSetAlpha
}) => {
    return (
        <div className="flex flex-col gap-8">
            {/* Assigned Agents */}
            <div>
                <h4 className="text-xs font-bold text-zinc-500 uppercase tracking-widest mb-4 flex items-center gap-2">
                    <Users size={12} /> {i18n.t('missions.header_team')}
                </h4>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                    {activeCluster.collaborators.map(id => {
                        const agent = agents.find(a => String(a.id) === String(id));
                        const isAlpha = activeCluster.alphaId === id;
                        const agentColor = agent?.themeColor || (activeCluster.theme === 'cyan' ? '#06b6d4' : activeCluster.theme === 'zinc' ? '#71717a' : activeCluster.theme === 'amber' ? '#f59e0b' : '#3b82f6');

                        return agent ? (
                            <div
                                key={id}
                                className="p-3 bg-zinc-900 border rounded-xl flex items-center justify-between group transition-all duration-300"
                                style={{
                                    borderColor: `${agentColor}${isAlpha ? '50' : '30'}`,
                                    boxShadow: isAlpha ? `0 0 15px ${agentColor}20` : 'none'
                                }}
                            >
                                <div className="flex items-center gap-3">
                                    <Tooltip content={isAlpha ? i18n.t('missions.tooltip_alpha') : i18n.t('missions.tooltip_promote_alpha')} position="top">
                                        <button
                                            onClick={() => onSetAlpha(id)}
                                            className="w-8 h-8 rounded-lg flex items-center justify-center border transition-all"
                                            style={{
                                                borderColor: `${agentColor}${isAlpha ? '50' : '30'}`,
                                                backgroundColor: `${agentColor}10`
                                            }}
                                        >
                                            {isAlpha ? (
                                                <Crown size={14} style={{ color: agentColor }} />
                                            ) : (
                                                <div className="relative flex items-center justify-center">
                                                    <span className="text-xs font-mono font-bold group-hover:opacity-0 transition-opacity" style={{ color: agentColor }}>{agent.name[0]}</span>
                                                    <Crown size={12} className="absolute opacity-0 group-hover:opacity-40 transition-opacity" style={{ color: agentColor }} />
                                                </div>
                                            )}
                                        </button>
                                    </Tooltip>
                                    <div className="flex flex-col">
                                        <div className="flex items-center gap-2">
                                            <span className="text-xs font-bold text-zinc-200">{agent.name}</span>
                                            {isAlpha && <Crown size={10} className="text-amber-400 fill-amber-400/20" />}
                                            {isAlpha && (
                                                <span
                                                    className="text-[8px] px-1 rounded uppercase font-bold tracking-tighter border"
                                                    style={{
                                                        color: agentColor,
                                                        borderColor: `${agentColor}40`,
                                                        backgroundColor: `${agentColor}15`
                                                    }}
                                                >
                                                    {i18n.t('missions.badge_alpha')}
                                                </span>
                                            )}
                                        </div>
                                        <span className="text-[11px] text-zinc-500 font-mono uppercase">{agent.role}</span>
                                    </div>
                                </div>
                                <Tooltip content={i18n.t('missions.tooltip_unassign')} position="left">
                                    <button
                                        onClick={() => onUnassign(id)}
                                        className="p-1.5 opacity-0 group-hover:opacity-100 transition-opacity text-zinc-600 hover:text-red-400"
                                    >
                                        <Trash2 size={14} />
                                    </button>
                                </Tooltip>
                            </div>
                        ) : null;
                    })}
                </div>
            </div>

            {/* Available Agents */}
            <div>
                <h4 className="text-xs font-bold text-zinc-500 uppercase tracking-widest mb-4 flex items-center gap-2">
                    <Plus size={12} /> {i18n.t('missions.header_assign')}
                </h4>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                    {availableAgents.length > 0 ? availableAgents.map(agent => {
                        const agentColor = agent.themeColor || '#3b82f6';
                        return (
                            <button
                                key={agent.id}
                                onClick={() => onAssign(agent.id)}
                                className="p-3 bg-zinc-900/50 border border-dashed rounded-xl flex items-center gap-3 text-left transition-all group hover:border-[#3b82f660] hover:bg-[#3b82f608]"
                                style={{ borderColor: `${agentColor}30` }}
                            >
                                <div
                                    className="w-8 h-8 rounded-lg flex items-center justify-center border transition-colors"
                                    style={{ borderColor: `${agentColor}40`, backgroundColor: `${agentColor}10` }}
                                >
                                    <Plus size={14} style={{ color: agentColor }} />
                                </div>
                                <div className="flex flex-col">
                                    <span className="text-xs font-bold text-zinc-400 group-hover:text-zinc-200">{agent.name}</span>
                                    <span className="text-[11px] text-zinc-600 uppercase">{agent.role}</span>
                                </div>
                            </button>
                        );
                    }) : (
                        <TwEmptyState title={i18n.t('missions.empty_agents_title')} description={i18n.t('missions.empty_agents_desc')} />
                    )}
                </div>
            </div>
        </div>
    );
};
