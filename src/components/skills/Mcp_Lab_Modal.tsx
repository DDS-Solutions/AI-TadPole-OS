/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Experimental laboratory for MCP (Model Context Protocol) integration. 
 * Facilitates the testing and discovery of external tools and server-side capabilities.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: MCP server handshake timeout (unreachable), tool manifest parsing error, or lab session leak.
 * - **Telemetry Link**: Search for `[Mcp_Lab_Modal]` or `mcp_discovery` in service logs.
 */

import React from 'react';
import { type Mcp_Tool_Hub_Definition } from '../../stores/skill_store';

interface Mcp_Lab_Modal_Props {
    tool: Mcp_Tool_Hub_Definition | null;
    open: boolean;
    on_close: () => void;
}

export const Mcp_Lab_Modal: React.FC<Mcp_Lab_Modal_Props> = ({ tool, open, on_close }) => {
    if (!open || !tool) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-300">
            <div className="bg-zinc-900 border border-zinc-800 rounded-2xl w-full max-w-2xl overflow-hidden animate-in zoom-in-95 duration-300">
                <div className="p-6 border-b border-zinc-800 flex justify-between items-center">
                    <div>
                        <h3 className="text-xl font-bold text-zinc-100">{tool.name}</h3>
                        <p className="text-sm text-zinc-500 mt-1">{tool.source}</p>
                    </div>
                    <button 
                        onClick={on_close}
                        className="text-zinc-500 hover:text-zinc-100 transition-colors"
                    >
                        Close
                    </button>
                </div>
                <div className="p-6 space-y-4">
                    <p className="text-zinc-400">{tool.description}</p>
                    <div className="space-y-2">
                        <p className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest">Input Schema</p>
                        <pre className="bg-black/50 p-4 rounded-xl border border-zinc-800 font-mono text-xs text-blue-400 overflow-auto max-h-[300px]">
                            {JSON.stringify(tool.input_schema, null, 2)}
                        </pre>
                    </div>
                </div>
            </div>
        </div>
    );
};

