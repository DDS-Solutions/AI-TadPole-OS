import { Activity, Cpu, ArrowRight, X, Trash2 } from 'lucide-react';
import { use_workspace_store } from '../stores/workspace_store';
import { agents as allAgents } from '../data/mock_agents';
import { i18n } from '../i18n';
import { Tooltip } from './ui';

interface Swarm_Oversight_Node_Props {
    cluster_id?: string;
    on_close?: () => void;
    class_name?: string;
}

export const Swarm_Oversight_Node: React.FC<Swarm_Oversight_Node_Props> = ({ cluster_id, on_close, class_name = '' }) => {
    const { active_proposals, clusters, dismiss_proposal } = use_workspace_store();
    
    // Filter proposals if cluster_id is provided, otherwise show all
    const proposals = cluster_id 
        ? (active_proposals[cluster_id] ? [active_proposals[cluster_id]] : []) 
        : Object.values(active_proposals);

    if (proposals.length === 0) return null;

    return (
        <div className={`flex flex-col gap-4 w-[400px] ${class_name}`}>
            {proposals.map((proposal) => {
                const cluster = clusters.find(c => c.id === proposal.cluster_id);
                return (
                    <div key={proposal.cluster_id} className="relative group">
                        {/* Connecting Line to Parent Node */}
                        <div className="absolute -top-6 left-8 w-px h-6 bg-blue-500/30 group-hover:bg-blue-500/60 transition-colors"></div>
                        <div className="absolute -top-6 left-8 -ml-[3px] w-[7px] h-[7px] rounded-full bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.5)]"></div>

                        <div className="bg-zinc-950 border border-blue-500/30 rounded-xl overflow-hidden shadow-2xl relative z-10">
                            {/* Header */}
                            <div className="bg-blue-500/10 px-4 py-2 border-b border-blue-500/20 flex items-center justify-between">
                                <div className="flex items-center gap-2">
                                    <Cpu size={14} className="text-blue-400" />
                                    <span className="text-[10px] font-bold text-blue-100 uppercase tracking-widest">
                                        {i18n.t('oversight.swarm_prefix')} • {cluster?.name || 'Cluster'}
                                    </span>
                                </div>
                                <div className="flex items-center gap-1.5">
                                    <div className="w-1.5 h-1.5 rounded-full bg-blue-500 animate-pulse" />
                                    <span className="text-[9px] font-mono text-blue-300/70">
                                        {new Date(proposal.timestamp).toLocaleTimeString()}
                                    </span>
                                    {on_close && (
                                        <button 
                                            onClick={(e) => { e.stopPropagation(); on_close(); }}
                                            aria-label={i18n.t('common.dismiss')}
                                            className="ml-2 p-0.5 rounded-full hover:bg-white/10 text-blue-300/50 hover:text-white transition-colors"
                                        >
                                            <X size={12} />
                                        </button>
                                    )}
                                </div>
                            </div>

                            <div className="p-4 space-y-4">
                                {/* Reasoning Trace */}
                                <div className="bg-black/40 p-3 rounded-lg border border-zinc-800/50 relative overflow-hidden">
                                    <div className="absolute top-0 left-0 w-1 h-full bg-blue-500/20" />
                                    <div className="text-[9px] font-bold text-zinc-500 mb-1.5 uppercase tracking-wide flex items-center gap-2">
                                        <Activity size={10} className="text-blue-500" /> {i18n.t('oversight.reasoning_trace')}
                                    </div>
                                    <p className="text-[10px] text-zinc-300 leading-relaxed font-mono whitespace-pre-wrap opacity-90">
                                        {proposal.reasoning}
                                    </p>
                                </div>

                                {/* Proposed Changes */}
                                <div>
                                    <div className="text-[9px] font-bold text-zinc-500 uppercase tracking-widest mb-2 flex items-center gap-2">
                                        <ArrowRight size={10} /> {i18n.t('oversight.optimization_strategy')}
                                    </div>
                                    <div className="space-y-2">
                                        {proposal.changes.map((change: any) => {
                                            const agent = allAgents.find(a => a.id === change.agent_id);
                                            return (
                                                <div key={change.agent_id} className="p-2 bg-zinc-900/50 border border-zinc-800 rounded mx-1 hover:border-zinc-700 transition-colors">
                                                    <div className="flex justify-between items-start mb-1">
                                                        <span className="text-[10px] font-bold text-zinc-200">{agent?.name || change.agent_id}</span>
                                                        <span className="text-[8px] px-1 rounded bg-blue-900/20 text-blue-300 border border-blue-500/20 font-mono uppercase">{i18n.t('oversight.modified')}</span>
                                                    </div>
                                                    <div className="grid grid-cols-2 gap-x-4 gap-y-1">
                                                        {change.proposed_role && (
                                                            <div className="flex items-center gap-1.5 text-[9px]">
                                                                <span className="text-zinc-500">{i18n.t('oversight.role_label')}</span>
                                                                <span className="text-blue-400 font-mono">{change.proposed_role}</span>
                                                            </div>
                                                        )}
                                                        {change.proposed_model && (
                                                            <div className="flex items-center gap-1.5 text-[9px]">
                                                                <span className="text-zinc-500">{i18n.t('oversight.model_label')}</span>
                                                                <span className="text-blue-400 font-mono">{change.proposed_model}</span>
                                                            </div>
                                                        )}
                                                        {change.added_skills && (
                                                            <div className="flex items-center gap-1.5 text-[9px] col-span-2">
                                                                <span className="text-zinc-500">{i18n.t('oversight.skills_label')}</span>
                                                                <span className="text-emerald-400 font-mono">+{change.added_skills.length} {i18n.t('oversight.new_skills')}</span>
                                                            </div>
                                                        )}
                                                    </div>
                                                </div>
                                            );
                                        })}
                                    </div>
                                </div>
                            </div>

                            {/* Actions Footer */}
                            <div className="px-4 pb-4 flex justify-end gap-2">
                                <Tooltip content={i18n.t('oversight.dismiss_tooltip')} position="top">
                                    <button 
                                        onClick={(e) => { 
                                            e.stopPropagation(); 
                                            dismiss_proposal(proposal.cluster_id);
                                            on_close?.();
                                        }}
                                        aria-label={i18n.t('oversight.btn_dismiss')}
                                        className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-red-500/20 bg-red-500/5 text-[10px] font-bold text-red-400 hover:bg-red-500/10 transition-colors uppercase tracking-widest"
                                    >
                                        <Trash2 size={12} />
                                        {i18n.t('oversight.btn_dismiss')}
                                    </button>
                                </Tooltip>
                                <button 
                                    onClick={(e) => { 
                                        e.stopPropagation(); 
                                        // In a real app, this would iterate and apply each proposed change
                                        use_workspace_store.getState().apply_proposal(proposal.cluster_id);
                                        on_close?.();
                                    }}
                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-blue-500/30 bg-blue-500/20 text-[10px] font-bold text-blue-100 hover:bg-blue-500/30 transition-colors uppercase tracking-widest shadow-[0_0_15px_rgba(59,130,246,0.2)]"
                                >
                                    <Activity size={12} />
                                    {i18n.t('missions.btn_authorize')}
                                </button>
                            </div>

                            {/* Footer Decorator */}
                            <div className="h-0.5 w-full bg-gradient-to-r from-transparent via-blue-500/20 to-transparent" />
                        </div>
                    </div>
                );
            })}
        </div>
    );
};
