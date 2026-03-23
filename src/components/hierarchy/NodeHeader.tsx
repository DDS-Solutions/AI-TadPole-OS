import React from 'react';
import { Crown, ChevronDown, Sliders, Zap, Brain, Shield } from 'lucide-react';
import { Tooltip } from '../ui';
import { getDepartmentIcon, getAgentStatusStyles, getValenceColor } from '../../utils/agentUIUtils';
import { useDropdownStore } from '../../stores/dropdownStore';
import { i18n } from '../../i18n';
import type { Agent } from '../../types';

interface NodeHeaderProps {
    agent: Agent;
    isAlpha?: boolean;
    isActive?: boolean;
    availableRoles: string[];
    onRoleChange?: (agentId: string, newRole: string) => void;
    onConfigureClick?: (agentId: string) => void;
    hasOversight?: boolean;
    isOversightOpen?: boolean;
    onOversightToggle?: () => void;
    isHealthOpen?: boolean;
    onHealthToggle?: () => void;
}

export const NodeHeader: React.FC<NodeHeaderProps> = ({
    agent,
    isAlpha,
    isActive,
    availableRoles,
    onRoleChange,
    onConfigureClick,
    hasOversight,
    isOversightOpen,
    onOversightToggle,
    isHealthOpen,
    onHealthToggle
}) => {
    const toggle = useDropdownStore(s => s.toggle);
    const close = useDropdownStore(s => s.close);
    const isRoleDropdownOpen = useDropdownStore(s => s.openId === agent.id && s.openType === 'role');

    const deptIcon = getDepartmentIcon(agent.department || '');
    const statusStyles = getAgentStatusStyles(agent.status);
    const agentColor = agent.themeColor || statusStyles.hex;

    const roleBadgeClass =
        agent.department === 'Executive' ? 'text-amber-400 border-amber-900 bg-amber-900/10' :
            agent.department === 'Engineering' ? 'text-blue-400 border-blue-900 bg-blue-900/10' :
                agent.department === 'Product' ? 'text-orange-400 border-orange-900 bg-orange-900/10' :
                    'text-zinc-400 border-zinc-800 bg-zinc-900';
    
    const failureCount = agent.failureCount || 0;
    const healthColor = failureCount >= 3 ? 'text-red-500' : failureCount > 0 ? 'text-amber-500' : 'text-emerald-500';
    const healthTooltip = failureCount >= 3 ? i18n.t('throttled') : failureCount > 0 ? i18n.t('degraded') : i18n.t('healthy');

    return (
        <div className={`grid grid-cols-[min-content_1fr_min-content] gap-2 items-start ${isRoleDropdownOpen ? 'z-50' : 'z-20'}`}>
            {/* Col 1: Icon */}
            <div className={`
                w-8 h-8 rounded-lg flex items-center justify-center border transition-all relative shrink-0
                ${isAlpha ? 'bg-zinc-900 border-amber-500/50 shadow-[0_0_10px_rgba(245,158,11,0.2)]' : (agent.status !== 'offline' && agent.status !== 'idle' ? 'bg-zinc-900 border-white/10' : 'bg-zinc-950 border-white/5')}
            `}>
                {agent.status !== 'offline' && agent.status !== 'idle' && (
                    <div className="absolute inset-0 rounded-lg animate-ping opacity-20 border border-current" style={{ backgroundColor: 'transparent', borderColor: agentColor }} />
                )}
                {agent.valence !== undefined && (
                    <div 
                        className="absolute -inset-1 rounded-lg animate-pulse" 
                        style={{ 
                            border: `2px solid ${getValenceColor(agent.valence)}`, 
                            boxShadow: `0 0 10px ${getValenceColor(agent.valence)}60`,
                            zIndex: -1
                        }} 
                    />
                )}
                {isAlpha ? (
                    <Crown size={14} className="text-amber-500 animate-pulse" />
                ) : (
                    React.createElement(deptIcon, { size: 14, className: agent.status !== 'offline' && agent.status !== 'idle' ? '' : 'opacity-40' })
                )}
            </div>

            {/* Col 2: Name + Role */}
            <div className="flex flex-col min-w-0 overflow-hidden">
                <div className="flex items-center gap-1.5 min-w-0">
                    <Tooltip content={i18n.t('agent_card.tooltip_full_id', { name: agent.name })} position="top">
                        <span className="font-bold text-zinc-100 text-xs tracking-tight leading-none truncate cursor-help">{agent.name}</span>
                    </Tooltip>
                    {isAlpha && <Crown size={10} className="text-amber-400 fill-amber-400/20 shrink-0" />}
                </div>
                <div className="relative shrink-0" role="presentation" onClick={(e) => e.stopPropagation()} onKeyDown={(e) => e.stopPropagation()}>
                    <Tooltip content={i18n.t('agent_card.tooltip_current_role', { role: agent.role })} position="top">
                        <button
                            onClick={(e) => { e.stopPropagation(); toggle(agent.id, 'role'); }}
                            aria-haspopup="listbox"
                            aria-expanded={isRoleDropdownOpen}
                            aria-label={i18n.t('agent_card.aria_role_selector', { role: agent.role })}
                            className={`text-[11px] px-1.5 py-0.5 rounded border font-mono flex items-center gap-1 hover:bg-white/5 transition-colors cursor-pointer max-w-full ${roleBadgeClass}`}>
                            <span className="truncate">{agent.role.toUpperCase()}</span>
                            <ChevronDown size={8} className="opacity-70 shrink-0" />
                        </button>
                    </Tooltip>

                    {isRoleDropdownOpen && (
                        <div className="absolute top-full left-0 mt-1 w-48 bg-zinc-900 border border-zinc-700 rounded-lg shadow-xl z-50 py-1 max-h-60 overflow-y-auto custom-scrollbar">
                            {availableRoles.map((role) => (
                                <button
                                    key={role}
                                    onClick={() => {
                                        onRoleChange?.(agent.id, role);
                                        close();
                                    }}
                                    className={`w-full text-left px-3 py-1.5 text-xs hover:bg-zinc-800 transition-colors ${agent.role === role ? 'text-blue-400 font-bold bg-blue-900/10' : 'text-zinc-300'}`}
                                >
                                    {role}
                                </button>
                            ))}
                        </div>
                    )}
                </div>
            </div>

            {/* Col 3: Status + Actions */}
            <div className="flex items-center gap-2 shrink-0">
                <div className="flex flex-col items-end gap-1">
                    {isActive && (
                        <div className="flex items-center gap-1 px-2 py-0.5 rounded bg-emerald-500/10 border border-emerald-500/20 text-[10px] font-bold text-emerald-400 uppercase tracking-widest animate-in fade-in zoom-in-95">
                            <Zap size={8} fill="currentColor" /> {i18n.t('agent_card.badge_active')}
                        </div>
                    )}
                    {isAlpha && hasOversight && (
                        <Tooltip content={isOversightOpen ? i18n.t('oversight.btn_hide') : i18n.t('oversight.btn_show')} position="top">
                            <button
                                onClick={(e) => { e.stopPropagation(); onOversightToggle?.(); }}
                                aria-label={isOversightOpen ? i18n.t('oversight.btn_hide') : i18n.t('oversight.btn_show')}
                                className={`
                                    flex items-center gap-1.5 px-2 py-0.5 rounded border text-[10px] font-bold uppercase tracking-widest transition-all duration-300
                                    ${isOversightOpen
                                        ? 'bg-blue-500/20 border-blue-500/40 text-blue-400 shadow-[0_0_15px_rgba(59,130,246,0.2)]'
                                        : 'bg-zinc-900 border-zinc-700 text-zinc-500 hover:border-blue-500/50 hover:text-blue-400 shadow-[0_0_10px_rgba(59,130,246,0.1)]'
                                    }
                                `}
                            >
                                <Brain size={10} className={!isOversightOpen ? 'animate-pulse' : ''} />
                                <span>{i18n.t('oversight.mod_req_label')}</span>
                            </button>
                        </Tooltip>
                    )}
                </div>
                <Tooltip content={healthTooltip} position="top">
                    <button
                        onClick={(e) => { e.stopPropagation(); onHealthToggle?.(); }}
                        aria-label={healthTooltip}
                        aria-expanded={isHealthOpen}
                        className={`p-1 rounded hover:bg-zinc-800 transition-colors ${isHealthOpen ? 'bg-zinc-800 ' + healthColor : 'text-zinc-600 hover:' + healthColor}`}
                    >
                        <Shield size={12} fill={isHealthOpen ? 'currentColor' : 'none'} className={failureCount >= 3 ? 'animate-pulse' : ''} />
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('agent_card.tooltip_configure')} position="top">
                    <button
                        onClick={(e) => { e.stopPropagation(); onConfigureClick?.(agent.id); }}
                        aria-label={i18n.t('agent_card.tooltip_configure')}
                        className="p-1 rounded hover:bg-blue-900/20 text-zinc-600 hover:text-blue-400 transition-colors"
                    >
                        <Sliders size={12} />
                    </button>
                </Tooltip>
                <div className={`w-1.5 h-1.5 rounded-full mt-1 transition-all duration-500 ${agent.status === 'active' ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]' :
                    agent.status === 'thinking' ? 'bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.5)] animate-pulse' :
                        agent.status === 'coding' ? 'bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.5)] animate-pulse' :
                            agent.status === 'speaking' ? 'bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.5)] animate-pulse' :
                                'bg-zinc-700'
                    }`} />
            </div>
        </div>
    );
};
