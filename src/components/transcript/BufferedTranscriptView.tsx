import React, { useState, useEffect, useRef, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Terminal, Brain, Wrench, AlertCircle, CheckCircle2, Search } from 'lucide-react';
import { EventBus, type LogEntry } from '../../services/eventBus';
import { i18n } from '../../i18n';
import clsx from 'clsx';

interface BufferedTranscriptViewProps {
    agentId?: string;
    missionId?: string;
    className?: string;
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
export const BufferedTranscriptView: React.FC<BufferedTranscriptViewProps> = ({ 
    agentId, 
    missionId,
    className 
}) => {
    const [entries, setEntries] = useState<LogEntry[]>([]);
    const [filter, setFilter] = useState('');
    const scrollRef = useRef<HTMLDivElement>(null);
    const seenIds = useRef<Set<string>>(new Set());
    const bufferRef = useRef<LogEntry[]>([]);

    useEffect(() => {
        // subscribe to the global event bus
        const unsubscribe = EventBus.subscribe((entry) => {
            // Filter by agent or mission if provided
            if (agentId && entry.agentId !== agentId) return;
            if (missionId && entry.missionId !== missionId) return;
            
            // De-duplicate using backend-provided IDs
            if (entry.id && seenIds.current.has(entry.id)) return;
            if (entry.id) seenIds.current.add(entry.id);

            bufferRef.current.push(entry);
        });

        // High-frequency batching via RAF
        let rafId: number;
        const flush = () => {
            if (bufferRef.current.length > 0) {
                const batch = [...bufferRef.current];
                bufferRef.current = [];
                setEntries(prev => {
                    const next = [...prev, ...batch];
                    return next.slice(-1000); // hard cap for performance
                });
            }
            rafId = requestAnimationFrame(flush);
        };
        rafId = requestAnimationFrame(flush);

        return () => {
            unsubscribe();
            cancelAnimationFrame(rafId);
        };
    }, [agentId, missionId]);

    // Auto-scroll
    useEffect(() => {
        if (scrollRef.current) {
            scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
        }
    }, [entries]);

    const filteredEntries = useMemo(() => {
        if (!filter) return entries;
        const lower = filter.toLowerCase();
        return entries.filter(e => e.text.toLowerCase().includes(lower) || e.source.toLowerCase().includes(lower));
    }, [entries, filter]);

    const renderEntry = (entry: LogEntry) => {
        const isThought = entry.metadata?.type === 'thought' || entry.text.startsWith('Thinking:');
        const isTool = entry.metadata?.type === 'tool' || entry.text.includes('Executing tool:');
        
        let icon = <Terminal size={12} />;
        let color = 'text-zinc-400';
        let bg = 'bg-zinc-950/20';

        if (isThought) {
            icon = <Brain size={12} className="text-purple-400" />;
            color = 'text-purple-300';
            bg = 'bg-purple-500/5';
        } else if (isTool) {
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
                    isThought ? "border-purple-500/50" : 
                    isTool ? "border-blue-500/50" : 
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
        <div className={clsx("flex flex-col h-full bg-black/40 border border-zinc-800 rounded-xl overflow-hidden backdrop-blur-md", className)}>
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
                        onChange={(e) => setFilter(e.target.value)}
                        placeholder={i18n.t('transcript.search_placeholder')}
                        className="w-full bg-zinc-900/50 border border-zinc-800 rounded-md py-1 pl-7 pr-2 text-[10px] text-zinc-300 placeholder:text-zinc-700 focus:outline-none focus:border-zinc-700 transition-colors"
                    />
                </div>

                <div className="text-[9px] font-mono text-zinc-600">
                    {filteredEntries.length} {i18n.t('transcript.entries')}
                </div>
            </div>

            <div 
                ref={scrollRef}
                className="flex-1 overflow-y-auto p-2 custom-scrollbar space-y-0.5"
            >
                <AnimatePresence initial={false}>
                    {filteredEntries.map(renderEntry)}
                </AnimatePresence>
                
                {filteredEntries.length === 0 && (
                    <div className="h-full flex flex-col items-center justify-center opacity-20 py-12">
                        <Terminal size={32} className="text-zinc-600 mb-2" />
                        <span className="text-[10px] font-bold uppercase tracking-[0.2em]">{i18n.t('transcript.empty_state')}</span>
                    </div>
                )}
            </div>
        </div>
    );
};
