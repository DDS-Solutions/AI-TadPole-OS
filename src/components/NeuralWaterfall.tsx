import React, { useMemo, useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Network } from 'lucide-react';
import { useTraceStore } from '../stores/traceStore';
import type { TraceSpan } from '../stores/traceStore';
import { useAgentStore } from '../stores/agentStore';

export const NeuralWaterfall: React.FC = () => {
    const { activeTraceId, getTraceTree } = useTraceStore();
    const { getAgent } = useAgentStore();
    const [now, setNow] = useState(() => Date.now());

    useEffect(() => {
        const interval = setInterval(() => setNow(Date.now()), 100);
        return () => clearInterval(interval);
    }, []);

    // Flatten tree and calculate timeline metrics
    const { timelineSpans, minTime, totalDuration } = useMemo(() => {
        if (!activeTraceId) return { timelineSpans: [], minTime: 0, totalDuration: 0 };

        const rawTree = getTraceTree(activeTraceId);
        const flat: (TraceSpan & { depth: number })[] = [];

        type TraceNode = TraceSpan & { children?: TraceNode[] };
        const flatten = (nodes: TraceNode[], depth: number) => {
            nodes.forEach(node => {
                flat.push({ ...node, depth });
                if (node.children) flatten(node.children, depth + 1);
            });
        };
        flatten(rawTree, 0);

        if (flat.length === 0) return { timelineSpans: [], minTime: 0, totalDuration: 0 };

        const min = Math.min(...flat.map(s => s.startTime));
        const max = Math.max(...flat.map(s => s.endTime || now));
        const duration = Math.max(1, max - min); // avoid div zero

        return { timelineSpans: flat, minTime: min, totalDuration: duration };
    }, [activeTraceId, getTraceTree, now]);

    if (!activeTraceId || timelineSpans.length === 0) return null;

    return (
        <div className="w-full h-48 bg-zinc-950/80 border-t border-zinc-900 absolute bottom-0 left-0 p-4 overflow-y-auto custom-scrollbar z-20 backdrop-blur-xl">
            <div className="flex items-center gap-2 mb-4">
                <Network size={14} className="text-cyan-500" />
                <span className="text-[10px] font-bold uppercase tracking-widest text-zinc-400">
                    Neural Waterfall (Gantt)
                </span>
                <span className="text-[9px] font-mono text-zinc-600 ml-auto">
                    {totalDuration}ms Total
                </span>
            </div>

            <div className="relative w-full">
                {timelineSpans.map((span) => {
                    const isRunning = !span.endTime;
                    const duration = isRunning ? (now - span.startTime) : (span.endTime! - span.startTime);

                    const leftPercent = ((span.startTime - minTime) / totalDuration) * 100;
                    const widthPercent = Math.max(0.5, (duration / totalDuration) * 100); // min width 0.5%

                    const agentName = getAgent(span.agentId)?.name || span.agentId;

                    const barColor = isRunning
                        ? 'bg-cyan-500/80 shadow-[0_0_10px_rgba(6,182,212,0.5)]'
                        : span.status === 'error'
                            ? 'bg-red-500/80'
                            : 'bg-emerald-500/80';

                    return (
                        <div key={span.id} className="flex items-center mb-1.5 group">
                            <div className="w-32 flex-shrink-0 text-right pr-4 truncate pt-1">
                                <span className="text-[9px] font-mono text-zinc-500 block uppercase tracking-wider">{agentName}</span>
                                <span className="text-[8px] font-mono text-zinc-600 block truncate">{span.name}</span>
                            </div>

                            <div className="flex-1 relative h-6 bg-zinc-900/50 rounded overflow-hidden">
                                {/* Grid lines background */}
                                <div className="absolute inset-0 bg-[linear-gradient(to_right,#ffffff05_1px,transparent_1px)] bg-[size:10%] pointer-events-none" />

                                <motion.div
                                    initial={{ opacity: 0, width: 0 }}
                                    animate={{
                                        opacity: 1,
                                        left: `${leftPercent}%`,
                                        width: `${widthPercent}%`
                                    }}
                                    className={`absolute top-1 bottom-1 rounded-sm ${barColor} flex items-center px-1 overflow-hidden`}
                                >
                                    {duration > 50 && (
                                        <span className="text-[8px] font-mono text-white/90 truncate drop-shadow-md">
                                            {duration}ms
                                        </span>
                                    )}
                                </motion.div>
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
};
