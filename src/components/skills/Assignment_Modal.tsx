/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Capability assignment bridge for agent/swarm nodes. 
 * Orchestrates the linking of Skills and Workflows to active agent slots via the `skill_store`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Assignment sync failure if `agent_id` is invalid, or skill list empty due to store hydration lag.
 * - **Telemetry Link**: Search for `[Assignment_Modal]` or `assign_skill` in service logs.
 */

import React from 'react';
import { Users, Check } from 'lucide-react';
import { i18n } from '../../i18n';
import type { Agent } from '../../types';

interface Assignment_Modal_Props {
    is_open: boolean;
    on_close: () => void;
    assign_target: { type: 'skill' | 'workflow' | 'mcp', name: string } | null;
    agents: Agent[];
    on_toggle_assignment: (agent_id: string) => void;
}

export const Assignment_Modal: React.FC<Assignment_Modal_Props> = ({
    is_open,
    on_close,
    assign_target,
    agents,
    on_toggle_assignment
}) => {
    if (!is_open || !assign_target) return null;

    return (
        <div className="fixed inset-0 z-[60] bg-black/80 backdrop-blur-md flex items-center justify-center p-4">
            <div className="bg-zinc-950 border border-zinc-800 w-full max-w-md rounded-2xl shadow-2xl flex flex-col relative overflow-hidden">
                <div className="neural-grid opacity-10" />
                <div className="p-5 border-b border-zinc-800 flex justify-between items-center shrink-0 relative z-10 bg-zinc-950/50">
                    <h2 className="text-sm font-bold text-zinc-100 flex items-center gap-2 uppercase tracking-wider">
                        <Users className="text-emerald-500" size={16} /> {i18n.t('agent_manager.modal_assign_title', { type: assign_target.type.toUpperCase(), name: assign_target.name })}
                    </h2>
                    <button onClick={on_close} className="text-zinc-500 hover:text-zinc-300 p-1">✕</button>
                </div>
                <div className="p-6 space-y-4 relative z-10 bg-zinc-950/80">
                    <p className="text-xs text-zinc-500 font-mono leading-relaxed">
                        {i18n.t('agent_manager.label_select_agents')}
                    </p>
                    <div className="space-y-2 max-h-64 overflow-y-auto custom-scrollbar pr-2">
                        {agents.map(agent => {
                            const is_assigned = 
                                (assign_target.type === 'skill' && (agent.skills || []).includes(assign_target.name)) ||
                                (assign_target.type === 'workflow' && (agent.workflows || []).includes(assign_target.name)) ||
                                (assign_target.type === 'mcp' && (agent.mcp_tools || []).includes(assign_target.name));
                            
                            return (
                                <button
                                    key={agent.id}
                                    onClick={() => on_toggle_assignment(agent.id)}
                                    className={`w-full flex items-center justify-between p-3 rounded-xl border transition-all ${
                                        is_assigned 
                                            ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400 shadow-[0_0_10px_rgba(16,185,129,0.1)]' 
                                            : 'bg-zinc-900/50 border-zinc-800 text-zinc-400 hover:border-zinc-700'
                                    }`}
                                >
                                    <div className="flex items-center gap-3">
                                        <div 
                                            className="w-8 h-8 rounded-lg flex items-center justify-center font-bold text-sm shrink-0"
                                            style={{ 
                                                backgroundColor: agent.theme_color ? `${agent.theme_color}20` : '#27272a',
                                                color: agent.theme_color || '#71717a',
                                                border: `1px solid ${agent.theme_color ? `${agent.theme_color}40` : '#3f3f46'}`
                                            }}
                                        >
                                            {agent.name[0]}
                                        </div>
                                        <div className="text-left">
                                            <div className="text-xs font-bold truncate max-w-[180px]">{agent.name}</div>
                                            <div className="text-[10px] opacity-50 font-mono uppercase">{agent.role}</div>
                                        </div>
                                    </div>
                                    {is_assigned && <Check size={14} className="text-emerald-500" />}
                                </button>
                            );
                        })}
                    </div>
                </div>
                <div className="p-6 border-t border-zinc-800 bg-zinc-900/50 flex justify-end">
                    <button 
                        onClick={on_close} 
                        className="px-6 py-2 bg-zinc-800 hover:bg-zinc-700 text-white rounded-lg text-xs font-bold uppercase tracking-widest transition-all"
                    >
                        Done
                    </button>
                </div>
            </div>
        </div>
    );
};

