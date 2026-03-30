import { Target, Repeat, Check, X, Cpu } from 'lucide-react';
import { useWorkspaceStore } from '../stores/workspaceStore';
import { TadpoleOSService } from '../services/tadpoleosService';
import { EventBus } from '../services/eventBus';
import { TadpoleOSSocket, type HandoffEvent } from '../services/socket';
import { useState, useEffect } from 'react';
import { getThemeColors } from '../utils/agentUIUtils';
import { TwEmptyState } from '../components/ui';
import { useAgentStore } from '../stores/agentStore';
import { resolveAgentModelConfig } from '../utils/modelUtils';
import { useTraceStore } from '../stores/traceStore';
import { i18n } from '../i18n';

// Modular Components
import { ClusterSidebar } from '../components/missions/ClusterSidebar';
import { MissionHeader } from '../components/missions/MissionHeader';
import { NeuralMap } from '../components/missions/NeuralMap';
import { AgentTeamView } from '../components/missions/AgentTeamView';
import { BufferedTranscriptView } from '../components/transcript/BufferedTranscriptView';

export default function Missions() {
    const {
        clusters,
        assignAgentToCluster,
        unassignAgentFromCluster,
        updateClusterObjective,
        setAlphaNode,
        deleteCluster,
        toggleClusterActive,
        approveBranch,
        rejectBranch,
        updateClusterDepartment,
        updateClusterBudget,
        toggleMissionAnalysis
    } = useWorkspaceStore();
    const [selectedClusterId, setSelectedClusterId] = useState<string | null>(clusters[0]?.id || null);
    const { agents, fetchAgents, updateAgent: storeUpdateAgent, isLoading: agentsLoading } = useAgentStore();

    useEffect(() => {
        if (agents.length === 0) {
            fetchAgents();
        }

        const unsubscribeHandoff = TadpoleOSSocket.subscribeHandoff((event: HandoffEvent) => {
            const tgt = event.toCluster || 'unknown';
            const desc = (event.payload?.description as string) || `Cross-cluster task handoff triggered for agent ${event.agentId}.`;

            useWorkspaceStore.getState().receiveHandoff(event.fromCluster || 'unknown', tgt, desc);
            EventBus.emit({
                source: 'System',
                text: i18n.t('missions.event_handoff', { tgt }),
                severity: 'info',
                missionId: tgt
            });
        });

        return () => unsubscribeHandoff();
    }, [fetchAgents, agents.length]);

    const activeCluster = clusters.find(c => c.id === selectedClusterId);
    const assignedAgentIds = new Set(clusters.flatMap(c => c.collaborators).map(String));
    const availableAgents = agents.filter(a => !assignedAgentIds.has(String(a.id)));

    const handleRunMission = async () => {
        if (!activeCluster) return;

        if (!activeCluster.alphaId) {
            EventBus.emit({
                source: 'System',
                text: i18n.t('missions.event_fail_alpha', { name: activeCluster.name }),
                severity: 'error',
                missionId: activeCluster.id
            });
            return;
        }

        if (!activeCluster.objective) {
            EventBus.emit({
                source: 'System',
                text: i18n.t('missions.event_fail_objective', { name: activeCluster.name }),
                severity: 'error',
                missionId: activeCluster.id
            });
            return;
        }

        const alphaAgent = agents.find(a => a.id === activeCluster.alphaId);
        if (!alphaAgent) {
            EventBus.emit({
                source: 'System',
                text: i18n.t('missions.event_error_alpha_not_found'),
                severity: 'error',
                missionId: activeCluster.id
            });
            return;
        }

        EventBus.emit({
            source: 'System',
            text: i18n.t('missions.event_launching', { objective: activeCluster.objective }),
            severity: 'warning',
            missionId: activeCluster.id
        });

        // --- TRACING ACTIVATION ---
        // Generate a traceId for this mission run to enable the Lineage Stream
        const requestId = (typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : `tr-${Date.now()}`;
        const traceId = requestId.replace(/-/g, '').padEnd(32, '0').slice(0, 32);
        useTraceStore.getState().setActiveTrace(traceId);
        // --- END TRACING ---

        try {
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            const { modelId, provider } = resolveAgentModelConfig(alphaAgent as any);
            const success = await TadpoleOSService.sendCommand(
                alphaAgent.id,
                activeCluster.objective,
                modelId,
                provider,
                activeCluster.id,
                activeCluster.department as string,
                activeCluster.budgetUsd,
                undefined,
                undefined,
                activeCluster.analysisEnabled,
                requestId
            );
            if (success) {
                storeUpdateAgent(alphaAgent.id, { 
                    status: 'active', 
                    currentTask: activeCluster.objective 
                });
                EventBus.emit({ 
                    source: 'System', 
                    text: i18n.t('missions.event_dispatched', { name: alphaAgent.name }), 
                    severity: 'success',
                    missionId: activeCluster.id
                });
            } else {
                throw new Error("Engine rejected the command.");
            }
        } catch (err: unknown) {
            EventBus.emit({
                source: 'System',
                text: i18n.t('missions.event_launch_fail', { error: err instanceof Error ? err.message : String(err) }),
                severity: 'error',
                missionId: activeCluster.id
            });
        }
    };

    const activeProposals = useWorkspaceStore(state => state.activeProposals);
    const dismissProposal = useWorkspaceStore(state => state.dismissProposal);
    const applyProposal = useWorkspaceStore(state => state.applyProposal);

    return (
        <div className="h-full flex flex-col animate-in fade-in duration-500" aria-label="Missions Board">
            <h1 className="sr-only">Tadpole OS Swarm Missions Board</h1>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-6 flex-1 min-h-0">
                <ClusterSidebar
                    clusters={clusters}
                    selectedClusterId={selectedClusterId}
                    agents={agents}
                    onSelectCluster={setSelectedClusterId}
                    onCreateCluster={useWorkspaceStore.getState().createCluster}
                    onDeleteCluster={deleteCluster}
                    onToggleActive={toggleClusterActive}
                    onUpdateDepartment={updateClusterDepartment}
                    onUpdateBudget={updateClusterBudget}
                />

                <div className="md:col-span-2 bg-zinc-950 border border-zinc-800 rounded-2xl flex flex-col overflow-hidden">
                    {activeCluster ? (
                        <>
                            <MissionHeader
                                activeCluster={activeCluster}
                                agentsLoading={agentsLoading}
                                hasAgents={agents.length > 0}
                                onRunMission={handleRunMission}
                                onToggleAnalysis={toggleMissionAnalysis}
                            />

                            <div className="p-6 flex flex-col gap-8 flex-1 overflow-y-auto custom-scrollbar">
                                <NeuralMap
                                    cluster={activeCluster}
                                    agents={agents}
                                    themeColor={getThemeColors(activeCluster.theme).hex}
                                />

                                {/* Proposals */}
                                {activeProposals[activeCluster.id] && (
                                    <div className={`p-4 rounded-xl border ${getThemeColors(activeCluster.theme).bg} ${getThemeColors(activeCluster.theme).border} animate-in slide-in-from-top-4`}>
                                        <div className="flex justify-between items-start gap-4">
                                            <div className="flex gap-4">
                                                <div className={`p-2 rounded-lg bg-zinc-900 border ${getThemeColors(activeCluster.theme).border}`}>
                                                    <Cpu size={20} className={getThemeColors(activeCluster.theme).text} />
                                                </div>
                                                <div className="space-y-1">
                                                    <h4 className="text-xs font-bold text-zinc-100 uppercase tracking-tight">{i18n.t('missions.proposal_title')}</h4>
                                                    <p className="text-xs text-zinc-400 font-mono leading-relaxed max-w-xl">
                                                        {activeProposals[activeCluster.id].reasoning}
                                                    </p>
                                                </div>
                                            </div>
                                            <div className="flex gap-2">
                                                    <button
                                                        onClick={() => dismissProposal(activeCluster.id)}
                                                        className="px-3 py-1.5 rounded-lg border border-zinc-700 bg-zinc-900 text-zinc-500 text-xs font-bold uppercase transition-colors"
                                                    >
                                                        {i18n.t('missions.btn_dismiss')}
                                                    </button>
                                                <button
                                                    onClick={() => applyProposal(activeCluster.id)}
                                                    className={`px-3 py-1.5 rounded-lg border ${getThemeColors(activeCluster.theme).text} ${getThemeColors(activeCluster.theme).border} bg-zinc-900 text-xs font-bold uppercase shadow-lg ${getThemeColors(activeCluster.theme).glow}`}
                                                >
                                                    {i18n.t('missions.btn_authorize')}
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                )}

                                {/* Handoffs */}
                                {activeCluster.pendingTasks.filter(t => t.id.startsWith('ho-') && t.status === 'pending').length > 0 && (
                                    <div className="p-4 rounded-xl border border-amber-500/30 bg-amber-500/5">
                                        <h4 className="text-xs font-bold text-amber-400 uppercase tracking-widest mb-3 flex items-center gap-2">
                                            <Repeat size={12} /> {i18n.t('missions.header_handoffs')}
                                        </h4>
                                        <div className="space-y-3">
                                            {activeCluster.pendingTasks.filter(t => t.id.startsWith('ho-') && t.status === 'pending').map(task => (
                                                <div key={task.id} className="p-3 bg-black/40 border border-amber-500/20 rounded-lg flex items-center justify-between gap-4">
                                                    <div className="flex flex-col gap-1">
                                                        <p className="text-xs text-zinc-100">{task.description}</p>
                                                        <span className="text-[8px] text-amber-500/60 uppercase font-mono">{i18n.t('missions.label_delegation_request')}</span>
                                                    </div>
                                                    <div className="flex items-center gap-2">
                                                        <button
                                                            onClick={() => approveBranch(activeCluster.id, task.id)}
                                                            className="p-1.5 rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/30"
                                                        >
                                                            <Check size={14} />
                                                        </button>
                                                        <button
                                                            onClick={() => rejectBranch(activeCluster.id, task.id)}
                                                            className="p-1.5 rounded-lg bg-red-500/10 text-red-400 border border-red-500/30"
                                                        >
                                                            <X size={14} />
                                                        </button>
                                                    </div>
                                                </div>
                                            ))}
                                        </div>
                                    </div>
                                )}

                                <div key={activeCluster.id} className="p-4 rounded-xl bg-zinc-900/30 border border-zinc-800/50 flex flex-col gap-3">
                                    <h4 className="text-xs font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                                        <Target size={12} className={getThemeColors(activeCluster.theme).text} /> {i18n.t('missions.header_objective')}
                                    </h4>
                                    <textarea
                                        value={activeCluster.objective || ''}
                                        onChange={(e) => updateClusterObjective(activeCluster.id, e.target.value)}
                                        placeholder={i18n.t('missions.placeholder_objective')}
                                        className="bg-transparent text-sm text-zinc-300 border-none focus:ring-0 resize-none h-16 custom-scrollbar"
                                    />
                                </div>

                                <AgentTeamView
                                    activeCluster={activeCluster}
                                    agents={agents}
                                    availableAgents={availableAgents}
                                    onAssign={(id) => assignAgentToCluster(id, activeCluster.id)}
                                    onUnassign={(id) => unassignAgentFromCluster(id, activeCluster.id)}
                                    onSetAlpha={(id) => setAlphaNode(activeCluster.id, id)}
                                />

                                <div className="mt-4 h-64">
                                    <BufferedTranscriptView 
                                        missionId={activeCluster.id} 
                                        className="shadow-2xl border-zinc-800/50" 
                                    />
                                </div>
                            </div>
                        </>
                    ) : (
                        <TwEmptyState
                            icon={<Target size={48} />}
                            title={i18n.t('missions.empty_title')}
                            description={i18n.t('missions.empty_desc')}
                        />
                    )}
                </div>
            </div>
        </div>
    );
}


