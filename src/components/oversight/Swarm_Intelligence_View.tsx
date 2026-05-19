/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Swarm_Intelligence_View]` in observability traces.
 */

import React from 'react';
import { Cpu, Activity, Plus } from 'lucide-react';
import { Tooltip, Tw_Empty_State } from '../ui';
import { i18n } from '../../i18n';
import type { Mission_Cluster } from '../../stores/workspace_store';

interface IntelligenceProposalChange {
    agent_id: string;
    proposed_role?: string;
    proposed_model?: string;
    added_skills?: string[];
}

interface IntelligenceProposal {
    cluster_id: string;
    timestamp: string | number;
    reasoning: string;
    changes?: IntelligenceProposalChange[];
}

interface SwarmIntelligenceViewProps {
    active_proposals: Record<string, IntelligenceProposal>;
    clusters: Mission_Cluster[];
    resolve_agent_name: (id: string) => string;
}

export const Swarm_Intelligence_View: React.FC<SwarmIntelligenceViewProps> = ({
    active_proposals,
    clusters,
    resolve_agent_name
}) => {
    const proposals_list = Object.values(active_proposals || {});

    return (
        <div className="bg-[color:var(--color-surface)] border border-green-500/30 rounded-lg overflow-hidden">
            <div className="bg-green-500/10 p-3 border-b border-green-500/20 flex items-center gap-2">
                <Tooltip content={i18n.t('oversight.swarm_intel_tooltip')} position="right" class_name="w-full">
                    <Cpu className="w-4 h-4 text-green-400 cursor-help" />
                </Tooltip>
                <h2 className="font-semibold text-blue-100">{i18n.t('oversight.swarm_intel_title')}</h2>
            </div>
            <div className="p-6">
                {proposals_list.length > 0 ? (
                    <div className="grid grid-cols-1 gap-6">
                        {proposals_list.map((proposal: IntelligenceProposal) => {
                            const cluster = (clusters || []).find(c => c.id === proposal.cluster_id);
                            return (
                                <div key={proposal.cluster_id} className="bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-xl p-4 space-y-4">
                                    <div className="flex justify-between items-center">
                                        <div className="flex items-center gap-2">
                                            <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
                                            <span className="text-xs font-bold text-zinc-300 uppercase tracking-widest">{cluster?.name || i18n.t('oversight.unknown_cluster')}</span>
                                        </div>
                                        <span className="text-[10px] text-zinc-600 font-mono">
                                            {i18n.t('oversight.alpha_node_prefix', { alpha_id: cluster?.alpha_id ?? '?' })} • {new Date(proposal.timestamp).toLocaleTimeString()}
                                        </span>
                                    </div>

                                    <div className="bg-black/40 p-3 rounded-lg border border-[color:var(--color-border)]/50">
                                        <div className="text-[9px] font-bold text-zinc-500 mb-2 uppercase tracking-wide flex items-center gap-2">
                                            <Activity size={10} /> {i18n.t('oversight.neural_trace_label')}
                                        </div>
                                        <p className="text-xs text-zinc-400 leading-relaxed font-mono whitespace-pre-wrap">
                                            {proposal.reasoning}
                                        </p>
                                    </div>

                                    <div className="space-y-2">
                                        <div className="text-[9px] font-bold text-zinc-500 uppercase tracking-wide">{i18n.t('oversight.proposed_reallocations_label')}</div>
                                        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
                                            {(proposal.changes || []).map((change: IntelligenceProposalChange) => {
                                                return (
                                                    <div key={change.agent_id} className="p-3 bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded-xl flex flex-col gap-2 shadow-sm transition-all hover:border-emerald-500/20">
                                                        <div className="flex justify-between items-start">
                                                            <div className="flex items-center gap-2">
                                                                <div className="w-5 h-5 rounded bg-zinc-800 flex items-center justify-center text-[9px] font-bold text-zinc-500 border border-zinc-700">
                                                                    {resolve_agent_name(change.agent_id).charAt(0)}
                                                                </div>
                                                                <span className="text-[10px] font-bold text-zinc-200">{resolve_agent_name(change.agent_id)}</span>
                                                            </div>
                                                            <span className="px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-500 text-[7px] font-bold uppercase tracking-widest border border-emerald-500/20">
                                                                {i18n.t('oversight.mod_req_label')}
                                                            </span>
                                                        </div>
                                                        <div className="space-y-1">
                                                            {change.proposed_role && (
                                                                <div className="flex items-center gap-2 text-[9px]">
                                                                    <span className="text-zinc-600 uppercase">{i18n.t('oversight.role_label')}</span>
                                                                    <span className="text-green-400">{change.proposed_role}</span>
                                                                </div>
                                                            )}
                                                            {change.proposed_model && (
                                                                <div className="flex items-center gap-2 text-[9px]">
                                                                    <span className="text-zinc-600 uppercase">{i18n.t('oversight.model_label')}</span>
                                                                    <span className="text-green-400">{change.proposed_model}</span>
                                                                </div>
                                                            )}
                                                            {change.added_skills && (
                                                                <div className="flex items-center gap-2 text-[9px]">
                                                                    <span className="text-zinc-600 uppercase">{i18n.t('oversight.skills_label')}</span>
                                                                    <span className="text-emerald-400">+{change.added_skills.length}</span>
                                                                </div>
                                                            )}
                                                        </div>
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                ) : (
                    <Tw_Empty_State
                        icon={<Plus size={32} />}
                        title={i18n.t('oversight.no_optimization_traces')}
                    />
                )}
            </div>
        </div>
    );
};

// Metadata: [Swarm_Intelligence_View]
