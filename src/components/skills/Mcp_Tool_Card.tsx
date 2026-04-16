/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Tool identity card for the MCP ecosystem. 
 * Renders tool parameters (JSON Schema) and provides real-time invocation status for autonomous agent tools.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Schema visualization overflow, or invocation pulse desync from ground-truth tool execution.
 * - **Telemetry Link**: Search for `[Mcp_Tool_Card]` or `tool_invocation` in UI tracing.
 */

import React from 'react';
import { Terminal, Shield, Workflow, ExternalLink, Activity, Info } from 'lucide-react';
import { type Mcp_Tool_Hub_Definition } from '../../stores/skill_store';
import { Tooltip } from '../ui/Tooltip';
import { i18n } from '../../i18n';

interface Mcp_Tool_Card_Props {
    tool: Mcp_Tool_Hub_Definition;
    on_edit: (tool: Mcp_Tool_Hub_Definition) => void;
}

export const Mcp_Tool_Card: React.FC<Mcp_Tool_Card_Props> = ({ tool, on_edit }) => {
    return (
        <div className={`bg-zinc-900 border ${tool.is_pulsing ? 'border-blue-500 shadow-lg shadow-blue-500/20' : 'border-zinc-800'} rounded-2xl overflow-hidden transition-all duration-300 hover:border-zinc-700 group flex flex-col`}>
            <div className="p-5 flex-1 space-y-4">
                <div className="flex justify-between items-start">
                    <div className="flex items-center gap-3">
                        <div className={`p-2 rounded-xl transition-colors ${tool.is_pulsing ? 'bg-blue-600' : 'bg-zinc-800 group-hover:bg-zinc-700'}`}>
                            <Terminal size={18} className={tool.is_pulsing ? 'text-white' : 'text-blue-500'} />
                        </div>
                        <div>
                            <h4 className="font-bold text-zinc-100 uppercase tracking-tight group-hover:text-blue-400 transition-colors">{tool.name}</h4>
                            <div className="flex items-center gap-2 mt-0.5">
                                <span className="text-[9px] font-mono text-zinc-500 uppercase tracking-tighter">{tool.source}</span>
                            </div>
                        </div>
                    </div>
                    <div className="flex items-center gap-1.5 grayscale group-hover:grayscale-0 opacity-40 group-hover:opacity-100 transition-all">
                        <Tooltip content={i18n.t('skills.verified_mcp_toolkit')}>
                            <Shield size={14} className="text-emerald-500" />
                        </Tooltip>
                        <Tooltip content={i18n.t('skills.view_mcp_source')}>
                            <ExternalLink size={14} className="text-zinc-500 hover:text-zinc-100 cursor-pointer" />
                        </Tooltip>
                    </div>
                </div>

                <p className="text-sm text-zinc-400 line-clamp-2 leading-relaxed">
                    {tool.description || i18n.t('skills.no_description')}
                </p>

                <div className="grid grid-cols-2 gap-3 pt-2">
                    <div className="bg-zinc-950/50 rounded-xl p-2.5 border border-zinc-800/50 space-y-1">
                        <div className="flex items-center gap-2 text-[8px] font-bold text-zinc-500 uppercase tracking-widest">
                            <Activity size={10} /> {i18n.t('skills.stats_invocations')}
                        </div>
                        <p className="text-sm font-mono font-bold text-zinc-200">{tool.stats?.invocations || 0}</p>
                    </div>
                    <div className="bg-zinc-950/50 rounded-xl p-2.5 border border-zinc-800/50 space-y-1">
                        <div className="flex items-center gap-2 text-[8px] font-bold text-zinc-500 uppercase tracking-widest">
                            <Workflow size={10} /> {i18n.t('skills.stats_reliability')}
                        </div>
                        <p className="text-sm font-mono font-bold text-zinc-200">
                            {tool.stats?.invocations > 0 
                                ? Math.round((tool.stats.success_count / tool.stats.invocations) * 100) 
                                : 100}%
                        </p>
                    </div>
                </div>
            </div>

            <div className="p-3 bg-zinc-950/50 border-t border-zinc-800/50 flex gap-2">
                <button
                    onClick={() => on_edit(tool)}
                    className="flex-1 px-4 py-2 bg-zinc-900 hover:bg-zinc-800 text-zinc-300 text-xs font-bold rounded-lg transition-all flex items-center justify-center gap-2 border border-zinc-800"
                >
                    <Info size={14} />
                    {i18n.t('skills.lab_view_button')}
                </button>
            </div>
        </div>
    );
};

