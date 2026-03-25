import React from 'react';
import { Activity, Code, Users, FlaskConical } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { McpToolHubDefinition } from '../../stores/skillStore';

interface McpToolCardProps {
    tool: McpToolHubDefinition;
    onAssign: (name: string) => void;
    onTest: (tool: McpToolHubDefinition) => void;
}

export const McpToolCard: React.FC<McpToolCardProps> = ({ tool, onAssign, onTest }) => {
    return (
        <div key={tool.name} className={`bg-zinc-950/40 border ${tool.isPulsing ? 'border-cyan-500 shadow-[0_0_15px_rgba(6,182,212,0.3)]' : 'border-zinc-800/60'} p-5 rounded-xl group relative overflow-hidden hover:border-cyan-500/30 transition-all duration-300`}>
            <div className="absolute top-4 right-4 text-[9px] font-bold tracking-tighter uppercase px-2 py-0.5 rounded border bg-zinc-900 border-zinc-800 text-zinc-500 group-hover:text-cyan-400 group-hover:border-cyan-500/20 transition-colors">
                {tool.source}
            </div>
            <div className="flex items-center gap-3 mb-3">
                <div className={`w-2 h-2 rounded-full ${tool.source === 'system' ? 'bg-cyan-500 shadow-[0_0_8px_rgba(6,182,212,0.5)]' : 'bg-zinc-600'} ${tool.isPulsing ? 'animate-ping' : ''}`}></div>
                <h3 className="font-mono text-sm text-zinc-100 font-bold">{tool.name}</h3>
                {tool.isPulsing && <Activity size={10} className="text-cyan-400 animate-pulse ml-1" />}
            </div>
            <p className="text-zinc-500 text-xs leading-relaxed mb-4 h-12 overflow-hidden line-clamp-3">
                {tool.description}
            </p>

            <div className="grid grid-cols-3 gap-2 mb-4 bg-black/30 rounded-lg p-2 border border-zinc-900">
                <Tooltip content="Total number of times this tool has been invoked by agents." position="top">
                    <div className="flex flex-col cursor-help">
                        <div className="text-[10px] text-zinc-500 uppercase tracking-widest font-bold mb-1">{i18n.t('skills.label_invocations')}</div>
                        <span className="text-[10px] font-mono text-zinc-400">{tool.stats.invocations}</span>
                    </div>
                </Tooltip>
                <Tooltip content="Percentage of executions that completed without errors." position="top">
                    <div className="flex flex-col border-x border-zinc-900 px-2 cursor-help">
                        <div className="text-[10px] text-zinc-500 uppercase tracking-widest font-bold mb-1">{i18n.t('skills.label_success_rate')}</div>
                        <span className={`text-[10px] font-mono ${tool.stats.invocations > 0 && (tool.stats.success_count / tool.stats.invocations) < 0.9 ? 'text-amber-500' : 'text-emerald-500'}`}>
                            {tool.stats.invocations > 0 ? Math.round((tool.stats.success_count / tool.stats.invocations) * 100) : 0}%
                        </span>
                    </div>
                </Tooltip>
                <Tooltip content="Average round-trip response time for this MCP tool." position="top">
                    <div className="flex flex-col items-end cursor-help">
                        <div className="text-[10px] text-zinc-500 uppercase tracking-widest font-bold mb-1 text-right">{i18n.t('skills.label_avg_latency')}</div>
                        <span className="text-[10px] font-mono text-amber-500">{tool.stats.avg_latency_ms}ms</span>
                    </div>
                </Tooltip>
            </div>

            <div className="flex items-center justify-between gap-3 mt-auto pt-4 border-t border-zinc-900">
                <details className="group/details">
                    <summary className="text-[9px] font-bold text-zinc-600 cursor-pointer list-none hover:text-zinc-400 transition-colors flex items-center gap-1 uppercase tracking-wider">
                        <Code size={10} /> {i18n.t('skills.label_schema')}
                    </summary>
                    <div className="absolute left-0 right-0 bottom-full mb-2 mx-4 bg-zinc-900/95 backdrop-blur-xl rounded-lg p-4 border border-zinc-800 shadow-2xl z-50 invisible group-open/details:visible opacity-0 group-open/details:opacity-100 transition-all max-h-64 overflow-y-auto custom-scrollbar">
                        <pre className="text-[10px] text-cyan-300 font-mono">
                            {JSON.stringify(tool.input_schema, null, 2)}
                        </pre>
                    </div>
                </details>

                 <Tooltip content={i18n.t('agent_manager.btn_assign')} position="top">
                     <button onClick={() => onAssign(tool.name)} className="flex items-center gap-2 px-3 py-1.5 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-emerald-500/50 rounded-lg text-xs font-bold text-zinc-400 hover:text-emerald-400 transition-all">
                         <Users className="w-3.5 h-3.5" /> {i18n.t('agent_manager.btn_assign')}
                     </button>
                 </Tooltip>
                 <Tooltip content={i18n.t('skills.tooltip_test_tool')} position="top">
                     <button onClick={() => onTest(tool)} className="flex items-center gap-2 px-3 py-1.5 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-cyan-500/50 rounded-lg text-xs font-bold text-zinc-400 hover:text-cyan-400 transition-all">
                         <FlaskConical className="w-3.5 h-3.5" /> {i18n.t('skills.btn_test_tool')}
                     </button>
                 </Tooltip>
            </div>
        </div>
    );
};
