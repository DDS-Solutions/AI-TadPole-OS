/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / Cluster_Card
 * - **Primary Entrypoints**: `Cluster_Card`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React, { useState, useRef, useEffect } from 'react';
import { Zap, Trash2, Shield } from 'lucide-react';
import { Tooltip } from '../ui';
import { get_theme_colors } from '../../utils/agent_uiutils';
import type { Mission_Cluster } from '../../stores/workspace_store';
import type { Agent } from '../../types';
import { i18n } from '../../i18n';

export interface ClusterCardProps {
    cluster: Mission_Cluster;
    is_selected: boolean;
    agent_map: Map<string, Agent>;
    departments: string[];
    on_select_cluster: (id: string) => void;
    on_delete_cluster: (id: string) => void;
    on_toggle_active: (id: string) => void;
    on_update_department: (id: string, dept: Mission_Cluster['department']) => void;
    on_update_budget: (id: string, budget: number) => void;
    toggle_cluster_privacy?: (id: string) => void | Promise<void>;
}

export const Cluster_Card: React.FC<ClusterCardProps> = ({
    cluster,
    is_selected,
    agent_map,
    departments,
    on_select_cluster,
    on_delete_cluster,
    on_toggle_active,
    on_update_department,
    on_update_budget,
    toggle_cluster_privacy
}) => {
    const theme = get_theme_colors(cluster.theme);
    const is_active_cluster = cluster.is_active;
    const is_multi_role_team = Boolean(
        cluster.is_team || 
        cluster.team_badge || 
        (cluster.collaborators && cluster.collaborators.length > 1)
    );

    const [editing_budget, set_editing_budget] = useState<string | null>(null);
    const debounce_timer = useRef<ReturnType<typeof setTimeout> | null>(null);

    useEffect(() => {
        return () => {
            if (debounce_timer.current) clearTimeout(debounce_timer.current);
        };
    }, []);

    const handle_budget_change = (value: string) => {
        set_editing_budget(value);
        if (debounce_timer.current) clearTimeout(debounce_timer.current);
        debounce_timer.current = setTimeout(() => {
            on_update_budget(cluster.id, parseFloat(value) || 0);
        }, 800);
    };

    const handle_budget_blur = () => {
        set_editing_budget(null);
    };

    return (
        <div
            role="button"
            aria-selected={is_selected}
            tabIndex={0}
            onClick={() => on_select_cluster(cluster.id)}
            onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    on_select_cluster(cluster.id);
                }
            }}
            className={`
                group relative p-3 rounded-xl border transition-all cursor-pointer overflow-hidden
                ${is_selected ? `${theme.bg} ${theme.border} shadow-lg ${theme.glow} translate-y-[-2px]` : 'bg-[color:var(--color-surface)] border-[color:var(--color-border)] hover:border-zinc-700 hover:translate-y-[-1px]'}
                ${is_active_cluster ? 'ring-1 ring-emerald-500/30' : ''}
            `}
        >
            {is_active_cluster && (
                <div className="absolute inset-0 bg-emerald-500/5 animate-pulse pointer-events-none" />
            )}

            <div className="flex justify-between items-start mb-2 relative z-10 gap-2">
                <div className="flex flex-col min-w-0 flex-1">
                    <span className={`text-xs font-bold truncate ${is_selected ? theme.text : 'text-zinc-300'}`}>
                        {cluster.name}
                    </span>
                    <div className="flex flex-col gap-1 mt-2">
                        <span className="text-[9px] uppercase text-zinc-600 font-bold tracking-wider">{i18n.t('missions.label_dept')}</span>
                        <div className="flex items-center gap-2">
                            {/* Department selector container with explicit relative bounding box */}
                            <div className="relative inline-flex items-center min-w-[70px] min-h-[22px] group/dept">
                                <span className="text-[11px] px-1.5 py-0.5 rounded bg-[color:var(--color-surface)] border border-[color:var(--color-border)] text-zinc-500 font-mono uppercase group-hover/dept:text-zinc-300 group-hover/dept:border-zinc-700 transition-colors w-full text-center">
                                    {cluster.department}
                                </span>
                                <Tooltip content={i18n.t('missions.tooltip_reassign_dept')} position="top">
                                    <select
                                        className="absolute inset-0 w-full h-full opacity-0 cursor-pointer bg-[color:var(--color-background)] text-zinc-300"
                                        aria-label={i18n.t('missions.tooltip_reassign_dept')}
                                        value={cluster.department}
                                        onClick={(e) => e.stopPropagation()}
                                        onChange={(e) => {
                                            e.stopPropagation();
                                            on_update_department(cluster.id, e.target.value as Mission_Cluster['department']);
                                        }}
                                        style={{ colorScheme: 'dark' }}
                                    >
                                        {Array.from(new Set([cluster.department, ...departments])).filter(Boolean).map((dept) => (
                                            <option key={dept} value={dept} className="bg-[color:var(--color-background)] text-zinc-300">{dept}</option>
                                        ))}
                                    </select>
                                </Tooltip>
                            </div>
                            <span className="text-[10px] text-zinc-600 font-mono">| {(cluster.collaborators || []).length} {i18n.t('missions.label_nodes')}</span>
                            <Tooltip content={i18n.t('missions.tooltip_treasury')} position="top">
                                <div className="flex items-center gap-1.5 px-3 py-1 rounded bg-green-500/10 border border-green-500/20 hover:border-green-500/40 transition-all cursor-text">
                                    <span className="text-xs text-green-400 font-mono font-bold">{i18n.t('common_units.currency')}</span>
                                    <input
                                        type="number"
                                        step="0.01"
                                        className="w-20 bg-transparent border-none p-0 text-xs text-green-400 font-mono font-bold focus:ring-0 focus:outline-none [appearance:textfield]"
                                        aria-label={i18n.t('missions.label_budget')}
                                        value={editing_budget !== null ? editing_budget : (cluster.budget_usd || 0).toString()}
                                        onChange={(e) => {
                                            e.stopPropagation();
                                            handle_budget_change(e.target.value);
                                        }}
                                        onClick={(e) => e.stopPropagation()}
                                        onBlur={handle_budget_blur}
                                    />
                                </div>
                            </Tooltip>
                        </div>
                    </div>

                    {/* Team Badge rendered under the budget dollar value */}
                    {is_multi_role_team && (
                        <div className="mt-2 pt-1 border-t border-zinc-800/60 flex items-center justify-between">
                            <span className="text-[9px] font-mono font-bold text-zinc-500 uppercase tracking-wider">Swarm Team</span>
                            <span className="px-1.5 py-0.5 rounded text-[9px] font-mono font-bold bg-cyan-500/20 text-cyan-300 border border-cyan-500/40">
                                🏷️ {(cluster.team_badge || 'MULTI-ROLE').toUpperCase()}
                            </span>
                        </div>
                    )}
                </div>

                <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <Tooltip content={cluster.privacy_mode ? 'Air-Gap Active (100% Local Only)' : 'Enable Air-Gap (Local Only)'} position="top">
                        <button
                            onClick={(e) => {
                                e.stopPropagation();
                                if (toggle_cluster_privacy) {
                                    void toggle_cluster_privacy(cluster.id);
                                }
                            }}
                            className={`p-1 rounded hover:bg-zinc-800 transition-colors ${cluster.privacy_mode ? 'text-green-400 bg-green-500/10' : 'text-zinc-600'}`}
                        >
                            <Shield size={12} className={cluster.privacy_mode ? 'animate-pulse' : ''} />
                        </button>
                    </Tooltip>
                    <Tooltip content={is_active_cluster ? i18n.t('missions.tooltip_deactivate') : i18n.t('missions.tooltip_activate')} position="top">
                        <button
                            onClick={(e) => { e.stopPropagation(); on_toggle_active(cluster.id); }}
                            className={`p-1 rounded hover:bg-zinc-800 transition-colors ${is_active_cluster ? 'text-emerald-400' : 'text-zinc-600'}`}
                        >
                            <Zap size={12} fill={is_active_cluster ? "currentColor" : "none"} />
                        </button>
                    </Tooltip>
                    <Tooltip content={i18n.t('missions.tooltip_delete')} position="top">
                        <button
                            onClick={(e) => { e.stopPropagation(); on_delete_cluster(cluster.id); }}
                            className="p-1 rounded hover:bg-red-900/20 text-zinc-600 hover:text-red-400 transition-colors"
                        >
                            <Trash2 size={12} />
                        </button>
                    </Tooltip>
                </div>
            </div>

            {/* O(1) Fast Agent Avatar Lookup */}
            <div className="flex -space-x-2 overflow-hidden relative z-10 p-1">
                {(cluster.collaborators || []).slice(0, 5).map(id => {
                    const agent = agent_map.get(id);
                    const is_alpha = cluster.alpha_id === id;
                    const avatar_color = agent?.theme_color || (is_alpha ? '#f59e0b' : undefined);
                    return (
                        <Tooltip key={id} content={is_alpha ? i18n.t('missions.tooltip_alpha') : i18n.t('missions.tooltip_subordinate')} position="top">
                            <div
                                className="w-7 h-7 rounded-full border-2 border-black flex items-center justify-center transition-colors relative"
                                style={{ backgroundColor: avatar_color ? `${avatar_color}30` : '#27272a', borderColor: avatar_color || '#3f3f46' }}
                            >
                                <span className="text-[10px] font-bold" style={{ color: avatar_color || '#a1a1aa' }}>
                                    {agent?.name[0] || '?'}
                                </span>
                                {is_alpha && (
                                    <div className="absolute -top-0.5 -right-0.5 w-2 h-2 rounded-full bg-amber-400 border border-black shadow-[0_0_5px_rgba(245,158,11,0.8)]" />
                                )}
                            </div>
                        </Tooltip>
                    );
                })}
                {(cluster.collaborators || []).length > 5 && (
                    <div className="w-7 h-7 rounded-full border-2 border-black bg-[color:var(--color-surface)] flex items-center justify-center text-[10px] font-bold text-zinc-600">
                        +{(cluster.collaborators || []).length - 5}
                    </div>
                )}
            </div>
        </div>
    );
};

