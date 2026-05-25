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

/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **AnomalyPanel Component**: Displays dead-code anomalies and static analysis alerts.
 * Enables direct navigation to offending nodes.
 */

import React, { useState } from 'react';
import { AlertTriangle, Copy, Check } from 'lucide-react';
import type { ExtendedGraphNode } from './types';
import { redact_sensitive_info } from './utils/authUtils';

interface AnomalyPanelProps {
    anomalies: string[];
    nodes: ExtendedGraphNode[];
    selected_node: ExtendedGraphNode | null;
    on_anomaly_click: (node: ExtendedGraphNode) => void;
}

export const AnomalyPanel: React.FC<AnomalyPanelProps> = ({
    anomalies,
    nodes,
    selected_node,
    on_anomaly_click
}) => {
    const [copied, setCopied] = useState(false);

    const handleCopyInstructions = (e: React.MouseEvent) => {
        e.stopPropagation();
        const formattedList = anomalies.map((anomaly) => {
            if (!anomaly) return '- Unknown Anomaly';
            const match = anomaly.match(/Unused symbol \(0 incoming references\): (\w+) in (.+)/);
            if (match) {
                const safePath = redact_sensitive_info(match[2]);
                return `- Unused symbol \`${match[1]}\` in file \`${safePath}\``;
            }
            return `- ${redact_sensitive_info(anomaly)}`;
        }).join('\n');

        const promptText = `Here are the static code anomalies found in our workspace:\n${formattedList}\n\nPlease inspect these locations, investigate why these symbols are unused, and provide a clean refactoring plan or code edits to resolve them.`;

        navigator.clipboard.writeText(promptText);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    return (
        <div className="absolute top-36 right-6 w-80 bg-zinc-900/80 backdrop-blur-xl border border-zinc-800 p-4 rounded-2xl flex flex-col gap-3 max-h-[300px] overflow-y-auto custom-scrollbar z-40 shadow-2xl">
            <div className="flex items-center justify-between border-b border-zinc-850 pb-2">
                <div className="flex items-center gap-2">
                    <AlertTriangle size={12} className="text-amber-500" />
                    <span className="text-[10px] font-black text-amber-500 uppercase tracking-widest">
                        Anomalies ({anomalies.length})
                    </span>
                </div>
                <button
                    onClick={handleCopyInstructions}
                    className="flex items-center gap-1 px-1.5 py-0.5 bg-zinc-950 hover:bg-zinc-800 border border-zinc-850 hover:border-zinc-700 rounded text-[8px] font-bold font-mono text-zinc-400 hover:text-white transition-all cursor-pointer"
                    title="Copy AI Prompt Instructions"
                >
                    {copied ? (
                        <>
                            <Check size={8} className="text-emerald-400" />
                            <span className="text-emerald-400">Copied</span>
                        </>
                    ) : (
                        <>
                            <Copy size={8} />
                            <span className="ml-1">Copy Prompt</span>
                        </>
                    )}
                </button>
            </div>
            <div className="flex flex-col gap-2">
                {anomalies.map((anomaly, idx) => {
                    if (!anomaly) return null;
                    const match = anomaly.match(/Unused symbol \(0 incoming references\): (\w+) in (.+)/);
                    const name = match ? match[1] : anomaly;
                    const rawPath = match ? match[2] : 'Unknown Path';
                    const path = redact_sensitive_info(rawPath);
                    
                    const is_selected = selected_node && selected_node.name === name && selected_node.path === path;
                    
                    return (
                        <div
                            key={idx}
                            onClick={() => {
                                if (name && rawPath) {
                                    // Use raw path for lookup to match the node property, but display the redacted path
                                    const node = nodes.find(n => n.name === name && n.path === rawPath);
                                    if (node) {
                                        on_anomaly_click(node);
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
                                <span className="text-[9px] font-bold text-amber-400 font-mono">Unused Symbol</span>
                                <span className="text-[8px] text-zinc-500 font-mono font-bold truncate max-w-[150px]" title={path || ''}>
                                    {path}
                                </span>
                            </div>
                            <span className="text-[11px] font-bold text-zinc-200 font-mono truncate group-hover/anom:text-white">
                                {name}
                            </span>
                        </div>
                    );
                })}
            </div>
        </div>
    );
};

// Metadata: [AnomalyPanel]
