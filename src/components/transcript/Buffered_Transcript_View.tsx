import React, { useState, useEffect, useRef, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Terminal, Brain, Wrench, AlertCircle, CheckCircle2, Search } from 'lucide-react';
import { event_bus, type log_entry } from '../../services/event_bus';
import { i18n } from '../../i18n';
import clsx from 'clsx';

interface Buffered_Transcript_View_Props {
    agent_id?: string;
    mission_id?: string;
    class_name?: string;
}

/**
 * High-performance, de-duplicated transcript rendering engine.
 * 
 * Features:
 * - ID-based de-duplication (guards against broadcast bridge overlaps)
 * - Structured block categorization (thinking, tool, stdout, error)
 * - RAF-batching for high-frequency streams
 * - Partial-line buffering for streaming consistency
 */
export const Buffered_Transcript_View: React.FC<Buffered_Transcript_View_Props> = ({ 
    agent_id, 
    mission_id,
    class_name 
}) => {
    const [entries, set_entries] = useState<log_entry[]>([]);
    const [filter, set_filter] = useState('');
    const scroll_ref = useRef<HTMLDivElement>(null);
    const seen_ids = useRef<Set<string>>(new Set());
    const buffer_ref = useRef<log_entry[]>([]);

    useEffect(() => {
        // subscribe to the global event bus
        const unsubscribe = event_bus.subscribe_logs((entry) => {
            // Filter by agent or mission if provided
            if (agent_id && entry.agent_id !== agent_id) return;
            if (mission_id && entry.mission_id !== mission_id) return;
            
            // De-duplicate using backend-provided IDs
            if (entry.id && seen_ids.current.has(entry.id)) return;
            if (entry.id) seen_ids.current.add(entry.id);

            buffer_ref.current.push(entry);
        });

        // High-frequency batching via RAF
        let raf_id: number;
        const flush = () => {
            if (buffer_ref.current.length > 0) {
                const batch = [...buffer_ref.current];
                buffer_ref.current = [];
                set_entries(prev => {
                    const next = [...prev, ...batch];
                    return next.slice(-1000); // hard cap for performance
                });
            }
            raf_id = requestAnimationFrame(flush);
        };
        raf_id = requestAnimationFrame(flush);

        return () => {
            unsubscribe();
            cancelAnimationFrame(raf_id);
        };
    }, [agent_id, mission_id]);

    // Auto-scroll
    useEffect(() => {
        if (scroll_ref.current) {
            scroll_ref.current.scrollTop = scroll_ref.current.scrollHeight;
        }
    }, [entries]);

    const filtered_entries = useMemo(() => {
        if (!filter) return entries;
        const lower = filter.toLowerCase();
        return entries.filter(e => e.text.toLowerCase().includes(lower) || e.source.toLowerCase().includes(lower));
    }, [entries, filter]);

    const render_entry = (entry: log_entry) => {
        const is_thought = entry.metadata?.type === 'thought' || entry.text.startsWith('Thinking:');
        const is_tool = entry.metadata?.type === 'tool' || entry.text.includes('Executing tool:');
        
        let icon = <Terminal size={12} />;
        let color = 'text-zinc-400';
        let bg = 'bg-zinc-950/20';

        if (is_thought) {
            icon = <Brain size={12} className="text-purple-400" />;
            color = 'text-purple-300';
            bg = 'bg-purple-500/5';
        } else if (is_tool) {
            icon = <Wrench size={12} className="text-blue-400" />;
            color = 'text-blue-300';
            bg = 'bg-blue-500/5';
        } else if (entry.severity === 'error') {
            icon = <AlertCircle size={12} className="text-red-400" />;
            color = 'text-red-300';
            bg = 'bg-red-500/5';
        } else if (entry.severity === 'success') {
            icon = <CheckCircle2 size={12} className="text-emerald-400" />;
            color = 'text-emerald-300';
            bg = 'bg-emerald-500/5';
        }

        return (
            <motion.div 
                key={entry.id || `${entry.timestamp.getTime()}-${entry.text.slice(0, 10)}`}
                initial={{ opacity: 0, x: -5 }}
                animate={{ opacity: 1, x: 0 }}
                className={clsx(
                    "flex flex-col gap-1 p-2 border-l-2 transition-colors group mb-1",
                    bg,
                    is_thought ? "border-purple-500/50" : 
                    is_tool ? "border-blue-500/50" : 
                    entry.severity === 'error' ? "border-red-500/50" : 
                    entry.severity === 'success' ? "border-emerald-500/50" : "border-zinc-800"
                )}
            >
                <div className="flex items-center gap-2 opacity-50 group-hover:opacity-100 transition-opacity">
                    {icon}
                    <span className="text-[10px] font-mono uppercase tracking-widest">{entry.source}</span>
                    <span className="text-[9px] font-mono text-zinc-600 ml-auto">
                        {entry.timestamp.toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                    </span>
                </div>
                <div className={clsx("text-xs leading-relaxed break-words font-mono", color)}>
                    {entry.text}
                </div>
            </motion.div>
        );
    };

    return (
        <div className={clsx("flex flex-col h-full bg-black/40 border border-zinc-800 rounded-xl overflow-hidden backdrop-blur-md", class_name)}>
            <div className="p-3 border-b border-zinc-900 bg-zinc-950/40 flex items-center justify-between gap-4">
                <div className="flex items-center gap-2">
                    <Terminal size={14} className="text-zinc-500" />
                    <span className="text-[10px] font-bold uppercase tracking-widest text-zinc-400">{i18n.t('transcript.run_log')}</span>
                </div>
                
                <div className="relative flex-1 max-w-xs">
                    <Search className="absolute left-2 top-1/2 -translate-y-1/2 text-zinc-600" size={12} />
                    <input 
                        type="text" 
                        value={filter}
                        onChange={(e) => set_filter(e.target.value)}
                        placeholder={i18n.t('transcript.search_placeholder')}
                        className="w-full bg-zinc-900/50 border border-zinc-800 rounded-md py-1 pl-7 pr-2 text-[10px] text-zinc-300 placeholder:text-zinc-700 focus:outline-none focus:border-zinc-700 transition-colors"
                    />
                </div>

                <div className="text-[9px] font-mono text-zinc-600">
                    {filtered_entries.length} {i18n.t('transcript.entries')}
                </div>
            </div>

            <div 
                ref={scroll_ref}
                className="flex-1 overflow-y-auto p-2 custom-scrollbar space-y-0.5"
            >
                <AnimatePresence initial={false}>
                    {filtered_entries.map(render_entry)}
                </AnimatePresence>
                
                {filtered_entries.length === 0 && (
                    <div className="h-full flex flex-col items-center justify-center opacity-20 py-12">
                        <Terminal size={32} className="text-zinc-600 mb-2" />
                        <span className="text-[10px] font-bold uppercase tracking-[0.2em]">{i18n.t('transcript.empty_state')}</span>
                    </div>
                )}
            </div>
        </div>
    );
};
