/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Identity and visual signature editor for agent nodes. 
 * Orchestrates theme color calibration, role assignment, and panel lifecycles (close/detach).
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Hex code validation failure (rare but possible via manual input bypass), color picker z-index overlap in detached mode, or role selector desync during store hydration.
 * - **Telemetry Link**: Search for `[Agent_Config_Header]` or `color_profile` in UI logs.
 */

import { X, Sliders, ChevronDown, ExternalLink } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

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
    onUpdateIdentity,
    onUpdateThemeColor,
    onRoleChange,
    isDetached = false
}: AgentConfigHeaderProps) {
    return (
        <div className="p-6 border-b border-zinc-800 bg-zinc-900/80 backdrop-blur-md flex items-start justify-between shrink-0 relative overflow-hidden">
            <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-emerald-500/20 to-transparent" />

            <div className="flex items-start gap-4 z-10">
                <div className="relative group/picker">
                    <Tooltip content="Select a custom color profile for this agent node's signature." position="top">
                        <div
                            className="p-3 rounded-xl border transition-all duration-300 relative overflow-hidden"
                            style={{
                                backgroundColor: `${themeColor}15`,
                                borderColor: `${themeColor}40`,
                                boxShadow: `0 0 20px ${themeColor}10`
                            }}
                        >
                            <Sliders size={20} style={{ color: themeColor }} />
                            <input
                                type="color"
                                value={themeColor}
                                onChange={(e) => onUpdateThemeColor(e.target.value)}
                                className="absolute inset-0 opacity-0 cursor-pointer w-full h-full"
                                aria-label={i18n.t('agent_config.aria_theme_color')}
                            />
                        </div>
                    </Tooltip>
                    <div className="absolute -bottom-1 -right-1 w-3 h-3 rounded-full border border-black/50 shadow-sm" style={{ backgroundColor: themeColor }} />
                </div>
                <div className="space-y-1">
                    <h2 className="text-[11px] font-bold text-emerald-500 tracking-[0.2em] uppercase opacity-80">
                        {isNew ? i18n.t('agent_config.init_new') : i18n.t('agent_config.init_config')}
                    </h2>
                    <input
                        value={name}
                        onChange={(e) => onUpdateIdentity('name', e.target.value)}
                        className="bg-transparent border-none p-0 font-bold text-zinc-100 text-xl leading-tight focus:ring-0 w-full hover:bg-white/5 rounded px-1 -ml-1 transition-colors"
                        spellCheck={false}
                        aria-label={i18n.t('agent_config.aria_agent_name')}
                    />
                    <div className="flex items-center gap-3 pt-1">
                        <div className="relative group/role">
                            <select
                                value={role}
                                onChange={(e) => onRoleChange(e.target.value)}
                                aria-label={i18n.t('agent_config.aria_role_selector')}
                                className="appearance-none bg-zinc-900/80 border border-zinc-700/50 rounded px-2 py-0.5 text-xs font-bold text-zinc-300 uppercase tracking-[0.2em] cursor-pointer hover:border-emerald-500/50 hover:text-emerald-400 transition-all focus:outline-none pr-6"
                            >
                                {availableRoles.map(r => (
                                    <option key={r} value={r} className="bg-zinc-900">{r.toUpperCase()}</option>
                                ))}
                            </select>
                            <ChevronDown size={10} className="absolute right-1.5 top-1/2 -translate-y-1/2 text-zinc-600 group-hover/role:text-emerald-400 pointer-events-none" />
                        </div>

                        <div className="relative group/dept">
                            <select
                                value={department}
                                onChange={(e) => onUpdateIdentity('department', e.target.value)}
                                aria-label={i18n.t('agent_config.aria_dept_selector')}
                                className="appearance-none bg-zinc-900/80 border border-zinc-700/50 rounded px-2 py-0.5 text-xs font-bold text-green-400 uppercase tracking-[0.2em] cursor-pointer hover:border-green-500/50 hover:text-blue-300 transition-all focus:outline-none pr-6"
                            >
                                {Object.entries(i18n.t('common.departments', { returnObjects: true })).map(([key, label]) => (
                                    <option key={key} value={label as string} className="bg-zinc-900">{String(label).toUpperCase()}</option>
                                ))}
                            </select>
                            <ChevronDown size={10} className="absolute right-1.5 top-1/2 -translate-y-1/2 text-zinc-600 group-hover/dept:text-green-400 pointer-events-none" />
                        </div>

                        <span className="text-[11px] text-zinc-500 font-mono tracking-tighter opacity-50">
                            {agentId ? i18n.t('agent_config.neural_node_id', { id: agentId.substring(0, 8).toUpperCase() }) : i18n.t('agent_config.id_pending')}
                        </span>
                    </div>
                </div>
            </div>

            <div className="flex items-center gap-2 z-10 shrink-0">
                {!isDetached && onDetach && (
                    <Tooltip content={i18n.t('agent_config.tooltip_detach')} position="bottom">
                        <button
                            onClick={onDetach}
                            className="p-3 w-11 h-11 flex items-center justify-center rounded-lg hover:bg-zinc-800 transition-colors text-zinc-500 hover:text-white"
                            aria-label={i18n.t('agent_config.aria_detach')}
                        >
                            <ExternalLink size={18} />
                        </button>
                    </Tooltip>
                )}
                <Tooltip content={i18n.t('agent_config.aria_close_panel')} position="bottom">
                    <button onClick={onClose} className="p-3 w-11 h-11 flex items-center justify-center rounded-lg hover:bg-zinc-800 transition-colors text-zinc-500 hover:text-white" aria-label={i18n.t('agent_config.aria_close_panel')}>
                        <X size={20} />
                    </button>
                </Tooltip>
            </div>
        </div>
    );
}


// Metadata: [Agent_Config_Header]

// Metadata: [AgentConfigHeader]

// Metadata: [AgentConfigHeader]
