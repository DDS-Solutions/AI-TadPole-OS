/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Agent_Manager_Header]` in observability traces.
 */

import React from 'react';
import { Users, UserPlus, Cpu, Filter, Search } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

interface AgentManagerHeaderProps {
    active_tab: 'user' | 'ai';
    user_count: number;
    ai_count: number;
    filter_role: string;
    filter_roles: string[];
    search_query: string;
    on_add_click: () => void;
    on_tab_change: (tab: 'user' | 'ai') => void;
    on_filter_change: (role: string) => void;
    on_search_change: (query: string) => void;
}

export const Agent_Manager_Header: React.FC<AgentManagerHeaderProps> = ({
    active_tab,
    user_count,
    ai_count,
    filter_role,
    filter_roles,
    search_query,
    on_add_click,
    on_tab_change,
    on_filter_change,
    on_search_change
}) => {
    return (
        <div className="py-4 px-6 border-b border-[color:var(--color-border)] bg-[color:color-mix(in_srgb,var(--color-background)_80%,transparent)] backdrop-blur-xl sticky top-0 z-40 flex items-center justify-between">
            <div>
                <h1 className="text-xl font-bold text-zinc-100 flex items-center gap-2 uppercase tracking-tight">
                    <Users className="text-green-500" /> {i18n.t('agent_manager.title')}
                </h1>
            </div>
            <div className="flex items-center gap-4">
                <Tooltip content={i18n.t('agent_manager.tooltip_add')} position="bottom">
                    <button
                        onClick={on_add_click}
                        className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-500 text-white font-bold rounded-lg text-xs transition-all shadow-[0_0_15px_rgba(34,197,94,0.2)] hover:shadow-[0_0_20px_rgba(34,197,94,0.4)]"
                    >
                        <UserPlus size={14} />
                        {i18n.t('agent_manager.btn_add')}
                    </button>
                </Tooltip>

                <div className="h-6 w-px bg-zinc-800 mx-1" />

                <div className="flex items-center p-1 bg-[color:var(--color-surface)] rounded-lg border border-[color:var(--color-border)] self-center">
                    <button
                        onClick={() => on_tab_change('user')}
                        className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all flex items-center gap-2 ${active_tab === 'user'
                            ? 'bg-green-600 text-white shadow-lg'
                            : 'text-zinc-500 hover:text-zinc-300'
                            }`}
                    >
                        <Users size={12} />
                        User Sector
                        <span className={`px-1 rounded ${active_tab === 'user' ? 'bg-black/20 text-white' : 'bg-zinc-800 text-zinc-600'}`}>
                            {user_count}
                        </span>
                    </button>
                    <button
                        onClick={() => on_tab_change('ai')}
                        className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all flex items-center gap-2 ${active_tab === 'ai'
                            ? 'bg-emerald-600 text-white shadow-lg'
                            : 'text-zinc-500 hover:text-zinc-300'
                            }`}
                    >
                        <Cpu size={12} />
                        AI Swarm
                        <span className={`px-1 rounded ${active_tab === 'ai' ? 'bg-black/20 text-white' : 'bg-zinc-800 text-zinc-600'}`}>
                            {ai_count}
                        </span>
                    </button>
                </div>

                <div className="h-6 w-px bg-zinc-800 mx-1" />

                <div className="relative">
                    <Tooltip content={i18n.t('agent_manager.tooltip_filter')} position="bottom">
                        <div className="relative">
                            <Filter className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" size={14} />
                            <select
                                value={filter_role}
                                onChange={(e) => on_filter_change(e.target.value)}
                                className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] text-zinc-200 text-xs rounded-lg pl-9 pr-8 py-2 appearance-none focus:outline-none focus:border-green-500/50 cursor-pointer uppercase font-bold tracking-wider"
                            >
                                <option value="all">{i18n.t('agent_manager.filter_all')}</option>
                                {filter_roles.map(role => (
                                    <option key={role} value={role}>{role}</option>
                                ))}
                            </select>
                        </div>
                    </Tooltip>
                </div>

                <div className="relative">
                    <Tooltip content={i18n.t('agent_manager.tooltip_search')} position="bottom">
                        <div className="relative">
                            <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" size={14} />
                            <input
                                type="text"
                                placeholder={i18n.t('agent_manager.search_placeholder')}
                                value={search_query}
                                onChange={(e) => on_search_change(e.target.value)}
                                className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] text-zinc-200 text-xs rounded-lg pl-9 pr-3 py-2 w-64 focus:outline-none focus:border-green-500/50 transition-colors placeholder:text-zinc-600"
                            />
                        </div>
                    </Tooltip>
                </div>
            </div>
        </div>
    );
};

// Metadata: [Agent_Manager_Header]
