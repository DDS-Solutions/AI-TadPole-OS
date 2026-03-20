/**
 * @module OrgChart
 * The Neural Command Hierarchy page.
 * Renders a multi-level agent organization chart with real-time status updates,
 * dynamic pulsating connection animations, and integrated node configuration. 
 * Uses a tiered layout: Alpha -> Nexus -> Strategic Chains.
 */
import React, { useState, useEffect, useMemo } from 'react';
import { loadAgents, persistAgentUpdate, normalizeAgent } from '../services/agentService';
import { resolveProvider } from '../utils/modelUtils';
import type { RawAgent } from '../services/agentService';
import type { Agent } from '../types';
import { TadpoleOSSocket } from '../services/socket';
import { HierarchyNode } from '../components/HierarchyNode';
import AgentConfigPanel from '../components/AgentConfigPanel';
import { agents as mockAgents } from '../data/mockAgents';
import { useWorkspaceStore } from '../stores/workspaceStore';
import { useDropdownStore } from '../stores/dropdownStore';
import { i18n } from '../i18n';

export default function OrgChart() {
    const { clusters } = useWorkspaceStore();
    const dropdownOpenId = useDropdownStore(s => s.openId);
    const closeDropdowns = useDropdownStore(s => s.close);
    const [agentsList, setAgentsList] = useState<Agent[]>([]);

    const [configAgentId, setConfigAgentId] = useState<string | null>(null);

    const availableRoles = useMemo(() =>
        Array.from(new Set(mockAgents.map(a => a.role))).sort()
        , []);

    useEffect(() => {
        const fetchAgents = async () => {
            const data = await loadAgents();
            const liveIds = new Set(data.map(a => a.id));
            const merged = [
                ...data,
                ...mockAgents.filter(ma => !liveIds.has(ma.id))
            ];
            setAgentsList(merged);
        };
        fetchAgents();

        const unsubscribeUpdates = TadpoleOSSocket.subscribeAgentUpdates((event) => {
            if (event.type === 'agent:update' && event.agentId && event.data) {
                const workspacePath = useWorkspaceStore.getState().getAgentPath(event.agentId);
                const normalizedAgent = normalizeAgent({ ...event.data, id: event.agentId } as unknown as RawAgent, workspacePath);

                setAgentsList(prev => prev.map(a =>
                    a.id === event.agentId ? normalizedAgent : a
                ));
            }
        });

        return () => unsubscribeUpdates();
    }, []);
    // ── Handlers ──────────────────────────────────────────────
    const handleSkillTrigger = (agentId: string, skill: string) => {
        setAgentsList(prev => prev.map(a =>
            a.id === agentId
                ? { ...a, status: 'active' as const, currentTask: `⚡ Executing: ${skill}...`, activeWorkflow: skill }
                : a
        ));
    };

    const handleRoleChange = (agentId: string, newRole: string) => {
        handleAgentUpdate(agentId, { role: newRole });
    };

    const handleAgentUpdate = (agentId: string, updates: Partial<Agent>) => {
        setAgentsList(prev => prev.map(a =>
            a.id === agentId ? { ...a, ...updates } : a
        ));
        persistAgentUpdate(agentId, updates);
    };

    const handleModelChange = (agentId: string, newModel: string) => {
        const provider = resolveProvider(newModel);
        handleAgentUpdate(agentId, { model: newModel, modelConfig: { modelId: newModel, provider } });
    };

    const handleModel2Change = (agentId: string, newModel: string) => {
        const provider = resolveProvider(newModel);
        handleAgentUpdate(agentId, { model2: newModel, modelConfig2: { modelId: newModel, provider } });
    };

    const handleModel3Change = (agentId: string, newModel: string) => {
        const provider = resolveProvider(newModel);
        handleAgentUpdate(agentId, { model3: newModel, modelConfig3: { modelId: newModel, provider } });
    };

    // ── Data Partitioning ──────────────────────────────────
    const hierarchyData = useMemo(() => {
        if (agentsList.length === 0) return null;

        const combinedAgents = [...agentsList];

        // Level 1: Alpha (Force Agent of Nine)
        const alpha = combinedAgents.find(a => a.name === 'Agent of Nine' || a.id === '1') || combinedAgents[0];

        const nexus = combinedAgents.find(a => (a.name === 'Tadpole' || a.id === '2') && a.id !== alpha.id)
            || combinedAgents.find(a => a.id !== alpha.id);

        const usedHigherIds = new Set([alpha.id, nexus?.id].filter(Boolean));

        // Map remaining clusters to chains (excluding cl-command which forms the root/nexus)
        const chainClusters = clusters.filter(c => c.id !== 'cl-command');
        const chains = chainClusters.slice(0, 3).map(cluster => ({
            id: cluster.id,
            name: cluster.name,
            theme: cluster.theme,
            alphaId: cluster.alphaId,
            objective: cluster.objective,
            isActive: cluster.isActive,
            agents: cluster.collaborators
                .filter(cid => !usedHigherIds.has(cid))
                .map(cid => combinedAgents.find(a => a.id === cid))
                .filter(Boolean) as Agent[]
        }));

        return {
            alpha, nexus, chains
        };
    }, [agentsList, clusters]);

    if (!hierarchyData) {
        return (
            <div className="h-full flex items-center justify-center text-zinc-500 animate-pulse font-mono text-xs uppercase tracking-widest">
                {i18n.t('org_chart.label_initializing')}
            </div>
        );
    }

    return (
        <div className="h-full flex flex-col bg-zinc-950">

            <div className="flex-1 overflow-auto p-8 custom-scrollbar relative" onClick={closeDropdowns}>
                {/* Background Grid Pattern */}
                <div className="neural-grid" />

                <div className="min-w-max pt-1 pb-12 px-12 flex flex-col items-center gap-12 relative">

                    {/* Level 1: Root (Alpha) */}
                    <div className={`relative group w-[350px] ${dropdownOpenId === hierarchyData.alpha?.id ? 'z-[100]' : 'z-30'}`}>
                        <div className="mb-4 text-center">
                            <h3 className="text-[10px] font-bold uppercase tracking-[0.2em] mb-1 text-blue-400">
                                {i18n.t('org_chart.label_command_chain')}
                            </h3>
                            <p className="text-[9px] text-zinc-500 font-medium">{i18n.t('org_chart.label_strategic_command')}</p>
                        </div>
                        <div className="absolute -inset-4 bg-blue-500/10 blur-2xl rounded-full opacity-0 group-hover:opacity-100 transition-opacity duration-700" />
                        {(() => {
                            const cluster = clusters.find(c => c.collaborators.includes(hierarchyData.alpha?.id));
                            return (
                                <HierarchyNode
                                    isRoot
                                    isAlpha
                                    availableRoles={availableRoles}
                                    onRoleChange={handleRoleChange}
                                    agent={hierarchyData.alpha}
                                    themeColor="blue"
                                    isActive={cluster?.isActive}
                                    missionObjective={cluster?.objective}
                                    onSkillTrigger={handleSkillTrigger}
                                    onConfigureClick={(id) => setConfigAgentId(id)}
                                    onModelChange={handleModelChange}
                                    onModel2Change={handleModel2Change}
                                    onModel3Change={handleModel3Change}
                                    onUpdate={handleAgentUpdate}
                                />
                            );
                        })()}

                        {/* Connection to Nexus */}
                        <div className={`absolute top-full left-1/2 -translate-x-1/2 h-[30px] w-px bg-gradient-to-b from-blue-500/50 to-blue-500/20 ${(hierarchyData.nexus?.status !== 'offline' && hierarchyData.nexus?.status !== 'idle') || hierarchyData.chains.some(c => c.isActive) ? 'vertical-pulse text-blue-500' : ''}`} />
                    </div>

                    {/* Level 2: Nexus (Coordinator) */}
                    <div className={`relative pt-0 mt-[-18px] w-[350px] ${dropdownOpenId === hierarchyData.nexus?.id ? 'z-[100]' : 'z-20'}`}>
                        <div className="absolute -inset-4 bg-zinc-500/5 blur-xl rounded-full opacity-50" />
                        {(() => {
                            const cluster = clusters.find(c => hierarchyData.nexus?.id && c.collaborators.includes(hierarchyData.nexus.id));
                            return (
                                <HierarchyNode
                                    agent={hierarchyData.nexus}
                                    availableRoles={availableRoles}
                                    onRoleChange={handleRoleChange}
                                    themeColor="zinc"
                                    isActive={cluster?.isActive}
                                    missionObjective={cluster?.objective}
                                    onSkillTrigger={handleSkillTrigger}
                                    onConfigureClick={(id) => setConfigAgentId(id)}
                                    onModelChange={handleModelChange}
                                    onModel2Change={handleModel2Change}
                                    onModel3Change={handleModel3Change}
                                    onUpdate={handleAgentUpdate}
                                />
                            );
                        })()}

                        {/* Branching SVG */}
                        <svg className="absolute top-[100%] left-1/2 -translate-x-1/2 w-[1000px] h-[42px] overflow-visible pointer-events-none">
                            <path
                                d="M 500 0 L 500 30 M 500 30 L 100 30 M 500 30 L 900 30 M 100 30 L 100 42 M 500 30 L 500 42 M 900 30 L 900 42"
                                fill="none"
                                stroke="rgba(113, 113, 122, 0.3)"
                                strokeWidth="1"
                                className={hierarchyData.chains.some(c => c.isActive || c.agents.some(a => a.status !== 'offline' && a.status !== 'idle')) ? 'neural-pulse text-zinc-500' : ''}
                            />
                            {/* Termini dots */}
                            <circle cx="100" cy="30" r="2" fill="rgba(113, 113, 122, 0.5)" />
                            <circle cx="900" cy="30" r="2" fill="rgba(113, 113, 122, 0.5)" />
                        </svg>
                    </div>

                    {/* Level 3: Chains */}
                    <div className={`flex gap-16 relative ${hierarchyData.chains.some(c => c.agents.some(a => a.id === dropdownOpenId)) ? 'z-[100]' : 'z-10'}`}>
                        {hierarchyData.chains.map(chain => (
                            <AgentChain
                                key={chain.id}
                                chain={chain}
                                dropdownOpenId={dropdownOpenId}
                                availableRoles={availableRoles}
                                handleRoleChange={handleRoleChange}
                                handleSkillTrigger={handleSkillTrigger}
                                setConfigAgentId={setConfigAgentId}
                                handleAgentUpdate={handleAgentUpdate}
                                clusters={clusters}
                                handleModelChange={handleModelChange}
                                handleModel2Change={handleModel2Change}
                                handleModel3Change={handleModel3Change}
                            />
                        ))}
                    </div>

                    {/* Agent Config Panel Overlay */}
                    {configAgentId && (() => {
                        const agent = agentsList.find(a => a.id === configAgentId) || mockAgents.find(a => a.id === configAgentId);
                        return agent ? (
                            <AgentConfigPanel
                                agent={agent}
                                onClose={() => setConfigAgentId(null)}
                                onUpdate={handleAgentUpdate}
                            />
                        ) : null;
                    })()}

                    {/* Overlay Indicators */}
                    <div className="fixed bottom-8 right-8 flex flex-col gap-2 items-end">
                        <div className="px-3 py-1 bg-black/60 border border-zinc-800 rounded-full backdrop-blur-md flex items-center gap-2 shadow-2xl">
                            <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]" />
                            <span className="text-[10px] font-mono text-zinc-400 uppercase tracking-widest">
                                {i18n.t('org_chart.label_swarm_active')}
                            </span>
                        </div>
                    </div>

                </div>
            </div>
        </div>
    );
}

/**
 * Optimized sub-component for rendering an individual agent chain.
 */
const AgentChain = React.memo(({
    chain,
    dropdownOpenId,
    availableRoles,
    handleRoleChange,
    handleSkillTrigger,
    setConfigAgentId,
    handleAgentUpdate,
    clusters,
    handleModelChange,
    handleModel2Change,
    handleModel3Change
}: {
    chain: {
        id: string;
        name: string;
        theme: string;
        alphaId?: string;
        objective?: string;
        isActive?: boolean;
        agents: Agent[];
    },
    dropdownOpenId: string | null,
    availableRoles: string[],
    handleRoleChange: (id: string, role: string) => void,
    handleSkillTrigger: (id: string, skill: string) => void,
    setConfigAgentId: (id: string) => void,
    handleAgentUpdate: (id: string, updates: Partial<Agent>) => void,
    handleModelChange: (id: string, m: string) => void,
    handleModel2Change: (id: string, m: string) => void,
    handleModel3Change: (id: string, m: string) => void,
    clusters: { id: string, name: string, theme: string, alphaId?: string, collaborators: string[], isActive?: boolean }[]
}) => {
    return (
        <div className="flex flex-col items-center gap-12 relative">
            {/* Column Header */}
            <div className="mb-4 text-center">
                <h3 className={`text-[10px] font-bold uppercase tracking-[0.2em] mb-1 
                ${chain.theme === 'cyan' ? 'text-cyan-400' :
                        chain.theme === 'zinc' ? 'text-zinc-400' : 'text-amber-400'}`}>
                    Chain {chain.id}
                </h3>
                <p className="text-[9px] text-zinc-500 font-medium">{chain.name}</p>
            </div>

            {/* Agent Chain */}
            <div className="flex flex-col gap-12 relative">
                {chain.agents.map((agent: Agent, idx: number) => (
                    <div key={agent.id} className="relative w-[350px]" style={{ zIndex: dropdownOpenId === agent.id ? 110 : (100 - idx) }}>
                        <HierarchyNode
                            agent={agent}
                            availableRoles={availableRoles}
                            onRoleChange={handleRoleChange}
                            themeColor={chain.theme}
                            isAlpha={agent.id === chain.alphaId}
                            isActive={clusters.find(c => c.id === chain.id)?.isActive}
                            missionObjective={chain.objective}
                            onSkillTrigger={handleSkillTrigger}
                            onConfigureClick={(id) => setConfigAgentId(id)}
                            onModelChange={handleModelChange}
                            onModel2Change={handleModel2Change}
                            onModel3Change={handleModel3Change}
                            onUpdate={handleAgentUpdate}
                        />

                        {/* Vertical Connection */}
                        {idx < chain.agents.length - 1 && (
                            <div className={`absolute top-full left-1/2 -translate-x-1/2 h-12 w-px 
                            ${chain.theme === 'cyan' ? 'bg-cyan-500/20' :
                                    chain.theme === 'zinc' ? 'bg-zinc-500/20' : 'bg-amber-500/20'}
                            ${chain.isActive || (chain.agents[idx].status !== 'offline' && chain.agents[idx].status !== 'idle') || (chain.agents[idx + 1].status !== 'offline' && chain.agents[idx + 1].status !== 'idle') ? `vertical-pulse text-${chain.theme}-500` : ''}`}
                            />
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
});


