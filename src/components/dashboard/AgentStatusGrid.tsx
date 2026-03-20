import React from 'react';
import { Activity } from 'lucide-react';
import { Tooltip } from '../ui';
import { HierarchyNode } from '../HierarchyNode';
import type { Agent, MissionCluster } from '../../types';
import { i18n } from '../../i18n';

interface AgentStatusGridProps {
    agents: Agent[];
    assignedAgentIds: Set<string>;
    availableRoles: string[];
    clusters: MissionCluster[];
    onSkillTrigger: (agentId: string, skill: string, slot?: 1 | 2 | 3) => void;
    onConfigureClick: (id: string) => void;
    onModelChange: (agentId: string, newModel: string) => void;
    onModel2Change: (agentId: string, newModel: string) => void;
    onModel3Change: (agentId: string, newModel: string) => void;
    onRoleChange: (agentId: string, newRole: string) => void;
    handleAgentUpdate: (agentId: string, updates: Partial<Agent>) => void;
}

export const AgentStatusGrid: React.FC<AgentStatusGridProps> = ({
    agents,
    assignedAgentIds,
    availableRoles,
    clusters,
    onSkillTrigger,
    onConfigureClick,
    onModelChange,
    onModel2Change,
    onModel3Change,
    onRoleChange,
    handleAgentUpdate
}): React.ReactElement => {
    return (
        <div className="flex-1 overflow-y-auto min-h-0 bg-zinc-950 border border-zinc-800 rounded-xl px-6 pb-6 custom-scrollbar relative">
            <div className="neural-grid opacity-[0.1]" />
            <Tooltip content={i18n.t('dashboard.live_status_tooltip')} position="right">
                <h3 className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] mb-6 sticky top-0 bg-zinc-950/80 backdrop-blur-md pt-6 pb-3 border-b border-zinc-800/50 z-20 flex items-center gap-2 cursor-help">
                    <Activity size={12} className="text-blue-500" /> {i18n.t('dashboard.live_status')}
                </h3>
            </Tooltip>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 gap-6 pb-4 relative z-10">
                {agents
                    .filter(agent => assignedAgentIds.has(agent.id))
                    .map((agent): React.ReactElement => {
                        const agentCluster = clusters.find(c => c.collaborators.includes(agent.id));
                        const isAlpha = agentCluster?.alphaId === agent.id;
                        const isActive = agentCluster?.isActive;
                        const missionObjective = agentCluster?.objective || agent.activeMission?.objective;
                        const agentColor = agent.themeColor || '#10b981';

                        return (
                            <div key={agent.id} className="agent-grid-item group/item relative">
                                <div
                                    className="absolute -inset-4 blur-3xl rounded-full opacity-0 group-hover/item:opacity-20 transition-opacity duration-700 pointer-events-none"
                                    style={{ backgroundColor: agentColor }}
                                />
                                <HierarchyNode
                                    agent={agent}
                                    availableRoles={availableRoles}
                                    onSkillTrigger={onSkillTrigger}
                                    onConfigureClick={onConfigureClick}
                                    isAlpha={isAlpha}
                                    isActive={isActive}
                                    missionObjective={missionObjective}
                                    onModelChange={onModelChange}
                                    onModel2Change={onModel2Change}
                                    onModel3Change={onModel3Change}
                                    onRoleChange={onRoleChange}
                                    onUpdate={handleAgentUpdate}
                                />
                            </div>
                        );
                    })}
            </div>
        </div>
    );
};
