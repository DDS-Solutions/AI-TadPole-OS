/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:UI-Components**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[AnomalyPanel]` in observability traces.
 */

import React, { useState, useCallback, useRef, useEffect, useMemo } from 'react';
import { AlertTriangle, Copy, Check } from 'lucide-react';
import { redact_sensitive_info } from './utils/authUtils';
import { parseAnomaly } from './utils/anomalyParser';
import { useGraphDataContext, useSelectionContext } from './graph_context_hooks';
import { i18n } from '../../../i18n';

export const AnomalyPanel: React.FC = () => {
    const { data, graphData } = useGraphDataContext();
    const { selectedNode, selectNode } = useSelectionContext();

    const [copied, setCopied] = useState(false);
    const copyTimeoutRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

    // P0-3: Cleanup timeout on unmount to prevent setState on unmounted component
    useEffect(() => {
        return () => {
            if (copyTimeoutRef.current) {
                clearTimeout(copyTimeoutRef.current);
            }
        };
    }, []);

    // P1-6: Memoize copy handler (must be above early return for hooks rule)
    const handleCopyInstructions = useCallback(async (e: React.MouseEvent) => {
        e.stopPropagation();
        if (!data?.anomalies) return;

        const parsed = data.anomalies
            .filter((a): a is string => !!a)
            .map((anomaly, idx) => parseAnomaly(anomaly, idx));

        const formattedList = parsed.map((p) => {
            if (p.type === 'UNUSED_SYMBOL' && p.rawPath) {
                const safePath = redact_sensitive_info(p.rawPath);
                return `- Unused symbol \`${p.name}\` in file \`${safePath}\``;
            }
            return `- ${redact_sensitive_info(p.original)}`;
        }).join('\n');

        const promptText = i18n.t('knowledge_graph.anomaly_prompt_template', { list: formattedList });

        // P0-2: Clipboard robustness — existence check + try/catch + fallback
        try {
            if (navigator.clipboard && typeof navigator.clipboard.writeText === 'function') {
                await navigator.clipboard.writeText(promptText);
            } else {
                // Fallback for non-secure contexts (HTTP) or missing Clipboard API
                const textarea = document.createElement('textarea');
                textarea.value = promptText;
                textarea.style.position = 'fixed';
                textarea.style.left = '-9999px';
                textarea.style.opacity = '0';
                document.body.appendChild(textarea);
                textarea.select();
                document.execCommand('copy');
                document.body.removeChild(textarea);
            }
            setCopied(true);
            // P0-3: Track timeout ref for cleanup
            copyTimeoutRef.current = setTimeout(() => setCopied(false), 2000);
        } catch (err) {
            console.error('[AnomalyPanel] Copy to clipboard failed:', err);
        }
    }, [data]);

    // P1-4: Use extracted parser for structured anomaly data (memoized, must be above early return)
    const parsedAnomalies = useMemo(() => 
        (data?.anomalies ?? [])
            .filter((a): a is string => !!a)
            .map((anomaly, idx) => parseAnomaly(anomaly, idx)),
        [data?.anomalies]
    );

    if (!data?.anomalies || data.anomalies.length === 0) return null;

    const { nodeMap } = graphData;

    return (
        <div className="absolute top-36 right-6 w-80 bg-zinc-900/80 backdrop-blur-xl border border-zinc-800 p-4 rounded-2xl flex flex-col gap-3 max-h-[300px] overflow-y-auto custom-scrollbar z-40 shadow-2xl">
            <div className="flex items-center justify-between border-b border-zinc-850 pb-2">
                <div className="flex items-center gap-2">
                    <AlertTriangle size={12} className="text-amber-500" />
                    <span className="text-[10px] font-black text-amber-500 uppercase tracking-widest">
                        {i18n.t('knowledge_graph.anomaly_title', { count: parsedAnomalies.length })}
                     </span>
                </div>
                <button
                    onClick={handleCopyInstructions}
                    className="flex items-center gap-1 px-1.5 py-0.5 bg-zinc-950 hover:bg-zinc-800 border border-zinc-850 hover:border-zinc-700 rounded text-[8px] font-bold font-mono text-zinc-400 hover:text-white transition-all cursor-pointer"
                    title={i18n.t('knowledge_graph.anomaly_tooltip_copy')}
                >
                    {copied ? (
                        <>
                            <Check size={8} className="text-emerald-400" />
                            <span className="text-emerald-400">{i18n.t('knowledge_graph.anomaly_btn_copied')}</span>
                        </>
                    ) : (
                        <>
                            <Copy size={8} />
                            <span className="ml-1">{i18n.t('knowledge_graph.anomaly_btn_copy')}</span>
                        </>
                    )}
                </button>
            </div>
            <div className="flex flex-col gap-2">
                {parsedAnomalies.map((parsed) => {
                    const displayPath = parsed.rawPath
                        ? redact_sensitive_info(parsed.rawPath)
                        : i18n.t('knowledge_graph.anomaly_unknown_path');
                    
                    const is_selected = selectedNode
                        && selectedNode.name === parsed.name
                        && parsed.rawPath !== null
                        && selectedNode.path === parsed.rawPath;
                    
                    return (
                        <div
                            // P0-1: Stable key from parser instead of array index
                            key={parsed.stableKey}
                            onClick={() => {
                                if (parsed.name && parsed.rawPath) {
                                    // O(1) lookup via nodeMap instead of O(N) nodes.find
                                    const nodeId = `${parsed.rawPath}:${parsed.name}`;
                                    const node = nodeMap.get(nodeId);
                                    if (node) {
                                        selectNode(node, true);
                                    }
                                }
                            }}
                            className={`p-2.5 rounded-xl border transition-all cursor-pointer text-left flex flex-col gap-1 group/anom ${
                                is_selected 
                                    ? 'bg-amber-500/10 border-amber-500 shadow-[0_0_15px_rgba(245,158,11,0.15)]' 
                                    : 'bg-zinc-950/60 hover:bg-zinc-950 border-zinc-800 hover:border-amber-500/50'
                            }`}
                        >
                            <div className="flex items-center gap-1.5 justify-between">
                                <span className="text-[9px] font-bold text-amber-400 font-mono">
                                    {parsed.type === 'UNUSED_SYMBOL' ? i18n.t('knowledge_graph.anomaly_type_unused') : i18n.t('knowledge_graph.anomaly_type_general')}
                                </span>
                                <span className="text-[8px] text-zinc-500 font-mono font-bold truncate max-w-[150px]" title={displayPath}>
                                    {displayPath}
                                </span>
                            </div>
                            <span className="text-[11px] font-bold text-zinc-200 font-mono truncate group-hover/anom:text-white">
                                {parsed.name}
                            </span>
                        </div>
                    );
                })}
            </div>
        </div>
    );
};

// Metadata: [AnomalyPanel]
