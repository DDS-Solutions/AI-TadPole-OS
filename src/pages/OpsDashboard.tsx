import { useState, useEffect, useMemo, useCallback } from 'react';
import { TadpoleOSService } from '../services/tadpoleosService';
import { resolveProvider } from '../utils/modelUtils';
import { EventBus } from '../services/eventBus';
import { useDropdownStore } from '../stores/dropdownStore';
import { useRoleStore } from '../stores/roleStore';
import { i18n } from '../i18n';

import TerminalComponent from '../components/Terminal';
import AgentConfigPanel from '../components/AgentConfigPanel';
import ErrorBoundary from '../components/ErrorBoundary';

import { useHeaderStore } from '../stores/headerStore';
import { useDashboardData } from '../hooks/useDashboardData';
import { DashboardHeaderActions } from '../components/dashboard/DashboardHeaderActions';
import { StatMetrics } from '../components/dashboard/StatMetrics';
import { AgentStatusGrid } from '../components/dashboard/AgentStatusGrid';
import { SystemLog } from '../components/dashboard/SystemLog';
import { useTabStore } from '../stores/tabStore';
import { ExternalLink } from 'lucide-react';
import type { Agent } from '../types';

/**
 * OpsDashboard
 * 
 * The central command-and-control center for the Tadpole OS agent swarm.
 */
export default function OpsDashboard() {
    const {
        isOnline, agentsList, agentsCount, activeAgents, totalCost, budgetUtil,
        nodes, nodesLoading, assignedAgentIds, availableRoles,
        clusters, updateAgent, addAgent, discoverNodes, recruitVelocity
    } = useDashboardData();

    const { isSystemLogDetached, toggleSystemLogDetachment } = useTabStore();

    const [deployingTarget, setDeployingTarget] = useState<string | null>(null);
    const [configAgentId, setConfigAgentId] = useState<string | null>(null);

    const setHeaderActions = useHeaderStore(s => s.setActions);
    const closeDropdowns = useDropdownStore(s => s.close);

    // ── Handlers ──────────────────────────────────────────────

    const handleAgentUpdate = (id: string, updates: Partial<Agent>) => {
        updateAgent(id, updates);
    };

    const handleCreateAgent = async (params: Partial<Agent>) => {
        try {
            const newId = `agent-${Math.random().toString(36).substring(2, 11)}`;
            const newAgent: Agent = {
                id: newId,
                name: params.name || i18n.t('ops.placeholder_name'),
                role: params.role || 'assistant',
                status: 'idle',
                model: params.model || 'gemini-1.5-flash',
                capabilities: params.capabilities || [],
                workflows: params.workflows || [],
                costUsd: 0,
                budgetUsd: params.budgetUsd || 0,
                themeColor: params.themeColor || '#10b981',
                valence: 0.5,
                ...params
            } as Agent;

            const success = await TadpoleOSService.createAgent(newAgent);
            if (success) {
                await addAgent(newAgent);
                EventBus.emit({ text: i18n.t('ops.event_agent_init', { name: newAgent.name }), severity: 'success', source: 'System' });
            }
        } catch (error) {
            console.error('Failed to create agent:', error);
            EventBus.emit({ text: i18n.t('ops.event_agent_fail'), severity: 'error', source: 'System' });
        }
    };

    const handleRoleChange = (agentId: string, newRole: string) => {
        const roles = useRoleStore.getState().roles;
        const newActions = roles[newRole] || { capabilities: [], workflows: [] };
        handleAgentUpdate(agentId, {
            role: newRole,
            capabilities: newActions.capabilities,
            workflows: newActions.workflows
        });
    };

    const handleSkillTrigger = async (agentId: string, skill: string, slot: 1 | 2 | 3 = 1) => {
        const agent = agentsList.find(a => a.id === agentId);
        if (!agent) return;

        updateAgent(agentId, {
            status: 'active' as const,
            currentTask: i18n.t('ops.event_executing', { skill }),
            activeModelSlot: slot
        });

        let modelId = agent.model;
        let provider = agent.modelConfig?.provider;

        if (slot === 2) {
            modelId = agent.model2 || modelId;
            provider = agent.modelConfig2?.provider || provider;
        } else if (slot === 3) {
            modelId = agent.model3 || modelId;
            provider = agent.modelConfig3?.provider || provider;
        }

        try {
            const agentCluster = clusters.find(c => c.collaborators.includes(agentId));
            await TadpoleOSService.sendCommand(agentId, skill, modelId || 'gemini-1.5-flash', provider || 'google', agentCluster?.id, agent.department, agentCluster?.budgetUsd);
        } catch (e) {
            console.error("❌ [OpsDashboard] Failed to trigger skill:", e);
            EventBus.emit({
                text: i18n.t('ops.event_trigger_fail', { skill, name: agent.name, error: String(e) }),
                severity: 'error',
                source: 'System'
            });
        }
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

    const handleDeploy = useCallback(async (nodeId: string, nodeName: string) => {
        setDeployingTarget(nodeId);
        const targetNumber = nodeId === 'bunker-1' ? 1 : nodeId === 'bunker-2' ? 2 : 1;

        EventBus.emit({ text: i18n.t('ops.event_deploy_start', { name: nodeName }), severity: 'info', source: 'System' });
        try {
            const data = await TadpoleOSService.deployEngine(targetNumber);
            EventBus.emit({ text: i18n.t('ops.event_deploy_success', { name: nodeName }), severity: 'success', source: 'System' });
            if (data.output) {
                EventBus.emit({ text: data.output.slice(-500), severity: 'info', source: 'System' });
            }
        } catch (e: unknown) {
            const errorMsg = e instanceof Error ? e.message : String(e);
            EventBus.emit({ text: i18n.t('ops.event_deploy_error', { error: errorMsg }), severity: 'error', source: 'System' });
        } finally {
            setDeployingTarget(null);
        }
    }, []);

    const headerActions = useMemo(() => (
        <DashboardHeaderActions
            onDiscover={discoverNodes}
            nodesLoading={nodesLoading}
            nodes={nodes}
            onDeploy={handleDeploy}
            deployingTarget={deployingTarget}
            onInitializeAgent={() => setConfigAgentId('new')}
        />
    ), [discoverNodes, nodesLoading, nodes, deployingTarget, handleDeploy]);

    useEffect(() => {
        setHeaderActions(headerActions);
    }, [headerActions, setHeaderActions]);

    return (
        <ErrorBoundary>
            <div className="flex flex-col h-full gap-6">
                <div 
                    className="grid grid-cols-1 lg:grid-cols-4 gap-6 flex-1 min-h-0"
                    onClick={() => closeDropdowns()}
                >
                    <div className="lg:col-span-3 flex flex-col gap-6 min-h-0">
                        <StatMetrics
                            isOnline={isOnline}
                            activeAgents={activeAgents}
                            agentsCount={agentsCount}
                            totalCost={totalCost}
                            budgetUtil={budgetUtil}
                            recruitVelocity={recruitVelocity}
                        />

                        <AgentStatusGrid
                            agents={agentsList}
                            assignedAgentIds={assignedAgentIds}
                            availableRoles={availableRoles}
                            clusters={clusters}
                            onSkillTrigger={handleSkillTrigger}
                            onModelChange={handleModelChange}
                            onModel2Change={handleModel2Change}
                            onModel3Change={handleModel3Change}
                            onRoleChange={handleRoleChange}
                            onConfigureClick={(id) => setConfigAgentId(id)}
                            handleAgentUpdate={handleAgentUpdate}
                        />
                    </div>

                    {isSystemLogDetached ? (
                        <div className="xl:col-span-1 sovereign-card flex flex-col items-center justify-center p-8 text-center space-y-4 relative overflow-hidden group">
                            <div className="neural-grid opacity-[0.05]" />
                            <div className="relative">
                                <ExternalLink size={32} className="text-zinc-800 animate-pulse" />
                                <div className="absolute inset-0 bg-blue-500/10 blur-xl rounded-full" />
                            </div>
                            <div className="space-y-1 relative z-10">
                                <h3 className="text-sm font-bold text-zinc-400">{i18n.t('dashboard.log_title')} Detached</h3>
                                <p className="text-[10px] text-zinc-600 font-mono italic">LINK_ACTIVE :: EXTERNAL_MONITOR</p>
                            </div>
                            <button 
                                onClick={() => toggleSystemLogDetachment()}
                                className="relative z-10 px-3 py-1.5 bg-zinc-900 border border-zinc-800 text-zinc-400 text-[10px] font-bold uppercase tracking-widest rounded-lg hover:bg-zinc-800 hover:text-zinc-200 transition-all active:scale-95"
                            >
                                {i18n.t('layout.recall_sector')}
                            </button>
                        </div>
                    ) : (
                        <SystemLog />
                    )}
                </div>

                {configAgentId && (
                    <AgentConfigPanel
                        agent={configAgentId === 'new' ? undefined : (agentsList.find(a => a.id === configAgentId) as Agent)} 
                        onClose={() => setConfigAgentId(null)}
                        onUpdate={(id, updates) => {
                            if (id === 'new') {
                                // Create new agent logic
                                handleCreateAgent(updates);
                            } else {
                                handleAgentUpdate(id, updates);
                            }
                        }}
                        isNew={configAgentId === 'new'}
                    />
                )}

                <TerminalComponent agents={agentsList} />
            </div>
        </ErrorBoundary>
    );
}
