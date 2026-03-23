import React, { useState, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Activity, Share2, Clock, GitCommit } from 'lucide-react';
import { useAgentStore } from '../stores/agentStore';
import { useTraceStore } from '../stores/traceStore';
import type { TraceNode } from '../stores/traceStore';
import { i18n } from '../i18n';

// Recursive component to render the OTel trace tree
const TraceTreeNode: React.FC<{ node: TraceNode; depth: number }> = ({ node, depth }): React.ReactElement => {
    const { getAgent } = useAgentStore();
    const agentName = getAgent(node.agentId)?.name || node.agentId;

    // Status colors
    const statusColor = node.status === 'running'
        ? 'text-cyan-400 bg-cyan-400/10 border-cyan-400/30'
        : node.status === 'error'
            ? 'text-red-400 bg-red-400/10 border-red-400/30'
            : 'text-emerald-400 bg-emerald-400/10 border-emerald-400/30';

    return (
        <div className="w-full flex flex-col pt-2">
            <div
                className="flex items-start gap-3 relative"
                style={{ marginLeft: `${depth * 16}px` }}
            >
                {/* Visual line connecting parent to child */}
                {depth > 0 && (
                    <div className="absolute -left-4 top-4 w-4 h-px bg-zinc-700" />
                )}
                {depth > 0 && (
                    <div className="absolute -left-4 -top-full bottom-auto h-[calc(100%+16px)] w-px bg-zinc-700" />
                )}

                <div className="flex-1 p-3 bg-zinc-900/60 border border-zinc-800 rounded-xl hover:bg-zinc-800/80 transition-all">
                    <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2">
                            <div className={`w-2 h-2 rounded-full shadow-[0_0_8px_currentColor] ${statusColor}`} />
                            <span className="text-[10px] font-mono font-bold text-zinc-300 uppercase tracking-widest">{agentName}</span>
                            <span className="text-[9px] text-zinc-500 font-mono px-1.5 py-0.5 rounded bg-zinc-950/50">
                                {node.name}
                            </span>
                        </div>
                        <span className="text-[9px] text-zinc-600 font-mono flex items-center gap-1">
                            <Clock size={10} />
                            {node.endTime ? `${node.endTime - node.startTime}ms` : i18n.t('trace.running')}
                        </span>
                    </div>

                    <div className="flex items-center gap-4 mt-2 pt-2 border-t border-zinc-800/50">
                        <span className="text-[8px] font-mono text-zinc-500 flex items-center gap-1">
                            <GitCommit size={10} /> {i18n.t('trace.span')}: {node.id.toUpperCase()}
                        </span>
                        {node.attributes && Object.keys(node.attributes).length > 0 && (
                            <span className="text-[8px] font-mono text-zinc-500 truncate max-w-[150px]">
                                {JSON.stringify(node.attributes)}
                            </span>
                        )}
                    </div>
                </div>
            </div>

            {/* Recursively render children */}
            {node.children && node.children.length > 0 && (
                <div className="flex flex-col relative w-full">
                    {node.children.map((child): React.ReactElement => (
                        <TraceTreeNode key={child.id} node={child} depth={depth + 1} />
                    ))}
                </div>
            )}
        </div>
    );
};

export const LineageStream: React.FC = (): React.ReactElement => {
    const { activeTraceId, getTraceTree } = useTraceStore();
    const [sidebarWidth, setSidebarWidth] = useState(380);
    const streamRef = useRef<HTMLDivElement>(null);

    const activeTree = activeTraceId ? getTraceTree(activeTraceId) : [];

    const handleSidebarResizeStart = (e: React.MouseEvent): void => {
        e.preventDefault();
        const startX = e.clientX;
        const startWidth = sidebarWidth;

        const onMouseMove = (moveEvent: MouseEvent): void => {
            const currentX = moveEvent.clientX;
            const deltaX = startX - currentX;
            setSidebarWidth(Math.min(800, Math.max(300, startWidth + deltaX)));
        };

        const onMouseUp = (): void => {
            document.removeEventListener('mousemove', onMouseMove);
            document.removeEventListener('mouseup', onMouseUp);
        };

        document.addEventListener('mousemove', onMouseMove);
        document.addEventListener('mouseup', onMouseUp);
    };

    return (
        <div
            className="flex flex-col h-full bg-zinc-950/20 border-l border-zinc-900 overflow-hidden relative group/sidebar"
            style={{ width: sidebarWidth }}
            ref={streamRef}
        >
            <div
                onMouseDown={handleSidebarResizeStart}
                className="absolute inset-y-0 left-0 w-1 cursor-col-resize hover:bg-emerald-500/20 active:bg-emerald-500/40 transition-colors z-20"
            />

            <div className="p-4 border-b border-zinc-900 flex items-center justify-between bg-zinc-950/40 backdrop-blur-md sticky top-0 z-10">
                <div className="flex items-center gap-2">
                    <Activity size={16} className="text-emerald-500" />
                    <span className="text-[10px] font-bold uppercase tracking-widest text-zinc-400">
                        {i18n.t('trace.stream_title')}
                    </span>
                </div>
                {activeTraceId && (
                    <span className="text-[9px] font-mono px-2 py-0.5 rounded bg-zinc-900 border border-zinc-800 text-zinc-500 truncate max-w-[120px]">
                        {activeTraceId}
                    </span>
                )}
            </div>

            <div className="flex-1 overflow-y-auto p-4 space-y-2 custom-scrollbar relative">
                <AnimatePresence>
                    {activeTree.map((rootNode): React.ReactElement => (
                        <motion.div
                            key={rootNode.id}
                            initial={{ opacity: 0, scale: 0.95 }}
                            animate={{ opacity: 1, scale: 1 }}
                            exit={{ opacity: 0 }}
                        >
                            <TraceTreeNode node={rootNode} depth={0} />
                        </motion.div>
                    ))}
                </AnimatePresence>

                {activeTree.length === 0 && (
                    <div className="flex flex-col items-center justify-center h-48 opacity-20">
                        <Share2 size={32} className="text-zinc-500 mb-2" />
                        <span className="text-[10px] font-bold uppercase tracking-widest text-zinc-600 text-center px-4">
                            {i18n.t('trace.waiting')}<br />{i18n.t('trace.waiting_hint')}
                        </span>
                    </div>
                )}
            </div>
        </div>
    );
};
