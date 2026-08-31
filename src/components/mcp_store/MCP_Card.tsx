/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Mcp_Store / MCP_Card
 * - **Primary Entrypoints**: `MCP_Card`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { Server, Download, Check } from 'lucide-react';
import type { MCP_Connector } from './types';

interface MCP_CardProps {
    connector: MCP_Connector;
}

export function MCP_Card({ connector }: MCP_CardProps) {
    return (
        <div className="sovereign-panel-hover sovereign-panel flex flex-col h-full bg-[color:var(--color-surface)] group cursor-pointer transition-sovereign relative overflow-hidden">
            {/* Neural Pulse Highlight Effect */}
            <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-transparent via-emerald-500/20 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
            
            <div className="flex justify-between items-start mb-4 relative z-10">
                <div className="w-10 h-10 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center group-hover:bg-emerald-500/20 transition-colors">
                    <Server size={20} className="text-emerald-500" />
                </div>
                <div className="px-2 py-1 rounded text-[10px] font-mono uppercase bg-zinc-800/80 text-zinc-400 border border-zinc-700">
                    v{connector.version}
                </div>
            </div>
            
            <h3 className="text-lg font-semibold text-zinc-100 mb-2 group-hover:text-emerald-400 transition-colors relative z-10 tracking-tight">
                {connector.name}
            </h3>
            
            <p className="text-sm text-zinc-400 flex-grow line-clamp-3 mb-4 relative z-10 leading-relaxed">
                {connector.description}
            </p>
            
            <div className="flex items-center justify-between pt-4 border-t border-[color:var(--color-border)]/50 mt-auto relative z-10">
                <span className="text-xs font-medium text-zinc-500 flex items-center gap-1">
                    <span className="w-1.5 h-1.5 rounded-full bg-zinc-600"></span>
                    {connector.category}
                </span>
                
                <button 
                    onClick={(e) => { e.stopPropagation(); }}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-zinc-800 hover:bg-zinc-700 text-xs font-medium text-zinc-300 transition-sovereign border border-zinc-700/50 hover:border-zinc-600 focus-sovereign"
                >
                    {connector.installed ? (
                        <>
                            <Check size={14} className="text-emerald-500" />
                            Installed
                        </>
                    ) : (
                        <>
                            <Download size={14} className="text-zinc-400 group-hover:text-zinc-300" />
                            Install
                        </>
                    )}
                </button>
            </div>
        </div>
    );
}
