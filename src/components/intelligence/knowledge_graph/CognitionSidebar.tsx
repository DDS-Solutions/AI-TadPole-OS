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
import { Info, Search, Target, RefreshCw, Cpu, Brain, Send, Trash2, ShieldAlert } from 'lucide-react';
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
                        <span className="text-[9px] font-bold text-zinc-500 uppercase tracking-wider">Blast Radius</span>
                        {blastRadiusLoading ? (
                            <span className="flex items-center gap-1 text-[9px] font-bold text-cyan-400 animate-pulse">
                                <RefreshCw size={8} className="animate-spin text-cyan-400 shrink-0" />
                                Analyzing...
                            </span>
                        ) : (
                            <span className="text-[9px] font-bold text-rose-500 bg-rose-500/10 px-1.5 py-0.5 rounded-md">+{Math.max(0, affected_nodes.size - 1)} dependents</span>
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
                    <Search size={12} className="text-cyan-400" /> <span className="text-[9px] font-bold uppercase tracking-widest">Explore</span>
                </button>
                <button className="flex items-center justify-center gap-2 px-3 py-2.5 bg-cyan-500 hover:bg-cyan-400 text-black rounded-xl transition-all group cursor-pointer">
                    <Target size={12} /> <span className="text-[9px] font-bold uppercase tracking-widest">Analyze</span>
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
                    <span>Security lock active. Swarm mutation actions disabled.</span>
                </div>
            )}
            <div className="flex flex-col gap-1.5">
                <label className="text-[8px] font-black text-zinc-500 uppercase tracking-widest flex items-center gap-1">
                    <Cpu size={10} className="text-zinc-400" /> Target Agent Node
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
                    <Brain size={10} className="text-zinc-400" /> Inject Cognition
                </label>
                <div className="flex gap-2">
                    <input
                        type="text"
                        value={new_memory_text}
                        onChange={(e) => setNew_memory_text(e.target.value)}
                        placeholder="Write new mental record..."
                        disabled={!has_write_permission}
                        className="flex-1 bg-zinc-950 border border-zinc-800 rounded-xl px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-cyan-500 font-mono disabled:opacity-40"
                        onKeyDown={(e) => e.key === 'Enter' && handle_inject_memory()}
                    />
                    <button
                        onClick={handle_inject_memory}
                        disabled={!new_memory_text || memory_loading || !has_write_permission}
                        className="px-3 bg-cyan-500 hover:bg-cyan-400 disabled:opacity-40 text-black rounded-xl transition-all flex items-center justify-center font-bold font-mono text-xs cursor-pointer"
                    >
                        <Send size={12} />
                    </button>
                </div>
            </div>
            <div className="flex flex-col gap-1.5">
                <label className="text-[8px] font-black text-zinc-500 uppercase tracking-widest flex items-center gap-1">
                    <Search size={10} className="text-zinc-400" /> Semantic Search
                </label>
                <div className="flex gap-2">
                    <input
                        type="text"
                        value={search_query}
                        onChange={(e) => set_search_query(e.target.value)}
                        placeholder="Query agent's memory space..."
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
                    <label className="text-[8px] font-black text-zinc-500 uppercase tracking-widest">Memory Vector Space</label>
                    {memory_loading && <span className="text-[8px] text-cyan-400 font-bold uppercase tracking-widest animate-pulse">Syncing...</span>}
                </div>
                <div className="max-h-48 overflow-y-auto space-y-2 pr-1 custom-scrollbar">
                    {memory_error ? (
                        <div className="text-[10px] text-rose-400 italic bg-rose-950/20 border border-rose-900/30 p-2 rounded-xl">{memory_error}</div>
                    ) : display_memories.length === 0 ? (
                        <div className="text-[10px] text-zinc-500 italic bg-zinc-950/40 p-3 rounded-xl border border-zinc-900 text-center font-mono">No cognitive records found.</div>
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
        setActiveInfoTab
    } = useUIStateContext();

    const workspace = useMemoryWorkspace(isMemoryNode, activeInfoTab);

    if (!selectedNode) return null;

    const total_nodes_count = data?.nodes.length || 0;

    return (
        <div className={`absolute bottom-6 left-6 ${isMemoryNode && activeInfoTab === 'memory' ? 'w-[400px]' : 'w-80'} bg-zinc-900/80 backdrop-blur-xl border border-zinc-800 p-5 rounded-2xl animate-in fade-in slide-in-from-bottom-4 duration-300 transition-all z-50 shadow-2xl`}>
            <div className="flex flex-col gap-4">
                <div className="flex items-start justify-between">
                    <div className="flex flex-col gap-1 min-w-0">
                        <span className="text-[8px] font-black text-cyan-400 uppercase tracking-[0.2em]">{get_kind_display(selectedNode.kind)}</span>
                        <h3 className="text-sm font-bold text-white truncate pr-2 font-mono" title={selectedNode.name}>{selectedNode.name}</h3>
                    </div>
                    <button onClick={resetSelection} className="text-zinc-500 hover:text-white transition-colors cursor-pointer shrink-0">
                        <RefreshCw size={14} />
                    </button>
                </div>
                {isMemoryNode && (
                    <div className="flex border-b border-zinc-850 pb-2">
                        <button onClick={() => setActiveInfoTab('info')} className={`flex-1 text-[10px] font-bold uppercase tracking-wider text-center py-1 transition-all cursor-pointer ${activeInfoTab === 'info' ? 'text-cyan-400 border-b-2 border-cyan-400 font-black' : 'text-zinc-500 hover:text-zinc-300 font-medium'}`}>Symbol Info</button>
                        <button onClick={() => setActiveInfoTab('memory')} className={`flex-1 text-[10px] font-bold uppercase tracking-wider text-center py-1 transition-all cursor-pointer ${activeInfoTab === 'memory' ? 'text-cyan-400 border-b-2 border-cyan-400 font-black' : 'text-zinc-500 hover:text-zinc-300 font-medium'}`}>Memory Workspace</button>
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
            </div>
        </div>
    );
};

// Metadata: [CognitionSidebar]
