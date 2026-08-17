/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Global mission cluster navigator and treasury manager. 
 * Orchestrates cluster selection, budget/budget-utilization editing, and department classification across the OS.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Budget edit debounce starvation (800ms lag), cluster name truncation in compact sidebar, or focus loss during rapid department switching.
 * - **Telemetry Link**: Search for `[Cluster_Sidebar]` or Budget_Update in browser logs.
 */

import React, { useState, useRef, useMemo } from 'react';
import { Plus, Zap, Trash2, Target, Shield, Sliders, Layers, Check } from 'lucide-react';
import { Tooltip } from '../ui';
import { get_theme_colors } from '../../utils/agent_uiutils';
import { use_workspace_store, type Mission_Cluster, type Team_Cluster_Preset } from '../../stores/workspace_store';
import type { Agent } from '../../types';
import { i18n } from '../../i18n';
import { Cluster_Manager_Modal } from './Cluster_Manager_Modal';

interface ClusterSidebarProps {
    clusters: Mission_Cluster[];
    selected_cluster_id: string | null;
    agents: Agent[];
    on_select_cluster: (id: string) => void;
    on_create_cluster: (cluster: Partial<Mission_Cluster>) => void;
    on_delete_cluster: (id: string) => void;
    on_toggle_active: (id: string) => void;
    on_update_department: (id: string, dept: Mission_Cluster['department']) => void;
    on_update_budget: (id: string, budget: number) => void;
}

const INITIAL_NEW_CLUSTER_STATE = {
    name: '',
    department: 'Engineering' as Mission_Cluster['department'],
    theme: 'blue' as Mission_Cluster['theme'],
    path: '/workspaces/new-mission',
    collaborators: [] as string[],
    privacy_mode: false,
    is_team: false,
    team_badge: ''
};

export const Cluster_Sidebar: React.FC<ClusterSidebarProps> = ({
    clusters,
    selected_cluster_id,
    agents,
    on_select_cluster,
    on_create_cluster,
    on_delete_cluster,
    on_toggle_active,
    on_update_department,
    on_update_budget
}) => {
    const toggle_cluster_privacy = use_workspace_store(state => state.toggle_cluster_privacy);
    const team_presets = use_workspace_store(state => state.team_presets || []);

    const [show_manager_modal, set_show_manager_modal] = useState(false);
    const [show_create_drawer, set_show_create_drawer] = useState(false);
    const [show_preset_picker, set_show_preset_picker] = useState(false);

    const [new_mission_budget, set_new_mission_budget] = useState('1.00');
    const [new_cluster, set_new_cluster] = useState(INITIAL_NEW_CLUSTER_STATE);

    // O(1) Fast Agent Lookup Map
    const agent_map = useMemo(() => {
        return new Map(agents.map(a => [a.id, a]));
    }, [agents]);

    // Local state for budget editing to prevent focus loss on re-render
    const [editing_budgets, set_editing_budgets] = useState<Record<string, string>>({});
    const timeoutRef = useRef<Record<string, ReturnType<typeof setTimeout>>>({});

    const handle_budget_change = (id: string, value: string) => {
        set_editing_budgets(prev => ({ ...prev, [id]: value }));

        if (timeoutRef.current[id]) clearTimeout(timeoutRef.current[id]);
        
        timeoutRef.current[id] = setTimeout(() => {
            on_update_budget(id, parseFloat(value) || 0);
        }, 800);
    };

    const reset_create_form = () => {
        set_new_cluster(INITIAL_NEW_CLUSTER_STATE);
        set_new_mission_budget('1.00');
        set_show_create_drawer(false);
        set_show_preset_picker(false);
    };

    const handle_select_preset = (preset: Team_Cluster_Preset) => {
        set_new_cluster({
            name: preset.name,
            department: preset.department,
            theme: preset.theme,
            path: `/workspaces/${preset.id}-${crypto.randomUUID()}`,
            collaborators: preset.collaborators || [],
            privacy_mode: false,
            is_team: true,
            team_badge: preset.badge_label
        });
        set_new_mission_budget(preset.default_budget_usd.toString());
        set_show_preset_picker(false);
    };

    const toggle_collaborator = (agent_id: string) => {
        set_new_cluster(prev => ({
            ...prev,
            collaborators: prev.collaborators.includes(agent_id)
                ? prev.collaborators.filter(id => id !== agent_id)
                : [...prev.collaborators, agent_id]
        }));
    };

    const handle_create = () => {
        on_create_cluster({
            ...new_cluster,
            budget_usd: parseFloat(new_mission_budget) || 1.00
        });
        reset_create_form();
    };

    return (
        <div className="md:col-span-1 flex flex-col gap-3 h-full min-h-0 overflow-hidden pr-2 pl-1">
            <Cluster_Manager_Modal
                isOpen={show_manager_modal}
                onClose={() => set_show_manager_modal(false)}
                agents={agents}
            />

            <div className="flex items-center justify-between px-2 shrink-0 gap-1">
                <h3 className="sovereign-header-text truncate">{i18n.t('missions.header_active_clusters')}</h3>
                <div className="flex items-center gap-1 shrink-0">
                    <Tooltip content="Manage Mode-Switched Utility Clusters & Team Presets" position="left">
                        <button
                            onClick={() => set_show_manager_modal(true)}
                            className="p-1 px-2 rounded-lg border border-[color:var(--color-border)] bg-[color:var(--color-surface)] text-xs font-bold text-cyan-400 hover:text-cyan-300 hover:border-cyan-500/50 transition-all flex items-center gap-1"
                        >
                            <Sliders size={10} /> CLUSTERS
                        </button>
                    </Tooltip>
                    <Tooltip content={i18n.t('missions.tooltip_create_cluster')} position="left">
                        <button
                            onClick={() => set_show_create_drawer(prev => !prev)}
                            className="p-1 px-2 rounded-lg border border-[color:var(--color-border)] bg-[color:var(--color-surface)] text-xs font-bold text-zinc-400 hover:text-white hover:border-zinc-700 transition-all flex items-center gap-1"
                        >
                            <Plus size={10} /> {i18n.t('missions.btn_new_mission')}
                        </button>
                    </Tooltip>
                </div>
            </div>

            {/* Inline Expanding Cluster Creation Drawer */}
            {show_create_drawer && (
                <div className="shrink-0 sovereign-card animate-in slide-in-from-top-2 border-green-500/30 bg-green-500/5 shadow-xl relative z-20">
                    <h4 className="sovereign-header-text !text-green-400 mb-3">{i18n.t('missions.header_create_cluster')}</h4>
                    <div className="space-y-3">
                        <input
                            className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded p-2 text-xs text-zinc-200"
                            placeholder={i18n.t('missions.placeholder_name')}
                            aria-label={i18n.t('missions.placeholder_name')}
                            value={new_cluster.name}
                            onChange={e => set_new_cluster({ ...new_cluster, name: e.target.value })}
                        />
                        <div className="space-y-1">
                            <div className="flex items-center justify-between">
                                <label className="text-[10px] uppercase text-zinc-500 font-bold tracking-wider">{i18n.t('missions.label_budget')}</label>
                                <Tooltip content={i18n.t('missions.tooltip_budget')} position="top">
                                    <Target size={10} className="text-zinc-600 cursor-help" />
                                </Tooltip>
                            </div>
                            <div className="relative">
                                <span className="absolute left-2 top-1/2 -translate-y-1/2 text-zinc-500 font-mono text-[10px]">{i18n.t('common_units.currency')}</span>
                                <input
                                    type="number"
                                    step="0.01"
                                    className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded p-2 pl-4 text-xs text-zinc-200 font-mono"
                                    placeholder={i18n.t('common_units.placeholder_budget')}
                                    aria-label={i18n.t('missions.label_budget')}
                                    value={new_mission_budget}
                                    onChange={e => set_new_mission_budget(e.target.value)}
                                />
                            </div>
                        </div>
                        <div className="flex gap-2 items-end">
                            <div className="flex-1 space-y-1">
                                <label className="text-[10px] uppercase text-zinc-500 font-bold tracking-wider">{i18n.t('missions.label_dept')}</label>
                                <select
                                    className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded p-2 text-xs text-zinc-300"
                                    aria-label={i18n.t('missions.label_dept')}
                                    value={new_cluster.department}
                                    onChange={e => set_new_cluster({ ...new_cluster, department: e.target.value as Mission_Cluster['department'] })}
                                >
                                    {Object.entries(i18n.t('common.departments', { returnObjects: true }) as Record<string, string>).map(([key, label]) => (
                                        <option key={key} value={label} className="bg-[color:var(--color-background)] text-zinc-300">{label}</option>
                                    ))}
                                </select>
                            </div>
                        </div>
                        <div className="flex items-center justify-between p-2 rounded bg-zinc-900/60 border border-zinc-800">
                            <div className="flex items-center gap-1.5 text-xs text-zinc-300 font-bold">
                                <Shield size={12} className={new_cluster.privacy_mode ? 'text-green-400 animate-pulse' : 'text-zinc-500'} />
                                <span>Local-Only Air-Gap Mode</span>
                            </div>
                            <input
                                type="checkbox"
                                className="w-4 h-4 rounded border-zinc-700 bg-zinc-800 text-green-500 cursor-pointer"
                                checked={!!new_cluster.privacy_mode}
                                onChange={e => set_new_cluster({ ...new_cluster, privacy_mode: e.target.checked })}
                            />
                        </div>

                        {/* ADD TEAM CLUSTER Preset Accordion */}
                        <div className="pt-1">
                            <button
                                type="button"
                                onClick={() => set_show_preset_picker(prev => !prev)}
                                className="w-full py-1.5 px-2 rounded border border-cyan-500/40 bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-300 text-xs font-bold transition-all flex items-center justify-center gap-1.5"
                            >
                                <Layers size={12} /> {show_preset_picker ? 'Hide Team Cluster Presets' : '+ ADD TEAM CLUSTER PRESET'}
                            </button>

                            {show_preset_picker && (
                                <div className="mt-2 p-2 bg-zinc-900/80 border border-cyan-500/30 rounded-lg space-y-2 max-h-40 overflow-y-auto">
                                    <span className="text-[10px] uppercase font-bold text-cyan-400 tracking-wider block mb-1">
                                        Select Predefined Utility Cluster
                                    </span>
                                    {team_presets.map(preset => (
                                        <button
                                            type="button"
                                            key={preset.id}
                                            onClick={() => handle_select_preset(preset)}
                                            className="w-full text-left p-2 rounded bg-zinc-800/80 hover:bg-cyan-900/30 border border-zinc-700 hover:border-cyan-500/50 transition-all flex items-center justify-between group"
                                        >
                                            <div>
                                                <div className="flex items-center gap-1.5">
                                                    <span className="text-[10px] font-mono text-cyan-300 font-bold">🏷️ {preset.badge_label}</span>
                                                    <span className="text-xs font-bold text-white group-hover:text-cyan-200">{preset.name}</span>
                                                </div>
                                                <p className="text-[10px] text-zinc-400 line-clamp-1">{preset.description}</p>
                                            </div>
                                            <span className="text-[10px] font-mono text-green-400 font-bold shrink-0 ml-2">
                                                ${preset.default_budget_usd}
                                            </span>
                                        </button>
                                    ))}
                                </div>
                            )}
                        </div>

                        {/* Editable Agent Collaborators Checkbox List */}
                        <div className="space-y-1">
                            <label className="text-[10px] uppercase text-zinc-400 font-bold tracking-wider block">
                                Multi-Role Collaborators ({new_cluster.collaborators.length} Selected)
                            </label>
                            <div className="flex flex-wrap gap-1 max-h-28 overflow-y-auto p-1.5 bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded">
                                {agents.map(agent => {
                                    const is_checked = new_cluster.collaborators.includes(agent.id);
                                    return (
                                        <button
                                            type="button"
                                            key={agent.id}
                                            onClick={() => toggle_collaborator(agent.id)}
                                            className={`px-1.5 py-0.5 rounded text-[10px] transition-all flex items-center gap-1 ${
                                                is_checked
                                                    ? 'bg-cyan-600/80 text-white font-medium border border-cyan-500'
                                                    : 'bg-zinc-800 text-zinc-400 border border-zinc-700 hover:text-zinc-200'
                                            }`}
                                        >
                                            {is_checked && <Check size={8} />}
                                            {agent.name}
                                        </button>
                                    );
                                })}
                            </div>
                        </div>

                        <div className="flex justify-end gap-2 pt-1">
                            <button
                                type="button"
                                onClick={reset_create_form}
                                className="h-[34px] px-3 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded text-xs font-bold uppercase transition-colors"
                            >
                                Cancel
                            </button>
                            <button
                                type="button"
                                onClick={handle_create}
                                disabled={!new_cluster.name}
                                className="h-[34px] px-4 bg-green-600 hover:bg-green-500 text-white rounded text-xs font-bold uppercase disabled:opacity-50 transition-colors shadow-md shadow-green-950/40"
                            >
                                {i18n.t('missions.btn_create')}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Dedicated scrollable container for Cluster Cards */}
            <div className="flex-1 overflow-y-auto custom-scrollbar space-y-3 min-h-0 pr-1 pb-4">
                {clusters.map(cluster => {
                    const is_selected = selected_cluster_id === cluster.id;
                    const theme = get_theme_colors(cluster.theme);
                    const is_active_cluster = cluster.is_active;
                    const is_multi_role_team = Boolean(
                        cluster.is_team || 
                        cluster.team_badge || 
                        (cluster.collaborators && cluster.collaborators.length > 1)
                    );

                    return (
                        <div
                            key={cluster.id}
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
                                                        {Object.entries(i18n.t('common.departments', { returnObjects: true }) as Record<string, string>).map(([key, label]) => (
                                                            <option key={key} value={label} className="bg-[color:var(--color-background)] text-zinc-300">{label}</option>
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
                                                        value={editing_budgets[cluster.id] !== undefined ? editing_budgets[cluster.id] : (cluster.budget_usd || 0).toString()}
                                                        onChange={(e) => {
                                                            e.stopPropagation();
                                                            handle_budget_change(cluster.id, e.target.value);
                                                        }}
                                                        onClick={(e) => e.stopPropagation()}
                                                        onBlur={() => {
                                                            set_editing_budgets(prev => {
                                                                const next = { ...prev };
                                                                delete next[cluster.id];
                                                                return next;
                                                            });
                                                        }}
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
                                                    toggle_cluster_privacy(cluster.id);
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
                })}
            </div>
        </div>
    );
};
