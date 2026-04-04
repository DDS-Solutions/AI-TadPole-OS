import React, { useState } from 'react';
import { Plus, Zap, Trash2, Target } from 'lucide-react';
import { Tooltip } from '../ui';
import { get_theme_colors } from '../../utils/agent_uiutils';
import type { Mission_Cluster } from '../../stores/workspace_store';
import type { Agent } from '../../types';
import { i18n } from '../../i18n';

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
    const [show_create_modal, set_show_create_modal] = useState(false);
    const [new_mission_budget, set_new_mission_budget] = useState('1.00');
    const [new_cluster, set_new_cluster] = useState({
        name: '',
        department: 'Engineering' as Mission_Cluster['department'],
        theme: 'blue' as const,
        path: '/workspaces/new-mission',
        collaborators: [] as string[]
    });

    // Local state for budget editing to prevent focus loss on re-render
    const [editing_budgets, set_editing_budgets] = useState<Record<string, string>>({});

    const handle_budget_change = (id: string, value: string) => {
        set_editing_budgets(prev => ({ ...prev, [id]: value }));

        // Debounce the store update
        const timeout_id = (globalThis as unknown as Record<string, ReturnType<typeof setTimeout>>)[`_budgetTimeout_${id}`];
        if (timeout_id) clearTimeout(timeout_id);
        (globalThis as unknown as Record<string, ReturnType<typeof setTimeout>>)[`_budgetTimeout_${id}`] = setTimeout(() => {
            on_update_budget(id, parseFloat(value) || 0);
        }, 800);
    };

    const handle_create = () => {
        on_create_cluster({
            ...new_cluster,
            budget_usd: parseFloat(new_mission_budget)
        });
        set_show_create_modal(false);
    };

    return (
        <div className="md:col-span-1 flex flex-col gap-4 overflow-y-auto custom-scrollbar pr-2 pl-1">
            <div className="flex items-center justify-between px-2">
                <h3 className="sovereign-header-text">{i18n.t('missions.header_active_clusters')}</h3>
                <Tooltip content={i18n.t('missions.tooltip_create_cluster')} position="left">
                    <button
                        onClick={() => set_show_create_modal(true)}
                        className="p-1 px-2 rounded-lg border border-zinc-800 bg-zinc-900 text-xs font-bold text-zinc-400 hover:text-white hover:border-zinc-700 transition-all flex items-center gap-1"
                    >
                        <Plus size={10} /> {i18n.t('missions.btn_new_mission')}
                    </button>
                </Tooltip>
            </div>

            {show_create_modal && (
                <div className="sovereign-card animate-in slide-in-from-top-2 border-blue-500/30 bg-blue-500/5">
                    <h4 className="sovereign-header-text !text-blue-400 mb-3">{i18n.t('missions.header_create_cluster')}</h4>
                    <div className="space-y-3">
                        <input
                            className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-xs text-zinc-200"
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
                                <span className="absolute left-2 top-1/2 -translate-y-1/2 text-zinc-500 font-mono text-[10px]">$</span>
                                <input
                                    type="number"
                                    step="0.01"
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 pl-4 text-xs text-zinc-200 font-mono"
                                    placeholder="0.00"
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
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-xs text-zinc-300"
                                    aria-label={i18n.t('missions.label_dept')}
                                    value={new_cluster.department}
                                    onChange={e => set_new_cluster({ ...new_cluster, department: e.target.value as Mission_Cluster['department'] })}
                                >
                                    <option value="Executive">Executive</option>
                                    <option value="Engineering">Engineering</option>
                                    <option value="Operations">Operations</option>
                                    <option value="Product">Product</option>
                                    <option value="Marketing">Marketing</option>
                                    <option value="Sales">Sales</option>
                                    <option value="Design">Design</option>
                                    <option value="Research">Research</option>
                                    <option value="Support">Support</option>
                                    <option value="Quality Assurance">Quality Assurance</option>
                                    <option value="Intelligence">Intelligence</option>
                                    <option value="Finance">Finance</option>
                                    <option value="Growth">Growth</option>
                                    <option value="Success">Success</option>
                                </select>
                            </div>
                            <button
                                onClick={handle_create}
                                disabled={!new_cluster.name}
                                className="h-[34px] px-3 bg-blue-600 text-white rounded text-xs font-bold uppercase disabled:opacity-50"
                            >
                                {i18n.t('missions.btn_create')}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {clusters.map(cluster => {
                const is_selected = selected_cluster_id === cluster.id;
                const theme = get_theme_colors(cluster.theme);
                const is_active_cluster = cluster.is_active;
 
                return (
                    <div
                        key={cluster.id}
                        role="button"
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
                            ${is_selected ? `${theme.bg} ${theme.border} shadow-lg ${theme.glow} translate-y-[-2px]` : 'bg-zinc-900 border-zinc-800 hover:border-zinc-700 hover:translate-y-[-1px]'}
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
                                        <div className="relative group/dept">
                                            <span className="text-[11px] px-1.5 py-0.5 rounded bg-zinc-900 border border-zinc-800 text-zinc-500 font-mono uppercase group-hover/dept:text-zinc-300 group-hover/dept:border-zinc-700 transition-colors">
                                                {cluster.department}
                                            </span>
                                            <Tooltip content={i18n.t('missions.tooltip_reassign_dept')} position="top">
                                                <select
                                                    className="absolute inset-0 w-full h-full opacity-0 cursor-pointer bg-zinc-950 text-zinc-300"
                                                    aria-label={i18n.t('missions.tooltip_reassign_dept')}
                                                    value={cluster.department}
                                                    onClick={(e) => e.stopPropagation()}
                                                    onChange={(e) => {
                                                        e.stopPropagation();
                                                        on_update_department(cluster.id, e.target.value as Mission_Cluster['department']);
                                                    }}
                                                    style={{ colorScheme: 'dark' }}
                                                >
                                                    {['Executive', 'Engineering', 'Operations', 'Product', 'Marketing', 'Sales', 'Design', 'Research', 'Support', 'Quality Assurance', 'Intelligence', 'Finance', 'Growth', 'Success'].map(dept => (
                                                        <option key={dept} value={dept} className="bg-zinc-950 text-zinc-300">{dept}</option>
                                                    ))}
                                                </select>
                                            </Tooltip>
                                        </div>
                                        <span className="text-[10px] text-zinc-600 font-mono">| {cluster.collaborators.length} {i18n.t('missions.label_nodes')}</span>
                                        <Tooltip content={i18n.t('missions.tooltip_treasury')} position="top">
                                            <div className="flex items-center gap-1.5 px-3 py-1 rounded bg-blue-500/10 border border-blue-500/20 hover:border-blue-500/40 transition-all cursor-text">
                                                <span className="text-xs text-blue-400 font-mono font-bold">$</span>
                                                <input
                                                    type="number"
                                                    step="0.01"
                                                    className="w-20 bg-transparent border-none p-0 text-xs text-blue-400 font-mono font-bold focus:ring-0 focus:outline-none [appearance:textfield]"
                                                    aria-label={i18n.t('missions.label_budget')}
                                                    value={editing_budgets[cluster.id] !== undefined ? editing_budgets[cluster.id] : (cluster.budget_usd || 0).toString()}
                                                    onChange={(e) => {
                                                        e.stopPropagation();
                                                        handle_budget_change(cluster.id, e.target.value);
                                                    }}
                                                    onClick={(e) => e.stopPropagation()}
                                                    onBlur={() => {
                                                        // Ensure clean sync on blur
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
                            </div>

                            <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
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

                        <div className="flex -space-x-2 overflow-hidden relative z-10 p-1">
                            {cluster.collaborators.slice(0, 5).map(id => {
                                const agent = agents.find(a => a.id === id);
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
                            {cluster.collaborators.length > 5 && (
                                <div className="w-7 h-7 rounded-full border-2 border-black bg-zinc-900 flex items-center justify-center text-[10px] font-bold text-zinc-600">
                                    +{cluster.collaborators.length - 5}
                                </div>
                            )}
                        </div>
                    </div>
                );
            })}
        </div>
    );
};
