/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Identity and visual signature editor for agent nodes. 
 * Orchestrates theme color calibration, role assignment, and panel lifecycles (close/detach).
 * Delegates global role and department administration to dedicated manager modals.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Z-index overlap of sub-modals or dropdown synchronization latency.
 * - **Telemetry Link**: Search for `[Agent_Config_Header]` or color_profile in UI logs.
 */

import { useState, useCallback, useMemo } from 'react';
import { X, Sliders, ChevronDown, ExternalLink, Settings, Trash2 } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import { use_role_store } from '../../stores/role_store';
import { use_department_store } from '../../stores/department_store';
import { RoleManagerModal } from './RoleManagerModal';
import { DepartmentManagerModal } from './DepartmentManagerModal';

interface AgentConfigHeaderProps {
    name: string;
    role: string;
    department: string;
    themeColor: string;
    isNew: boolean;
    agentId?: string;
    availableRoles: string[];
    onClose: () => void;
    onDetach?: () => void;
    onDelete?: () => void;
    onUpdateIdentity: (field: 'name' | 'role' | 'department', value: string) => void;
    onUpdateThemeColor: (color: string) => void;
    onRoleChange: (role: string) => void;
    isDetached?: boolean;
}

/**
 * Agent_Config_Header
 * Provides the identity and visual style configuration interface for agent nodes.
 * Supports real-time theme color calibration and role assignment.
 */
export function AgentConfigHeader({
    name,
    role,
    department,
    themeColor,
    isNew,
    agentId,
    availableRoles,
    onClose,
    onDetach,
    onDelete,
    onUpdateIdentity,
    onUpdateThemeColor,
    onRoleChange,
    isDetached = false
}: AgentConfigHeaderProps) {
    // External Stores
    const roles = use_role_store(s => s.roles);
    const { departments } = use_department_store();

    // Modals & management states
    const [isManagingRoles, setIsManagingRoles] = useState(false);
    const [isManagingDepts, setIsManagingDepts] = useState(false);

    const safeThemeColor = useMemo(() => {
        return (themeColor && /^#[0-9A-F]{6}$/i.test(themeColor)) ? themeColor : '#10b981';
    }, [themeColor]);

    const handleUpdateDeptCallback = useCallback((newDept: string) => {
        onUpdateIdentity('department', newDept);
    }, [onUpdateIdentity]);

    return (
        <div className="p-6 border-b border-[color:var(--color-border)] bg-[color:color-mix(in_srgb,var(--color-surface)_80%,transparent)] backdrop-blur-md flex items-start justify-between shrink-0 relative overflow-hidden">
            <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-emerald-500/20 to-transparent" />

            <div className="flex items-start gap-4 z-10 min-w-0 flex-1 pr-4">
                <div className="relative group/picker shrink-0">
                    <Tooltip content="Select a custom color profile for this agent node's signature." position="top">
                        <div
                            className="p-3 rounded-xl border transition-all duration-300 relative overflow-hidden"
                            style={{
                                backgroundColor: `color-mix(in srgb, ${safeThemeColor} 8.3%, transparent)`,
                                borderColor: `color-mix(in srgb, ${safeThemeColor} 40%, transparent)`,
                                boxShadow: `0 0 20px color-mix(in srgb, ${safeThemeColor} 10%, transparent)`
                            }}
                        >
                            <Sliders size={20} style={{ color: safeThemeColor }} />
                            <input
                                type="color"
                                value={safeThemeColor}
                                onChange={(e) => onUpdateThemeColor(e.target.value)}
                                className="absolute inset-0 opacity-0 cursor-pointer w-full h-full"
                                aria-label={i18n.t('agent_config.aria_theme_color')}
                            />
                        </div>
                    </Tooltip>
                    <div className="absolute -bottom-1 -right-1 w-3 h-3 rounded-full border border-black/50 shadow-sm" style={{ backgroundColor: safeThemeColor }} />
                </div>
                <div className="space-y-1 min-w-0 flex-1">
                    <h2 className="text-[11px] font-bold text-emerald-500 tracking-[0.2em] uppercase opacity-80">
                        {isNew ? i18n.t('agent_config.init_new') : i18n.t('agent_config.init_config')}
                    </h2>
                    <input
                        value={name}
                        onChange={(e) => onUpdateIdentity('name', e.target.value)}
                        className="bg-transparent border-none p-0 font-bold text-zinc-100 text-lg leading-tight focus:ring-0 max-w-[280px] sm:max-w-[340px] w-full truncate hover:bg-white/5 rounded px-1 -ml-1 transition-colors"
                        spellCheck={false}
                        aria-label={i18n.t('agent_config.aria_agent_name')}
                    />
                    <div className="flex items-center gap-3 pt-1 flex-nowrap whitespace-nowrap">
                        <div className="relative group/role flex items-center gap-1.5 shrink-0">
                            <div className="relative">
                                <select
                                    value={role}
                                    onChange={(e) => onRoleChange(e.target.value)}
                                    aria-label={i18n.t('agent_config.aria_role_selector')}
                                    className="appearance-none bg-[color:color-mix(in_srgb,var(--color-surface)_80%,transparent)] border border-zinc-700/50 rounded px-2 py-0.5 text-xs font-bold text-zinc-300 uppercase tracking-wider cursor-pointer hover:border-emerald-500/50 hover:text-emerald-400 transition-all focus:outline-none pr-6"
                                >
                                    {availableRoles.map(r => (
                                        <option key={r} value={r} className="bg-[color:var(--color-surface)]">
                                            {(roles[r]?.name || r).toUpperCase()}
                                        </option>
                                    ))}
                                </select>
                                <ChevronDown size={10} className="absolute right-1.5 top-1/2 -translate-y-1/2 text-zinc-600 group-hover/role:text-emerald-400 pointer-events-none" />
                            </div>
                            <Tooltip content="Manage Roles" position="top">
                                <button
                                    type="button"
                                    onClick={() => setIsManagingRoles(true)}
                                    className="p-1 hover:bg-zinc-800/80 rounded transition-colors text-zinc-500 hover:text-zinc-300 cursor-pointer flex items-center justify-center border border-zinc-700/30 shrink-0"
                                >
                                    <Settings size={12} />
                                </button>
                            </Tooltip>
                        </div>

                        <div className="relative group/dept flex items-center gap-1.5 shrink-0">
                            <div className="relative">
                                <select
                                    value={department}
                                    onChange={(e) => onUpdateIdentity('department', e.target.value)}
                                    aria-label={i18n.t('agent_config.aria_dept_selector')}
                                    className="appearance-none bg-[color:color-mix(in_srgb,var(--color-surface)_80%,transparent)] border border-zinc-700/50 rounded px-2 py-0.5 text-xs font-bold text-green-400 uppercase tracking-wider cursor-pointer hover:border-green-500/50 hover:text-blue-300 transition-all focus:outline-none pr-6"
                                >
                                    {departments.map(d => (
                                        <option key={d} value={d} className="bg-[color:var(--color-surface)]">{d.toUpperCase()}</option>
                                    ))}
                                </select>
                                <ChevronDown size={10} className="absolute right-1.5 top-1/2 -translate-y-1/2 text-zinc-600 group-hover/dept:text-green-400 pointer-events-none" />
                            </div>
                            <Tooltip content="Manage Departments" position="top">
                                <button
                                    type="button"
                                    onClick={() => setIsManagingDepts(true)}
                                    className="p-1 hover:bg-zinc-800/80 rounded transition-colors text-zinc-500 hover:text-zinc-300 cursor-pointer flex items-center justify-center border border-zinc-700/30 shrink-0"
                                >
                                    <Settings size={12} />
                                </button>
                            </Tooltip>
                        </div>

                        <span className="text-[11px] text-zinc-500 font-mono tracking-tighter opacity-50 shrink-0">
                            {agentId ? i18n.t('agent_config.neural_node_id', { id: agentId.substring(0, 8).toUpperCase() }) : i18n.t('agent_config.id_pending')}
                        </span>
                    </div>
                </div>
            </div>

            <div className="flex items-center gap-2 z-10 shrink-0 ml-3">
                {!isNew && onDelete && (
                    <Tooltip content="Purge Node from Swarm Registry" position="bottom">
                        <button
                            onClick={() => {
                                if (window.confirm(`Are you sure you want to purge agent "${name}" from the swarm?`)) {
                                    onDelete();
                                }
                            }}
                            className="p-2.5 flex items-center justify-center rounded-lg border border-red-500/30 bg-red-500/10 hover:bg-red-500/20 transition-all text-red-400 hover:text-red-300 shadow-sm cursor-pointer"
                            aria-label="Delete Agent Node"
                        >
                            <Trash2 size={16} />
                        </button>
                    </Tooltip>
                )}
                {!isDetached && onDetach && (
                    <Tooltip content={i18n.t('agent_config.tooltip_detach')} position="bottom">
                        <button
                            onClick={onDetach}
                            className="p-2.5 flex items-center justify-center rounded-lg border border-cyan-500/30 bg-cyan-500/10 hover:bg-cyan-500/20 transition-all text-cyan-400 hover:text-cyan-300 shadow-sm cursor-pointer"
                            aria-label={i18n.t('agent_config.aria_detach')}
                        >
                            <ExternalLink size={16} />
                        </button>
                    </Tooltip>
                )}
                <Tooltip content={i18n.t('agent_config.aria_close_panel')} position="bottom">
                    <button 
                        onClick={onClose} 
                        className="p-2.5 flex items-center justify-center rounded-lg border border-zinc-700 bg-zinc-800/80 hover:bg-zinc-700 transition-all text-zinc-200 hover:text-white shadow-sm cursor-pointer" 
                        aria-label={i18n.t('agent_config.aria_close_panel')}
                    >
                        <X size={16} />
                    </button>
                </Tooltip>
            </div>

            <RoleManagerModal
                isOpen={isManagingRoles}
                onClose={() => setIsManagingRoles(false)}
                currentAgentRole={role}
            />

            <DepartmentManagerModal
                isOpen={isManagingDepts}
                onClose={() => setIsManagingDepts(false)}
                currentAgentDept={department}
                onUpdateAgentDept={handleUpdateDeptCallback}
            />
        </div>
    );
}

// Metadata: [Agent_Config_Header]
