import React, { useState, useCallback } from 'react';
import { AnimatePresence } from 'framer-motion';
import type { AgentStatus } from '../../types';
import { HierarchyNode } from '../HierarchyNode';
import type { Agent, MissionCluster } from '../../types';
import { OperationTabHeader } from './OperationTabHeader';
import { PortalWindow } from '../ui/PortalWindow';
import { ExternalLink } from 'lucide-react';
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
    onToggleCluster: (clusterId: string) => void;
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
    handleAgentUpdate,
    onToggleCluster
}): React.ReactElement => {
    const [activeTabId, setActiveTabId] = useState('global');
    const [detachedTabIds, setDetachedTabIds] = useState<string[]>([]);

    // ── Detachment Logic ────────────────────────────────────────

    const handleDetachTab = useCallback((id: string) => {
        setDetachedTabIds(prev => prev.includes(id) ? prev : [...prev, id]);
    }, []);

    const handleReattachTab = useCallback((id: string) => {
        setDetachedTabIds(prev => prev.filter(tid => tid !== id));
    }, []);

    // ── Filtering Logic ────────────────────────────────────────

    const getFilteredAgents = useCallback((tabId: string) => {
        if (tabId === 'global') {
            return agents.filter(agent => 
                assignedAgentIds.has(String(agent.id)) || 
                (['active', 'thinking', 'speaking', 'coding'] as AgentStatus[]).includes(agent.status)
            );
        }

        // Must be a cluster ID
        const activeCluster = clusters.find(c => c.id === tabId);
        if (!activeCluster) return [];

        return agents.filter(agent => 
            activeCluster.collaborators.map(String).includes(String(agent.id))
        );
    }, [agents, assignedAgentIds, clusters]);

    const renderGridContent = (tabId: string) => {
        const filtered = getFilteredAgents(tabId);
        
        return (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 gap-6 pb-4 relative z-10">
                {filtered.map((agent): React.ReactElement => {
                    const agentCluster = clusters.find(c => c.collaborators.map(String).includes(String(agent.id)));
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
                
                {filtered.length === 0 && (
                    <div className="col-span-full py-20 flex flex-col items-center justify-center text-zinc-500 gap-2 opacity-50">
                        <div className="w-10 h-10 border border-dashed border-zinc-700 rounded-lg flex items-center justify-center">
                            <span className="text-xl">!</span>
                        </div>
                        <p className="text-[10px] uppercase tracking-widest font-bold">No Neural Nodes Detected in this Sector</p>
                    </div>
                )}
            </div>
        );
    };

    const isCurrentTabDetached = detachedTabIds.includes(activeTabId);

    return (
        <>
            <div className="flex-1 overflow-y-auto min-h-0 bg-zinc-950 border border-zinc-800 rounded-xl flex flex-col custom-scrollbar relative">
                <div className="neural-grid opacity-[0.1]" />
                
                <OperationTabHeader 
                    activeTabId={activeTabId}
                    clusters={clusters}
                    onTabChange={setActiveTabId}
                    onToggleCluster={onToggleCluster}
                    onDetachTab={handleDetachTab}
                />

                <div className="flex-1 overflow-y-auto px-6 pt-6 custom-scrollbar pb-6 relative">
                    {isCurrentTabDetached ? (
                        <div className="absolute inset-0 flex items-center justify-center bg-zinc-950/80 backdrop-blur-sm z-20">
                            <div className="text-center space-y-4">
                                <div className="relative inline-block">
                                    <ExternalLink size={48} className="text-zinc-800 animate-pulse" />
                                    <div className="absolute inset-0 bg-blue-500/10 blur-xl rounded-full" />
                                </div>
                                <div className="space-y-1">
                                    <h3 className="text-lg font-bold tracking-tight text-zinc-200">{i18n.t('layout.sector_detached') || 'Sector Detached'}</h3>
                                    <p className="text-sm text-zinc-500 font-mono">
                                        {i18n.t('layout.link_established') || 'Neural Link Established'} :: {activeTabId.toUpperCase()}
                                    </p>
                                </div>
                                <button 
                                    onClick={() => handleReattachTab(activeTabId)}
                                    className="px-4 py-2 bg-zinc-100 text-black text-xs font-bold uppercase tracking-widest rounded-lg hover:bg-white transition-all shadow-lg active:scale-95"
                                >
                                    {i18n.t('layout.recall_sector') || 'Recall Sector'}
                                </button>
                            </div>
                        </div>
                    ) : (
                        renderGridContent(activeTabId)
                    )}
                </div>
            </div>

            <AnimatePresence>
                {detachedTabIds.map(tabId => {
                    const cluster = clusters.find(c => c.id === tabId);
                    const title = tabId === 'global' ? 'Global Swarm View' : (cluster?.name || tabId.toUpperCase());
                    
                    return (
                        <PortalWindow 
                            key={tabId}
                            onClose={() => handleReattachTab(tabId)} 
                            title={`LIVE STATUS: ${title}`}
                            id={`detached-op-${tabId}`}
                        >
                            <div className="bg-zinc-950 p-6 h-full overflow-y-auto custom-scrollbar">
                                {renderGridContent(tabId)}
                            </div>
                        </PortalWindow>
                    );
                })}
            </AnimatePresence>
        </>
    );
};
