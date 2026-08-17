/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:UI-Components**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[CognitionSidebar]` in observability traces.
 */

import React from 'react';
import { Info, Search, Target, RefreshCw, Cpu, Brain, Send, Trash2, ShieldAlert, Calendar, Tag, ExternalLink, ShieldCheck, Database, Globe, FileText, Activity, BookOpen } from 'lucide-react';
import { i18n } from '../../../i18n';
import type { ExtendedGraphNode } from './types';
import type { Agent } from '../../../types';
import { useMemoryWorkspace, type MemoryWorkspaceHookResult } from './useMemoryWorkspace';
import { 
    useSelectionContext, 
    useGraphDataContext, 
    useUIStateContext 
} from './graph_context_hooks';
import { get_kind_display } from './utils/graph_render_config';

const SymbolInfoSection = React.memo(({ 
    node, affected_nodes, total_nodes_count, blastRadiusLoading
}: { 
    node: ExtendedGraphNode; 
    affected_nodes: Set<string>; 
    total_nodes_count: number;
    blastRadiusLoading?: boolean;
}) => {
    const progress_percent = Math.min(100, (affected_nodes.size / (total_nodes_count || 1)) * 100);
    return (
        <div className="flex flex-col gap-4 font-mono">
            <div className="flex flex-col gap-2">
                <div className="flex items-center gap-2 bg-zinc-950/50 p-2.5 rounded-lg border border-zinc-800/50">
                    <Info size={12} className="text-zinc-500 shrink-0" />
                    <span className="text-[10px] text-zinc-400 font-mono truncate" title={node.path}>{node.path}</span>
                </div>
                <div className="mt-2 flex flex-col gap-2">
                    <div className="flex items-center justify-between">
                        <span className="text-[9px] font-bold text-zinc-500 uppercase tracking-wider">{i18n.t('knowledge_graph.sidebar_blast_radius')}</span>
                        {blastRadiusLoading ? (
                            <span className="flex items-center gap-1 text-[9px] font-bold text-cyan-400 animate-pulse">
                                <RefreshCw size={8} className="animate-spin text-cyan-400 shrink-0" />
                                {i18n.t('knowledge_graph.sidebar_analyzing')}
                            </span>
                        ) : (
                            <span className="text-[9px] font-bold text-rose-500 bg-rose-500/10 px-1.5 py-0.5 rounded-md">{i18n.t('knowledge_graph.sidebar_dependents', { count: Math.max(0, affected_nodes.size - 1) })}</span>
                        )}
                    </div>
                    <div className="w-full h-1 bg-zinc-800 rounded-full overflow-hidden">
                        <div 
                            className={`h-full transition-all duration-500 ${blastRadiusLoading ? 'bg-cyan-500 animate-pulse w-full' : 'bg-rose-500'}`} 
                            style={{ width: blastRadiusLoading ? undefined : `${progress_percent}%` }} 
                        />
                    </div>
                </div>
            </div>
            <div className="grid grid-cols-2 gap-2 mt-2">
                <button className="flex items-center justify-center gap-2 px-3 py-2.5 bg-zinc-800 hover:bg-zinc-700/80 text-white rounded-xl transition-all group cursor-pointer border border-zinc-700/30">
                    <Search size={12} className="text-cyan-400" /> <span className="text-[9px] font-bold uppercase tracking-widest">{i18n.t('knowledge_graph.sidebar_btn_explore')}</span>
                </button>
                <button className="flex items-center justify-center gap-2 px-3 py-2.5 bg-cyan-500 hover:bg-cyan-400 text-zinc-950 rounded-xl transition-all group cursor-pointer">
                    <Target size={12} /> <span className="text-[9px] font-bold uppercase tracking-widest">{i18n.t('knowledge_graph.sidebar_btn_analyze')}</span>
                </button>
            </div>
        </div>
    );
});

const MemoryWorkspaceSection = ({ workspace }: { workspace: MemoryWorkspaceHookResult }) => {
    const {
        agents, selected_agent_id, set_selected_agent_id, search_query, set_search_query,
        set_search_results, is_searching, new_memory_text, setNew_memory_text,
        display_memories, memory_loading, memory_error, has_write_permission,
        handle_inject_memory, handle_search, handle_delete_memory
    } = workspace;

    return (
        <div className="flex flex-col gap-4 font-mono">
            {!has_write_permission && (
                <div className="flex items-center gap-2 text-[9px] text-amber-500/90 bg-amber-500/5 border border-amber-500/15 p-2.5 rounded-xl">
                    <ShieldAlert size={12} className="shrink-0 text-amber-500 animate-pulse" />
                    <span>{i18n.t('knowledge_graph.sidebar_security_lock')}</span>
                </div>
            )}
            <div className="flex flex-col gap-1.5">
                <label className="text-[8px] font-black text-zinc-500 uppercase tracking-widest flex items-center gap-1">
                    <Cpu size={10} className="text-zinc-400" /> {i18n.t('knowledge_graph.sidebar_label_target_node')}
                </label>
                <select
                    value={selected_agent_id}
                    onChange={(e) => { 
                        set_selected_agent_id(e.target.value); 
                        set_search_results(null); 
                        set_search_query(''); 
                    }}
                    className="bg-zinc-950 border border-zinc-800 rounded-xl px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-cyan-500 font-mono w-full cursor-pointer"
                >
                    {(agents || []).map((agent: Agent) => <option key={agent.id} value={agent.id}>{agent.name} [{agent.role}]</option>)}
                </select>
            </div>
            <div className="flex flex-col gap-1.5">
                <label className="text-[8px] font-black text-zinc-500 uppercase tracking-widest flex items-center gap-1">
                    <Brain size={10} className="text-zinc-400" /> {i18n.t('knowledge_graph.sidebar_label_inject')}
                </label>
                <div className="flex gap-2">
                    <input
                        type="text"
                        value={new_memory_text}
                        onChange={(e) => setNew_memory_text(e.target.value)}
                        placeholder={i18n.t('knowledge_graph.sidebar_placeholder_inject')}
                        disabled={!has_write_permission}
                        className="flex-1 bg-zinc-950 border border-zinc-800 rounded-xl px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-cyan-500 font-mono disabled:opacity-40"
                        onKeyDown={(e) => e.key === 'Enter' && handle_inject_memory()}
                    />
                    <button
                        onClick={handle_inject_memory}
                        aria-label={i18n.t('knowledge_graph.sidebar_btn_inject', { defaultValue: 'Inject memory' })}
                        disabled={!new_memory_text || memory_loading || !has_write_permission}
                        className="px-3 bg-cyan-500 hover:bg-cyan-400 disabled:opacity-40 text-zinc-950 rounded-xl transition-all flex items-center justify-center font-bold font-mono text-xs cursor-pointer"
                    >
                        <Send size={12} />
                    </button>
                </div>
            </div>
            <div className="flex flex-col gap-1.5">
                <label className="text-[8px] font-black text-zinc-500 uppercase tracking-widest flex items-center gap-1">
                    <Search size={10} className="text-zinc-400" /> {i18n.t('knowledge_graph.sidebar_label_search')}
                </label>
                <div className="flex gap-2">
                    <input
                        type="text"
                        value={search_query}
                        onChange={(e) => set_search_query(e.target.value)}
                        placeholder={i18n.t('knowledge_graph.sidebar_placeholder_search')}
                        className="flex-1 bg-zinc-950 border border-zinc-800 rounded-xl px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-cyan-500 font-mono"
                        onKeyDown={(e) => e.key === 'Enter' && handle_search()}
                    />
                    <button
                        onClick={handle_search}
                        disabled={is_searching}
                        className="px-3 bg-zinc-850 hover:bg-zinc-750 text-zinc-300 hover:text-white border border-zinc-800 rounded-xl transition-all flex items-center justify-center cursor-pointer shrink-0"
                    >
                        {is_searching ? <RefreshCw size={12} className="animate-spin" /> : <Search size={12} />}
                    </button>
                </div>
            </div>
            <div className="flex flex-col gap-2">
                <div className="flex justify-between items-center border-b border-zinc-850 pb-1">
                    <label className="text-[8px] font-black text-zinc-500 uppercase tracking-widest">{i18n.t('knowledge_graph.sidebar_label_vector_space')}</label>
                    {memory_loading && <span className="text-[8px] text-cyan-400 font-bold uppercase tracking-widest animate-pulse">{i18n.t('knowledge_graph.sidebar_status_syncing')}</span>}
                </div>
                <div className="max-h-48 overflow-y-auto space-y-2 pr-1 custom-scrollbar">
                    {memory_error ? (
                        <div className="text-[10px] text-rose-400 italic bg-rose-950/20 border border-rose-900/30 p-2 rounded-xl">{memory_error}</div>
                    ) : display_memories.length === 0 ? (
                        <div className="text-[10px] text-zinc-500 italic bg-zinc-950/40 p-3 rounded-xl border border-zinc-900 text-center font-mono">{i18n.t('knowledge_graph.sidebar_no_records')}</div>
                    ) : (
                        display_memories.map((m) => (
                            <div key={m.id} className="group/mem p-2.5 bg-zinc-950/60 hover:bg-zinc-950 border border-zinc-800 hover:border-zinc-700/80 rounded-xl transition-all flex items-start justify-between gap-3">
                                <div className="flex flex-col gap-1 min-w-0">
                                    <p className="text-[10px] text-zinc-300 leading-relaxed font-mono break-words">{m.text}</p>
                                </div>
                                {has_write_permission && (
                                    <button onClick={() => handle_delete_memory(m.id)} className="p-1 text-zinc-600 hover:text-rose-400 opacity-0 group-hover/mem:opacity-100 cursor-pointer">
                                        <Trash2 size={10} />
                                    </button>
                                )}
                            </div>
                        ))
                    )}
                </div>
            </div>
        </div>
    );
};

const OKFInfoSection = ({ node }: { node: ExtendedGraphNode }) => {
    const getConceptIcon = (type?: string) => {
        const ct = (type || '').toLowerCase();
        if (ct.includes('bigquery') || ct.includes('table') || ct.includes('dataset') || ct.includes('db') || ct.includes('database')) {
            return <Database size={12} className="text-zinc-400" />;
        }
        if (ct.includes('api') || ct.includes('endpoint') || ct.includes('service') || ct.includes('route') || ct.includes('globe') || ct.includes('network')) {
            return <Globe size={12} className="text-zinc-400" />;
        }
        if (ct.includes('playbook') || ct.includes('sop') || ct.includes('file') || ct.includes('document') || ct.includes('doc') || ct.includes('text')) {
            return <FileText size={12} className="text-zinc-400" />;
        }
        if (ct.includes('metric') || ct.includes('kpi') || ct.includes('activity') || ct.includes('pulse') || ct.includes('telemetry')) {
            return <Activity size={12} className="text-zinc-400" />;
        }
        return <BookOpen size={12} className="text-zinc-400" />;
    };

    return (
        <div className="flex flex-col gap-4 font-mono text-zinc-350">
            {node.description && (
                <div className="text-[10px] leading-relaxed text-zinc-400 border-b border-zinc-850 pb-3 font-sans">
                    {node.description}
                </div>
            )}

            <div className="flex flex-col gap-2.5 text-[10px]">
                <div className="flex items-center justify-between py-1 border-b border-zinc-850/50">
                    <span className="text-zinc-500 uppercase text-[9px] font-bold">{i18n.t('knowledge_graph.sidebar_label_topic')}</span>
                    <span className="text-zinc-300 font-bold truncate max-w-[200px]" title={node.path}>{node.path}</span>
                </div>

                <div className="flex items-center justify-between py-1 border-b border-zinc-850/50">
                    <span className="text-zinc-500 uppercase text-[9px] font-bold">{i18n.t('knowledge_graph.sidebar_label_concept_type')}</span>
                    <div className="flex items-center gap-1.5 bg-zinc-950/40 px-2 py-0.5 rounded border border-zinc-850">
                        {getConceptIcon(node.concept_type)}
                        <span className="text-zinc-350 font-bold capitalize">{node.concept_type || 'general'}</span>
                    </div>
                </div>

                <div className="flex items-center justify-between py-1 border-b border-zinc-850/50">
                    <span className="text-zinc-500 uppercase text-[9px] font-bold">{i18n.t('knowledge_graph.sidebar_label_confidence')}</span>
                    <div className="flex items-center gap-2">
                        <div className="w-16 h-1.5 bg-zinc-850 rounded-full overflow-hidden">
                            <div className="h-full bg-cyan-500" style={{ width: `${(node.confidence ?? 0) * 100}%` }} />
                        </div>
                        <span className="font-bold text-[9px]">{Math.round((node.confidence ?? 0) * 100)}%</span>
                    </div>
                </div>

                <div className="flex items-center justify-between py-1 border-b border-zinc-850/50">
                    <span className="text-zinc-500 uppercase text-[9px] font-bold">{i18n.t('knowledge_graph.sidebar_label_governance')}</span>
                    {node.human_confirmed ? (
                        <div className="flex items-center gap-1 text-[9px] font-bold text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-md border border-emerald-500/20">
                            <ShieldCheck size={10} />
                            <span>{i18n.t('knowledge_graph.sidebar_state_confirmed')}</span>
                        </div>
                    ) : (
                        <div className="flex items-center gap-1 text-[9px] font-bold text-amber-500 bg-amber-500/10 px-2 py-0.5 rounded-md border border-amber-500/20">
                            <Calendar size={10} />
                            <span>{i18n.t('knowledge_graph.sidebar_state_pending')}</span>
                        </div>
                    )}
                </div>

                {node.resource_uri && (
                    <div className="flex flex-col gap-1 py-1 border-b border-zinc-850/50">
                        <span className="text-zinc-500 uppercase text-[9px] font-bold mb-1">{i18n.t('knowledge_graph.sidebar_label_resource')}</span>
                        <a 
                            href={node.resource_uri} 
                            target="_blank" 
                            rel="noopener noreferrer"
                            className="flex items-center justify-between px-2.5 py-1.5 bg-zinc-950/60 hover:bg-zinc-950 border border-zinc-850 hover:border-zinc-700/80 rounded-lg text-cyan-400 hover:text-cyan-300 transition-all text-[9px] truncate"
                        >
                            <span className="truncate">{node.resource_uri}</span>
                            <ExternalLink size={10} className="shrink-0 ml-1.5" />
                        </a>
                    </div>
                )}

                {node.tags && (
                    <div className="flex flex-col gap-1.5 py-1">
                        <span className="text-zinc-500 uppercase text-[9px] font-bold">{i18n.t('knowledge_graph.sidebar_label_tags')}</span>
                        <div className="flex flex-wrap gap-1">
                            {node.tags.split(',').map((tag, idx) => (
                                <span key={idx} className="flex items-center gap-1 text-[9px] font-bold text-zinc-400 bg-zinc-950/50 border border-zinc-850 px-2 py-0.5 rounded">
                                    <Tag size={8} className="text-zinc-500" />
                                    {tag.trim()}
                                </span>
                            ))}
                        </div>
                    </div>
                )}
            </div>

            {node.text && (
                <div className="flex flex-col gap-1.5 bg-zinc-950/50 border border-zinc-850/50 p-3 rounded-xl max-h-48 overflow-y-auto custom-scrollbar">
                    <span className="text-zinc-500 uppercase text-[8px] font-black tracking-widest mb-1">{i18n.t('knowledge_graph.sidebar_label_cognition_content')}</span>
                    <p className="text-[10px] leading-relaxed whitespace-pre-wrap break-words text-zinc-400 font-sans">{node.text}</p>
                </div>
            )}
        </div>
    );
};

export const CognitionSidebar: React.FC = () => {
    const {
        selectedNode,
        affectedNodes,
        resetSelection,
        blastRadiusLoading
    } = useSelectionContext();
    const { data } = useGraphDataContext();
    const {
        isMemoryNode,
        activeInfoTab,
        setActiveInfoTab,
        viewMode
    } = useUIStateContext();

    const workspace = useMemoryWorkspace(isMemoryNode, activeInfoTab);

    if (!selectedNode) return null;

    const total_nodes_count = data?.nodes.length || 0;
    const isOkfMode = viewMode === 'okf';
    const widthClass = isOkfMode 
        ? 'w-[360px]' 
        : (isMemoryNode && activeInfoTab === 'memory' ? 'w-[400px]' : 'w-80');

    return (
        <div className={`absolute bottom-6 left-6 ${widthClass} bg-zinc-900/80 backdrop-blur-xl border border-zinc-800 p-5 rounded-2xl animate-in fade-in slide-in-from-bottom-4 duration-300 transition-all z-50 shadow-2xl`}>
            <div className="flex flex-col gap-4">
                <div className="flex items-start justify-between">
                    <div className="flex flex-col gap-1 min-w-0">
                        <span className="text-[8px] font-black text-cyan-400 uppercase tracking-[0.2em]">
                            {isOkfMode ? (selectedNode.concept_type || 'concept') : get_kind_display(selectedNode.kind)}
                        </span>
                        <h3 className="text-sm font-bold text-white truncate pr-2 font-mono" title={selectedNode.title || selectedNode.name}>
                            {isOkfMode ? (selectedNode.title || selectedNode.name) : selectedNode.name}
                        </h3>
                    </div>
                    <button onClick={resetSelection} aria-label={i18n.t('common.reset', { defaultValue: 'Reset selection' })} className="text-zinc-500 hover:text-white transition-colors cursor-pointer shrink-0">
                        <RefreshCw size={14} />
                    </button>
                </div>

                {isOkfMode ? (
                    <OKFInfoSection node={selectedNode} />
                ) : (
                    <>
                        {isMemoryNode && (
                            <div className="flex border-b border-zinc-850 pb-2">
                                <button onClick={() => setActiveInfoTab('info')} className={`flex-1 text-[10px] font-bold uppercase tracking-wider text-center py-1 transition-all cursor-pointer ${activeInfoTab === 'info' ? 'text-cyan-400 border-b-2 border-cyan-400 font-black' : 'text-zinc-500 hover:text-zinc-300 font-medium'}`}>{i18n.t('knowledge_graph.sidebar_tab_info')}</button>
                                <button onClick={() => setActiveInfoTab('memory')} className={`flex-1 text-[10px] font-bold uppercase tracking-wider text-center py-1 transition-all cursor-pointer ${activeInfoTab === 'memory' ? 'text-cyan-400 border-b-2 border-cyan-400 font-black' : 'text-zinc-500 hover:text-zinc-300 font-medium'}`}>{i18n.t('knowledge_graph.sidebar_tab_memory')}</button>
                            </div>
                        )}
                        {(!isMemoryNode || activeInfoTab === 'info') ? (
                            <SymbolInfoSection 
                                node={selectedNode} 
                                affected_nodes={affectedNodes} 
                                total_nodes_count={total_nodes_count} 
                                blastRadiusLoading={blastRadiusLoading ?? false}
                            />
                        ) : (
                            <MemoryWorkspaceSection workspace={workspace} />
                        )}
                    </>
                )}
            </div>
        </div>
    );
};

// Metadata: [CognitionSidebar]
