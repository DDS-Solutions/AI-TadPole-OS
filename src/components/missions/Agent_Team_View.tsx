/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Mission-specific agent coordination view. 
 * Orchestrates "Alpha Agent" promotion, collaborator assignment, and department-themed status badges for swarm mission clusters.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Assign/Unassign RPC desync (agent remains in local `collaborators` array but not in store), Alpha promotion race condition, or theme color fallback to CSS default.
 * - **Telemetry Link**: Search for `[Agent_Team_View]` or `Alpha Promoted` in browser logs.
 */

import React from 'react';
import { Crown, Plus, Trash2, Users } from 'lucide-react';
import { Tooltip, Tw_Empty_State } from '../ui';
import type { Agent } from '../../types';
import type { Mission_Cluster } from '../../stores/workspace_store';
import { i18n } from '../../i18n';

interface AgentTeamViewProps {
    active_cluster: Mission_Cluster;
    agents: Agent[];
    available_agents: Agent[];
    on_assign: (id: string) => void;
    on_unassign: (id: string) => void;
    on_set_alpha: (id: string) => void;
}

export const Agent_Team_View: React.FC<AgentTeamViewProps> = ({
    active_cluster,
    agents,
    available_agents,
    on_assign,
    on_unassign,
    on_set_alpha
}) => {
    const DEFAULT_THEMES: Record<string, string> = {
        cyan: '#06b6d4',
        zinc: '#71717a',
        amber: '#f59e0b',
        blue: '#3b82f6'
    };
    
    const agentLookup = React.useMemo(() => 
        new Map(agents.map(a => [a.id, a])), 
    [agents]);

    return (
        <div className="flex flex-col gap-8">
            {/* Assigned Agents */}
            <div>
                <h4 className="text-xs font-bold text-zinc-500 uppercase tracking-widest mb-4 flex items-center gap-2">
                    <Users size={12} /> {i18n.t('missions.header_team')}
                </h4>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                    {(active_cluster.collaborators || []).map(id => {
                        const agent = agentLookup.get(id);
                        const is_alpha = active_cluster.alpha_id === id;
                        const agent_color = agent?.theme_color || DEFAULT_THEMES[active_cluster.theme] || DEFAULT_THEMES.blue;

                        return agent ? (
                            <div
                                key={id}
                                className="p-3 bg-zinc-900 border rounded-xl flex items-center justify-between group transition-all duration-300"
                                style={{
                                    borderColor: `${agent_color}${is_alpha ? '50' : '30'}`,
                                    boxShadow: is_alpha ? `0 0 15px ${agent_color}20` : 'none'
                                }}
                            >
                                <div className="flex items-center gap-3">
                                    <Tooltip content={is_alpha ? i18n.t('missions.tooltip_alpha') : i18n.t('missions.tooltip_promote_alpha')} position="top">
                                        <button
                                            onClick={() => on_set_alpha(id)}
                                            className="w-8 h-8 rounded-lg flex items-center justify-center border transition-all"
                                            style={{
                                                borderColor: `${agent_color}${is_alpha ? '50' : '30'}`,
                                                backgroundColor: `${agent_color}10`
                                            }}
                                        >
                                            {is_alpha ? (
                                                <Crown size={14} style={{ color: agent_color }} />
                                            ) : (
                                                <div className="relative flex items-center justify-center">
                                                    <span className="text-xs font-mono font-bold group-hover:opacity-0 transition-opacity" style={{ color: agent_color }}>{agent.name[0]}</span>
                                                    <Crown size={12} className="absolute opacity-0 group-hover:opacity-40 transition-opacity" style={{ color: agent_color }} />
                                                </div>
                                            )}
                                        </button>
                                    </Tooltip>
                                    <div className="flex flex-col">
                                        <div className="flex items-center gap-2">
                                            <span className="text-xs font-bold text-zinc-200">{agent.name}</span>
                                            {is_alpha && <Crown size={10} className="text-amber-400 fill-amber-400/20" />}
                                            {is_alpha && (
                                                <span
                                                    className="text-[8px] px-1 rounded uppercase font-bold tracking-tighter border"
                                                    style={{
                                                        color: agent_color,
                                                        borderColor: `${agent_color}40`,
                                                        backgroundColor: `${agent_color}15`
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
                                        onClick={() => on_unassign(id)}
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
                    {(available_agents || []).length > 0 ? (available_agents || []).map(agent => {
                        const agent_color = agent.theme_color || '#3b82f6';
                        return (
                            <button
                                key={agent.id}
                                onClick={() => on_assign(agent.id)}
                                className="p-3 bg-zinc-900/50 border border-dashed rounded-xl flex items-center gap-3 text-left transition-all group hover:border-[#3b82f660] hover:bg-[#3b82f608]"
                                style={{ borderColor: `${agent_color}30` }}
                            >
                                <div
                                    className="w-8 h-8 rounded-lg flex items-center justify-center border transition-colors"
                                    style={{ borderColor: `${agent_color}40`, backgroundColor: `${agent_color}10` }}
                                >
                                    <Plus size={14} style={{ color: agent_color }} />
                                </div>
                                <div className="flex flex-col">
                                    <span className="text-xs font-bold text-zinc-400 group-hover:text-zinc-200">{agent.name}</span>
                                    <span className="text-[11px] text-zinc-600 uppercase">{agent.role}</span>
                                </div>
                            </button>
                        );
                    }) : (
                        <Tw_Empty_State title={i18n.t('missions.empty_agents_title')} description={i18n.t('missions.empty_agents_desc')} />
                    )}
                </div>
            </div>
        </div>
    );
};


// Metadata: [Agent_Team_View]
