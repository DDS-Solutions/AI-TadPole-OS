import React, { useState } from 'react';
import { Plus, Zap, Trash2, Target } from 'lucide-react';
import { Tooltip } from '../ui';
import { getThemeColors } from '../../utils/agentUIUtils';
import type { MissionCluster } from '../../stores/workspaceStore';
import type { Agent } from '../../types';
import { i18n } from '../../i18n';

interface ClusterSidebarProps {
    clusters: MissionCluster[];
    selectedClusterId: string | null;
    agents: Agent[];
    onSelectCluster: (id: string) => void;
    onCreateCluster: (cluster: Partial<MissionCluster>) => void;
    onDeleteCluster: (id: string) => void;
    onToggleActive: (id: string) => void;
    onUpdateDepartment: (id: string, dept: MissionCluster['department']) => void;
    onUpdateBudget: (id: string, budget: number) => void;
}

export const ClusterSidebar: React.FC<ClusterSidebarProps> = ({
    clusters,
    selectedClusterId,
    agents,
    onSelectCluster,
    onCreateCluster,
    onDeleteCluster,
    onToggleActive,
    onUpdateDepartment,
    onUpdateBudget
}) => {
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [newMissionBudget, setNewMissionBudget] = useState('1.00');
    const [newCluster, setNewCluster] = useState({
        name: '',
        department: 'Engineering' as MissionCluster['department'],
        theme: 'blue' as const,
        path: '/workspaces/new-mission',
        collaborators: [] as string[]
    });

    // Local state for budget editing to prevent focus loss on re-render
    const [editingBudgets, setEditingBudgets] = useState<Record<string, string>>({});

    const handleBudgetChange = (id: string, value: string) => {
        setEditingBudgets(prev => ({ ...prev, [id]: value }));

        // Debounce the store update
        const timeoutId = (globalThis as unknown as Record<string, ReturnType<typeof setTimeout>>)[`_budgetTimeout_${id}`];
        if (timeoutId) clearTimeout(timeoutId);
        (globalThis as unknown as Record<string, ReturnType<typeof setTimeout>>)[`_budgetTimeout_${id}`] = setTimeout(() => {
            onUpdateBudget(id, parseFloat(value) || 0);
        }, 800);
    };

    const handleCreate = () => {
        onCreateCluster({
            ...newCluster,
            budgetUsd: parseFloat(newMissionBudget)
        });
        setShowCreateModal(false);
    };

    return (
        <div className="md:col-span-1 flex flex-col gap-4 overflow-y-auto custom-scrollbar pr-2 pl-1">
            <div className="flex items-center justify-between px-2">
                <h3 className="text-xs font-bold text-zinc-600 uppercase tracking-widest">{i18n.t('missions.header_active_clusters')}</h3>
                <Tooltip content={i18n.t('missions.tooltip_create_cluster')} position="left">
                    <button
                        onClick={() => setShowCreateModal(true)}
                        className="p-1 px-2 rounded-lg border border-zinc-800 bg-zinc-900 text-xs font-bold text-zinc-400 hover:text-white hover:border-zinc-700 transition-all flex items-center gap-1"
                    >
                        <Plus size={10} /> {i18n.t('missions.btn_new_mission')}
                    </button>
                </Tooltip>
            </div>

            {showCreateModal && (
                <div className="sovereign-card animate-in slide-in-from-top-2 border-blue-500/30 bg-blue-500/5">
                    <h4 className="text-xs font-bold text-blue-400 uppercase tracking-widest mb-3">{i18n.t('missions.header_create_cluster')}</h4>
                    <div className="space-y-3">
                        <input
                            className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-xs text-zinc-200"
                            placeholder={i18n.t('missions.placeholder_name')}
                            aria-label={i18n.t('missions.placeholder_name')}
                            value={newCluster.name}
                            onChange={e => setNewCluster({ ...newCluster, name: e.target.value })}
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
                                    value={newMissionBudget}
                                    onChange={e => setNewMissionBudget(e.target.value)}
                                />
                            </div>
                        </div>
                        <div className="flex gap-2 items-end">
                            <div className="flex-1 space-y-1">
                                <label className="text-[10px] uppercase text-zinc-500 font-bold tracking-wider">{i18n.t('missions.label_dept')}</label>
                                <select
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded p-2 text-xs text-zinc-300"
                                    aria-label={i18n.t('missions.label_dept')}
                                    value={newCluster.department}
                                    onChange={e => setNewCluster({ ...newCluster, department: e.target.value as MissionCluster['department'] })}
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
                                </select>
                            </div>
                            <button
                                onClick={handleCreate}
                                disabled={!newCluster.name}
                                className="h-[34px] px-3 bg-blue-600 text-white rounded text-xs font-bold uppercase disabled:opacity-50"
                            >
                                {i18n.t('missions.btn_create')}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {clusters.map(cluster => {
                const isSelected = selectedClusterId === cluster.id;
                const theme = getThemeColors(cluster.theme);
                const isActive = cluster.isActive;

                return (
                    <div
                        key={cluster.id}
                        role="button"
                        tabIndex={0}
                        onClick={() => onSelectCluster(cluster.id)}
                        onKeyDown={(e) => {
                            if (e.key === 'Enter' || e.key === ' ') {
                                e.preventDefault();
                                onSelectCluster(cluster.id);
                            }
                        }}
                        className={`
                            group relative p-3 rounded-xl border transition-all cursor-pointer overflow-hidden
                            ${isSelected ? `${theme.bg} ${theme.border} shadow-lg ${theme.glow}` : 'bg-zinc-900 border-zinc-800 hover:border-zinc-700'}
                            ${isActive ? 'ring-1 ring-emerald-500/30' : ''}
                        `}
                    >
                        {isActive && (
                            <div className="absolute inset-0 bg-emerald-500/5 animate-pulse pointer-events-none" />
                        )}

                        <div className="flex justify-between items-start mb-2 relative z-10 gap-2">
                            <div className="flex flex-col min-w-0 flex-1">
                                <span className={`text-xs font-bold truncate ${isSelected ? theme.text : 'text-zinc-300'}`}>
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
                                                        onUpdateDepartment(cluster.id, e.target.value as MissionCluster['department']);
                                                    }}
                                                    style={{ colorScheme: 'dark' }}
                                                >
                                                    {['Executive', 'Engineering', 'Operations', 'Product', 'Marketing', 'Sales', 'Design', 'Research', 'Support', 'Quality Assurance'].map(dept => (
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
                                                    value={editingBudgets[cluster.id] !== undefined ? editingBudgets[cluster.id] : (cluster.budgetUsd || 0).toString()}
                                                    onChange={(e) => {
                                                        e.stopPropagation();
                                                        handleBudgetChange(cluster.id, e.target.value);
                                                    }}
                                                    onClick={(e) => e.stopPropagation()}
                                                    onBlur={() => {
                                                        // Ensure clean sync on blur
                                                        setEditingBudgets(prev => {
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
                                <Tooltip content={isActive ? i18n.t('missions.tooltip_deactivate') : i18n.t('missions.tooltip_activate')} position="top">
                                    <button
                                        onClick={(e) => { e.stopPropagation(); onToggleActive(cluster.id); }}
                                        className={`p-1 rounded hover:bg-zinc-800 transition-colors ${isActive ? 'text-emerald-400' : 'text-zinc-600'}`}
                                    >
                                        <Zap size={12} fill={isActive ? "currentColor" : "none"} />
                                    </button>
                                </Tooltip>
                                <Tooltip content={i18n.t('missions.tooltip_delete')} position="top">
                                    <button
                                        onClick={(e) => { e.stopPropagation(); onDeleteCluster(cluster.id); }}
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
                                const isAlpha = cluster.alphaId === id;
                                const avatarColor = agent?.themeColor || (isAlpha ? '#f59e0b' : undefined);
                                return (
                                    <Tooltip key={id} content={isAlpha ? i18n.t('missions.tooltip_alpha') : i18n.t('missions.tooltip_subordinate')} position="top">
                                        <div
                                            className="w-7 h-7 rounded-full border-2 border-black flex items-center justify-center transition-colors relative"
                                            style={{ backgroundColor: avatarColor ? `${avatarColor}30` : '#27272a', borderColor: avatarColor || '#3f3f46' }}
                                        >
                                            <span className="text-[10px] font-bold" style={{ color: avatarColor || '#a1a1aa' }}>
                                                {agent?.name[0] || '?'}
                                            </span>
                                            {isAlpha && (
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
