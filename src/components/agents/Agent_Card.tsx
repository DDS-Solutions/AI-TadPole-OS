/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Agent Visualization Node**: Interactive card for displaying agent status, capabilities (Skills, WFK, MCP), and real-time mission progress. 
 * Orchestrates the reactive rendering of agent telemetry (Tokens, USD) and provides entry to the `AgentConfigPanel`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Memoization stall (Agent_Card_Memo) causing status lag, theme-color calculation overflow, or missing tooltips on overflowed skill badges.
 * - **Telemetry Link**: Search `[Agent_Card]` in observability traces.
 */

import React from 'react';
import { Sliders, Activity, Cpu, Shield, Zap, DollarSign, Code, FileText, Terminal } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Agent } from '../../types';
import { get_active_model_name } from '../../utils/model_utils';
import { Node_Task_Box } from '../hierarchy/Node_Task_Box';

interface AgentCardProps {
    agent: Agent;
    on_select: (agent: Agent) => void;
}

const ACTIVE_STATUSES = ['active', 'thinking', 'speaking', 'coding'] as const;
const STATUS_BADGE_COLORS: Record<string, string> = {
    active: 'text-success-text bg-success-bg',
    thinking: 'text-warning-text bg-warning-bg',
    speaking: 'text-info-text bg-info-bg',
    coding: 'text-[color:var(--color-theme-cyan)] bg-[color:color-mix(in_srgb,var(--color-theme-cyan)_10%,transparent)]',
};

export const Agent_Card: React.FC<AgentCardProps> = ({ agent, on_select }) => {
    const is_busy = ACTIVE_STATUSES.includes(agent.status as typeof ACTIVE_STATUSES[number]);
    const agent_color = agent.theme_color || '#22c55e';

    return (
        <div
            role="button"
            tabIndex={0}
            onClick={() => on_select(agent)}
            onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && on_select(agent)}
            className="group sovereign-card cursor-pointer p-4 hover:sovereign-glow-green"
            aria-label={i18n.t('agent_manager.aria_configure_agent', { name: agent.name })}
        >
            <div className="absolute top-0 right-0 p-3 opacity-0 group-hover:opacity-100 transition-opacity">
                <Tooltip content={i18n.t('agent_manager.tooltip_configure')} position="left">
                    <Sliders size={16} className="text-green-500" />
                </Tooltip>
            </div>

            <div className="flex items-start gap-4">
                <div
                    className="w-10 h-10 rounded-lg flex items-center justify-center font-bold text-lg shrink-0 transition-all duration-300"
                    style={{
                        backgroundColor: is_busy ? `${agent_color}15` : '#27272a',
                        color: is_busy ? agent_color : '#71717a',
                        border: `1px solid ${is_busy ? `${agent_color}30` : '#3f3f46'}`,
                        boxShadow: is_busy ? `0 0 12px ${agent_color}20` : 'none',
                    }}
                >
                    {(agent.name || '?')[0]}
                </div>
                <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                        <h3
                            className="text-sm font-bold text-zinc-200 truncate group-hover:text-emerald-400 transition-colors"
                        >
                            {agent.name}
                        </h3>
                        {is_busy && (
                            <Tooltip content={i18n.t('agent_manager.tooltip_active')} position="top">
                                <span className={`flex items-center gap-1 text-[9px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded cursor-help ${STATUS_BADGE_COLORS[agent.status] || 'text-success-text bg-success-bg'}`}>
                                    <Activity size={8} /> {agent.status.toUpperCase()}
                                </span>
                            </Tooltip>
                        )}
                        {agent.category === 'user' && agent.metadata?.has_participated_in_swarm === true && (
                            <Tooltip content="This specialist has been recruited into the AI Swarm pool." position="top">
                                <span className="flex items-center gap-1 text-[9px] font-bold text-emerald-400 uppercase tracking-wider bg-emerald-500/10 px-1.5 py-0.5 rounded cursor-help">
                                    <Cpu size={8} /> Recruited
                                </span>
                            </Tooltip>
                        )}
                        {agent.category === 'ai' && (
                            <Tooltip content="Autonomous swarm specialist spawned for mission objectives." position="top">
                                <span className="flex items-center gap-1 text-[9px] font-bold text-amber-500 uppercase tracking-wider bg-amber-500/10 px-1.5 py-0.5 rounded cursor-help">
                                    <Cpu size={8} /> Hive Brain
                                </span>
                            </Tooltip>
                        )}
                    </div>
                    <Tooltip content={i18n.t('agent_manager.tooltip_role')} position="top">
                        <p className="text-[10px] text-zinc-500 font-mono truncate uppercase flex items-center gap-1.5 cursor-help">
                            <Shield size={10} /> {agent.role}
                        </p>
                    </Tooltip>

                    <div className="mt-4 pt-4 border-t border-white/5 relative z-10">
                        <Node_Task_Box agent={agent} />
                    </div>

                    <div className="mt-4 flex flex-wrap gap-2">
                        <Tooltip content={`LLM: ${get_active_model_name(agent)} `} position="top">
                            <div className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded px-2 py-1 flex items-center gap-1.5 cursor-help">
                                <Cpu size={10} className="text-green-400" />
                                <span className="text-[10px] text-zinc-400 font-mono">{(get_active_model_name(agent) || 'Unknown').split(' ').pop()}</span>
                            </div>
                        </Tooltip>
                        <Tooltip content={i18n.t('agent_manager.tooltip_temp')} position="top">
                            <div className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded px-2 py-1 flex items-center gap-1.5 cursor-help">
                                <Zap size={10} className="text-amber-400" />
                                <span className="text-[10px] text-zinc-400 font-mono">{(agent.model_config?.temperature || 0.7).toFixed(1)} TEMP</span>
                            </div>
                        </Tooltip>
                        <Tooltip content={i18n.t('agent_manager.tooltip_credits')} position="top">
                            <div className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded px-2 py-1 flex items-center gap-1.5 cursor-help">
                                <DollarSign size={10} className="text-green-400" />
                                <span className="text-[10px] text-zinc-400 font-mono">
                                    ${(agent.cost_usd || 0).toFixed(3)} / ${agent.budget_usd || '0'}
                                </span>
                            </div>
                        </Tooltip>

                        {(agent.skills?.length || 0) > 0 && (
                            <Tooltip content={`${agent.skills?.join(', ')}`} position="top">
                                <div className="bg-blue-900/10 border border-blue-500/10 rounded px-2 py-1 flex items-center gap-1.5 cursor-help">
                                    <Code size={10} className="text-blue-400" />
                                    <span className="text-[10px] text-blue-400 font-mono uppercase font-bold tracking-tighter">{agent.skills?.length} SKILLS</span>
                                </div>
                            </Tooltip>
                        )}

                        {(agent.workflows?.length || 0) > 0 && (
                            <Tooltip content={`${agent.workflows?.join(', ')}`} position="top">
                                <div className="bg-amber-900/10 border border-amber-500/10 rounded px-2 py-1 flex items-center gap-1.5 cursor-help">
                                    <FileText size={10} className="text-amber-400" />
                                    <span className="text-[10px] text-amber-400 font-mono uppercase font-bold tracking-tighter">{agent.workflows?.length} WFK</span>
                                </div>
                            </Tooltip>
                        )}

                        {(agent.mcp_tools?.length || 0) > 0 && (
                            <Tooltip content={`${agent.mcp_tools?.join(', ')}`} position="top">
                                <div className="bg-cyan-900/10 border border-cyan-500/10 rounded px-2 py-1 flex items-center gap-1.5 cursor-help">
                                    <Terminal size={10} className="text-cyan-400" />
                                    <span className="text-[10px] text-cyan-400 font-mono uppercase font-bold tracking-tighter">{agent.mcp_tools?.length} MCP</span>
                                </div>
                            </Tooltip>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
};

export const Agent_Card_Memo = React.memo(Agent_Card, (prev_props, next_props) => {
    const p = prev_props.agent;
    const n = next_props.agent;
    return (
        p.status === n.status &&
        p.tokens_used === n.tokens_used &&
        p.cost_usd === n.cost_usd &&
        p.theme_color === n.theme_color &&
        p.name === n.name &&
        p.role === n.role &&
        p.model === n.model &&
        p.model_2 === n.model_2 &&
        p.model_3 === n.model_3 &&
        p.active_model_slot === n.active_model_slot &&
        p.model_config?.temperature === n.model_config?.temperature &&
        p.budget_usd === n.budget_usd &&
        p.category === n.category &&
        p.skills?.join() === n.skills?.join() &&
        p.workflows?.join() === n.workflows?.join() &&
        p.mcp_tools?.join() === n.mcp_tools?.join() &&
        p.metadata?.has_participated_in_swarm === n.metadata?.has_participated_in_swarm &&
        p.active_mission === n.active_mission &&
        p.current_task === n.current_task &&
        p.current_reasoning_turn === n.current_reasoning_turn &&
        p.reasoning_depth === n.reasoning_depth
    );
});

// Metadata: [Agent_Card]
