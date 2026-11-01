/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / History / Memory_Palace_Viewer
 * - **Primary Entrypoints**: `Memory_Palace_Viewer`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Topic room cards, swarm findings, contributor avatars, JetBrains Mono tags.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React, { useState, useMemo } from 'react';
import { 
    Brain, 
    Search, 
    Bot, 
    Clock, 
    Copy, 
    Check, 
    FolderGit2, 
} from 'lucide-react';
import type { SwarmFindingRecord } from '../../../types/mission_dossier';
import { Tooltip } from '../../ui';

interface MemoryPalaceViewerProps {
    findings: SwarmFindingRecord[];
}

export const Memory_Palace_Viewer: React.FC<MemoryPalaceViewerProps> = ({ findings }) => {
    const [search_text, set_search_text] = useState('');
    const [selected_topic, set_selected_topic] = useState<string>('all');
    const [copied_id, set_copied_id] = useState<string | null>(null);

    const topics = useMemo(() => {
        const set = new Set<string>();
        findings.forEach((f) => {
            if (f.topic) set.add(f.topic);
        });
        return Array.from(set).sort();
    }, [findings]);

    // Defensive topic fallback: if selected topic is not in the current mission's findings, fall back to 'all'
    const effective_topic: string = selected_topic === 'all' || topics.includes(selected_topic) ? selected_topic : 'all';

    const filtered_findings = useMemo(() => {
        return findings.filter((f) => {
            if (effective_topic !== 'all' && f.topic !== effective_topic) {
                return false;
            }
            if (search_text.trim()) {
                const q = search_text.toLowerCase();
                const match_finding = f.finding.toLowerCase().includes(q);
                const match_topic = f.topic.toLowerCase().includes(q);
                const match_agent = f.agent_id.toLowerCase().includes(q);
                if (!match_finding && !match_topic && !match_agent) return false;
            }
            return true;
        });
    }, [findings, effective_topic, search_text]);

    const handle_copy = async (text: string, id: string) => {
        try {
            await navigator.clipboard.writeText(text);
            set_copied_id(id);
            setTimeout(() => set_copied_id(null), 2000);
        } catch (err) {
            console.warn('Failed to copy finding to clipboard:', err);
        }
    };

    if (findings.length === 0) {
        return (
            <div className="flex flex-col items-center justify-center h-full p-12 text-center text-zinc-500 bg-zinc-950 border border-zinc-800/80 rounded-xl">
                <Brain size={36} className="text-zinc-600 mb-3" />
                <h3 className="text-sm font-semibold text-zinc-300">Memory Palace Empty</h3>
                <p className="text-xs text-zinc-500 max-w-sm mt-1">
                    No contextual swarm findings or architectural insights were committed to the Memory Palace during this mission.
                </p>
            </div>
        );
    }

    return (
        <div className="flex flex-col h-full bg-zinc-950 border border-zinc-800/80 rounded-xl overflow-hidden">
            {/* Header / Filter Toolbar */}
            <div className="p-3 border-b border-zinc-800/80 bg-zinc-900/50 flex flex-wrap items-center justify-between gap-3">
                <div className="relative flex-1 min-w-[200px]">
                    <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500 pointer-events-none" />
                    <input
                        type="text"
                        placeholder="Search findings by topic, keyword, or agent..."
                        value={search_text}
                        onChange={(e) => set_search_text(e.target.value)}
                        className="w-full bg-[color:var(--color-background)] border border-zinc-800 rounded-lg pl-9 pr-3 py-1.5 text-xs text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-emerald-500/50 focus:ring-1 focus:ring-emerald-500/30 transition-all font-sans"
                    />
                </div>

                {/* Topic Pills */}
                <div className="flex items-center gap-1.5 overflow-x-auto custom-scrollbar">
                    <button
                        onClick={() => set_selected_topic('all')}
                        className={`px-2.5 py-1 rounded-md text-[11px] font-medium transition-colors ${
                            effective_topic === 'all'
                                ? 'bg-cyan-500/15 text-cyan-300 border border-cyan-500/30'
                                : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/50'
                        }`}
                    >
                        All Topics ({findings.length})
                    </button>
                    {topics.map((t) => (
                        <button
                            key={t}
                            onClick={() => set_selected_topic(t)}
                            className={`px-2.5 py-1 rounded-md text-[11px] font-medium transition-colors whitespace-nowrap ${
                                effective_topic === t
                                    ? 'bg-cyan-500/15 text-cyan-300 border border-cyan-500/30'
                                    : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/50'
                            }`}
                        >
                            {t}
                        </button>
                    ))}
                </div>
            </div>

            {/* Findings Grid */}
            <div className="flex-1 overflow-y-auto custom-scrollbar p-4 grid grid-cols-1 md:grid-cols-2 gap-3">
                {filtered_findings.map((item) => (
                    <div
                        key={item.id}
                        className="p-4 rounded-xl bg-zinc-900/40 border border-zinc-800/80 hover:border-zinc-700 transition-all flex flex-col justify-between gap-3"
                    >
                        <div className="flex flex-col gap-2">
                            <div className="flex items-center justify-between">
                                <Tooltip content={`Cognitive topic room: ${item.topic}`} position="top">
                                    <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider bg-purple-500/10 text-purple-300 border border-purple-500/20 cursor-help">
                                        <FolderGit2 size={11} />
                                        {item.topic}
                                    </span>
                                </Tooltip>
                                <div className="flex items-center gap-2 text-[11px] font-mono text-zinc-500">
                                    <span className="flex items-center gap-1">
                                        <Bot size={12} className="text-cyan-400" />
                                        {item.agent_id}
                                    </span>
                                    <Tooltip content={copied_id === item.id ? "Copied finding to clipboard!" : "Copy finding to clipboard"} position="left">
                                        <button
                                            onClick={() => handle_copy(item.finding, item.id)}
                                            className="p-1 hover:text-zinc-200 transition-colors"
                                            aria-label="Copy finding"
                                        >
                                            {copied_id === item.id ? (
                                                <Check size={12} className="text-emerald-400" />
                                            ) : (
                                                <Copy size={12} />
                                            )}
                                        </button>
                                    </Tooltip>
                                </div>
                            </div>

                            <p className="text-xs text-zinc-200 font-sans leading-relaxed whitespace-pre-wrap">
                                {item.finding}
                            </p>
                        </div>

                        <div className="flex items-center justify-between pt-2 border-t border-zinc-800/50 text-[10px] font-mono text-zinc-500">
                            <span className="flex items-center gap-1">
                                <Clock size={10} />
                                {new Date(item.timestamp).toLocaleString()}
                            </span>
                            <span className="text-zinc-600">ID: {item.id.slice(0, 8)}</span>
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
};
