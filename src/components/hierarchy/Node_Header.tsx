/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Top-level identity and status bar for agent nodes. 
 * Handles role-switching dropdowns, valence-based glow pulses, and the high-priority Oversight/Health toggle system.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Role dropdown eviction during state updates, valence color interpolation errors, or Crown badge missing for Alpha-designated nodes.
 * - **Telemetry Link**: Search for `[Node_Header]` or `Role Selector` in browser logs.
 */

import React from 'react';
import { Crown, ChevronDown, Sliders, Zap, Brain, Shield } from 'lucide-react';
import { Tooltip } from '../ui';
import { get_department_icon, get_agent_status_styles, get_valence_color } from '../../utils/agent_uiutils';
import { use_dropdown_store } from '../../stores/dropdown_store';
import { i18n } from '../../i18n';
import type { Agent } from '../../types';

const STATUS_INDICATORS: Record<string, string> = {
    active: 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]',
    thinking: 'bg-green-500 shadow-[0_0_8px_rgba(59,130,246,0.5)] animate-pulse',
    coding: 'bg-green-500 shadow-[0_0_8px_rgba(59,130,246,0.5)] animate-pulse',
    speaking: 'bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.5)] animate-pulse',
    suspended: 'bg-zinc-600 shadow-[0_0_5px_rgba(82,82,91,0.5)]',
};

interface Node_Header_Props {
    agent: Agent;
    is_alpha?: boolean;
    is_active?: boolean;
    available_roles: string[];
    on_role_change?: (agent_id: string, new_role: string) => void;
    on_configure_click?: (agent_id: string) => void;
    has_oversight?: boolean;
    is_oversight_open?: boolean;
    on_oversight_toggle?: () => void;
    is_health_open?: boolean;
    on_health_toggle?: () => void;
}

export const Node_Header: React.FC<Node_Header_Props> = ({
    agent,
    is_alpha,
    is_active,
    available_roles,
    on_role_change,
    on_configure_click,
    has_oversight,
    is_oversight_open,
    on_oversight_toggle,
    is_health_open,
    on_health_toggle
}) => {
    const toggle_dropdown = use_dropdown_store(s => s.toggle_dropdown);
    const close_dropdown = use_dropdown_store(s => s.close_dropdown);
    const is_role_dropdown_open = use_dropdown_store(s => s.is_open(agent.id, 'role'));

    const dept_icon = get_department_icon(agent.department || '');
    const status_styles = get_agent_status_styles(agent.status);
    const agent_color = agent.theme_color || status_styles.hex;

    const role_badge_class =
        agent.department === 'Executive' ? 'text-amber-400 border-amber-900 bg-amber-900/10' :
            agent.department === 'Engineering' ? 'text-green-400 border-blue-900 bg-blue-900/10' :
                agent.department === 'Product' ? 'text-orange-400 border-orange-900 bg-orange-900/10' :
                    'text-zinc-400 border-zinc-800 bg-zinc-900';
    
    const failure_count = agent.failure_count || 0;
    const health_color = failure_count >= 3 ? 'text-red-500' : failure_count > 0 ? 'text-amber-500' : 'text-emerald-500';
    const health_tooltip = failure_count >= 3 ? i18n.t('throttled') : failure_count > 0 ? i18n.t('degraded') : i18n.t('healthy');

    return (
        <div className={`grid grid-cols-[min-content_1fr_min-content] gap-2 items-start ${is_role_dropdown_open ? 'z-50' : 'z-20'}`}>
            <div className={`
                w-8 h-8 rounded-lg flex items-center justify-center border transition-all relative shrink-0
                ${is_alpha ? 'bg-zinc-900 border-amber-500/50 shadow-[0_0_10px_rgba(245,158,11,0.2)]' : (agent.status !== 'offline' && agent.status !== 'idle' ? 'bg-zinc-900 border-white/10' : 'bg-zinc-950 border-white/5')}
            `}>
                {agent.status !== 'offline' && agent.status !== 'idle' && (
                    <div className="absolute inset-0 rounded-lg animate-ping opacity-20 border border-current" style={{ backgroundColor: 'transparent', borderColor: agent_color }} />
                )}
                {agent.valence !== undefined && (
                    <div 
                        className="absolute -inset-1 rounded-lg animate-pulse" 
                        style={{ 
                            border: `2px solid ${get_valence_color(agent.valence)}`, 
                            boxShadow: `0 0 10px ${get_valence_color(agent.valence)}60`,
                            zIndex: -1
                        }} 
                    />
                )}
                {is_alpha ? (
                    <Crown size={14} className="text-amber-500 animate-pulse" />
                ) : (
                    React.createElement(dept_icon, { size: 14, className: agent.status !== 'offline' && agent.status !== 'idle' ? '' : 'opacity-40' })
                )}
            </div>

            <div className="flex flex-col min-w-0 overflow-hidden">
                <div className="flex items-center gap-1.5 min-w-0">
                    <Tooltip content={i18n.t('agent_card.tooltip_full_id', { name: agent.name })} position="top">
                        <span className="font-bold text-zinc-100 text-xs tracking-tight leading-none truncate cursor-help">{agent.name}</span>
                    </Tooltip>
                    {is_alpha && <Crown size={10} className="text-amber-400 fill-amber-400/20 shrink-0" />}
                </div>
                <div className="relative shrink-0" role="presentation" onClick={(e) => e.stopPropagation()} onKeyDown={(e) => e.stopPropagation()}>
                    <Tooltip content={i18n.t('agent_card.tooltip_current_role', { role: agent.role })} position="top">
                        <button
                            onClick={(e) => { e.stopPropagation(); toggle_dropdown(agent.id, 'role'); }}
                            aria-haspopup="listbox"
                            aria-expanded={is_role_dropdown_open}
                            aria-label={i18n.t('agent_manager.aria_role_selector', { role: agent.role })}
                            className={`text-[11px] px-1.5 py-0.5 rounded border font-mono flex items-center gap-1 hover:bg-white/5 transition-colors cursor-pointer max-w-full ${role_badge_class}`}>
                            <span className="truncate">{agent.role.toUpperCase()}</span>
                            <ChevronDown size={8} className="opacity-70 shrink-0" />
                        </button>
                    </Tooltip>

                    {is_role_dropdown_open && (
                        <div className="absolute top-full left-0 mt-1 w-48 bg-zinc-900 border border-zinc-700 rounded-lg shadow-xl z-50 py-1 max-h-60 overflow-y-auto custom-scrollbar">
                            {available_roles.map((role) => (
                                <button
                                    key={role}
                                    onClick={() => {
                                        on_role_change?.(agent.id, role);
                                        close_dropdown();
                                    }}
                                    className={`w-full text-left px-3 py-1.5 text-xs hover:bg-zinc-800 transition-colors ${agent.role === role ? 'text-green-400 font-bold bg-blue-900/10' : 'text-zinc-300'}`}
                                >
                                    {role}
                                </button>
                            ))}
                        </div>
                    )}
                </div>
            </div>

            <div className="flex items-center gap-2 shrink-0">
                <div className="flex flex-col items-end gap-1">
                    {is_active && (
                        <div className="flex items-center gap-1 px-2 py-0.5 rounded bg-emerald-500/10 border border-emerald-500/20 text-[10px] font-bold text-emerald-400 uppercase tracking-widest animate-in fade-in zoom-in-95">
                            <Zap size={8} fill="currentColor" /> {i18n.t('agent_card.badge_active')}
                        </div>
                    )}
                    {is_alpha && has_oversight && (
                        <Tooltip content={is_oversight_open ? i18n.t('oversight.btn_hide') : i18n.t('oversight.btn_show')} position="top">
                            <button
                                onClick={(e) => { e.stopPropagation(); on_oversight_toggle?.(); }}
                                aria-label={is_oversight_open ? i18n.t('oversight.btn_hide') : i18n.t('oversight.btn_show')}
                                className={`
                                    flex items-center gap-1.5 px-2 py-0.5 rounded border text-[10px] font-bold uppercase tracking-widest transition-all duration-300
                                    ${is_oversight_open
                                        ? 'bg-green-500/20 border-green-500/40 text-green-400 shadow-[0_0_15px_rgba(59,130,246,0.2)]'
                                        : 'bg-zinc-900 border-zinc-700 text-zinc-500 hover:border-green-500/50 hover:text-green-400 shadow-[0_0_10px_rgba(59,130,246,0.1)]'
                                    }
                                `}
                            >
                                <Brain size={10} className={!is_oversight_open ? 'animate-pulse' : ''} />
                                <span>{i18n.t('oversight.mod_req_label')}</span>
                            </button>
                        </Tooltip>
                    )}
                </div>
                <Tooltip content={health_tooltip} position="top">
                    <button
                        onClick={(e) => { e.stopPropagation(); on_health_toggle?.(); }}
                        aria-label={health_tooltip}
                        aria-expanded={is_health_open}
                        className={`p-1 rounded hover:bg-zinc-800 transition-colors ${is_health_open ? 'bg-zinc-800 ' + health_color : 'text-zinc-600 hover:' + health_color}`}
                    >
                        <Shield size={12} fill={is_health_open ? 'currentColor' : 'none'} className={failure_count >= 3 ? 'animate-pulse' : ''} />
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('agent_card.tooltip_configure')} position="top">
                    <button
                        onClick={(e) => { e.stopPropagation(); on_configure_click?.(agent.id); }}
                        aria-label={i18n.t('agent_card.tooltip_configure')}
                        className="p-1 rounded hover:bg-blue-900/20 text-zinc-600 hover:text-green-400 transition-colors"
                    >
                        <Sliders size={12} />
                    </button>
                </Tooltip>
                <div className={`w-1.5 h-1.5 rounded-full mt-1 transition-all duration-500 ${STATUS_INDICATORS[agent.status] || 'bg-zinc-700'}`} />
            </div>
        </div>
    );
};


// Metadata: [Node_Header]
