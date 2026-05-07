/**
 * @docs ARCHITECTURE:Interface
 * @docs OPERATIONS_MANUAL:Agents
 * 
 * ### AI Assist Note
 * **Root View**: Comprehensive interface for managing the agent swarm. 
 * Orchestrates agent discovery, filtering by Sector (User vs AI), and deep configuration via `AgentConfigPanel`. 
 * Uses `Agent_Card_Memo` for performant grid rendering and `use_agent_store` for real-time state synchronization.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Agent limit reached (prevents creation), filtering logic mismatch (missing agents in search), or `Agent_Card` stale state (check `React.memo` equality logic).
 * - **Telemetry Link**: Check `fetch_agents` in `agent_store` or search `[AgentManager]` in component logs.
 */

import React, { useState, useEffect, useMemo } from 'react';
import { Users, Search, Filter, Cpu, Sliders, Shield, Activity, Zap, UserPlus, Info, DollarSign, Code, FileText, Terminal } from 'lucide-react';
import { Tooltip } from '../components/ui';
import { i18n } from '../i18n';
import { use_agent_store } from '../stores/agent_store';
import type { Agent, Department } from '../types';
import AgentConfigPanel from '../components/AgentConfigPanel';
import { use_role_store } from '../stores/role_store';
import { get_settings } from '../stores/settings_store';
import { resolve_provider, get_active_model_name } from '../utils/model_utils';

/**
 * @page Agent_Manager
 * Interface for managing the agent swarm.
 * 
 * FEATURES:
 * - Agent Grid: Visual overview of all agents.
 * - Filtering: Search by name/role and filter by role type.
 * - Configuration: Access to AgentConfigPanel for deep editing.
 * - State Sync: Real-time UI updates reflecting active/paused statuses.
 */
export default function Agent_Manager() {
    const { agents, fetch_agents, update_agent, add_agent } = use_agent_store();
    const [selected_agent, set_selected_agent] = useState<Agent | null>(null);
    const [is_creating, set_is_creating] = useState(false);
    const [search_query, set_search_query] = useState('');
    const [filter_role, set_filter_role] = useState<string>('all');
    const [active_tab, set_active_tab] = useState<'user' | 'ai'>('user');

    useEffect(() => {
        const controller = new AbortController();
        if (agents.length === 0) {
            fetch_agents({ signal: controller.signal });
        }
        return () => controller.abort();
    }, [agents.length, fetch_agents]);

    /**
     * Synchronizes agent state with the parent store to maintain panel feedback.
     * If in 'creation' mode, invokes the persistent registration flow.
     */
    const handle_update_agent = (id: string, updates: Partial<Agent>) => {
        if (is_creating) {
            // If we're creating, we use add_agent instead of update
            // Ensure the new agent has the correct category based on where it was created
            add_agent({ ...selected_agent!, category: active_tab, ...updates } as Agent);
            set_is_creating(false);
            set_selected_agent(null);
        } else {
            update_agent(id, updates);
            if (selected_agent && selected_agent.id === id) {
                set_selected_agent(prev => prev ? { ...prev, ...updates } : null);
            }
        }
    };

    const handle_add_new_click = () => {
        if (agents.length >= 25) {
            alert(i18n.t('agent_manager.error_limit_reached'));
            return;
        }

        const settings = get_settings();
        const new_agent: Agent = {
            id: `node_${Math.random().toString(36).substring(2, 11)}`,
            name: i18n.t('agent_manager.placeholder_name'),
            role: active_tab === 'ai' ? `AI-${i18n.t('agent_manager.placeholder_role')}` : i18n.t('agent_manager.placeholder_role'),
            status: "idle",
            category: active_tab,
            model: settings.default_model,
            skills: [],
            workflows: [],
            tokens_used: 0,
            input_tokens: 0,
            output_tokens: 0,
            cost_usd: 0,
            budget_usd: 1.0,
            description: "",
            active_model_slot: 1,
            mcp_tools: [],
            requires_oversight: false,
            failure_count: 0,
            connector_configs: [],
            metadata: {},
            department: i18n.t('agent_manager.default_dept') as Department,
            theme_color: active_tab === 'user' ? "#22c55e" : "#10b981", // User: Cyber Green, AI: Emerald-500
            model_config: {
                modelId: settings.default_model,
                provider: resolve_provider(settings.default_model),
                temperature: settings.default_temperature,
                systemPrompt: "",
                skills: [],
                workflows: []
            }
        };

        set_is_creating(true);
        set_selected_agent(new_agent);
    };

    const filtered_agents = agents.filter(agent => {
        let matches_tab = agent.category === active_tab;

        // Persistent Swarm Oversight: If in AI Swarm tab, show all AI agents 
        // PLUS any User-sector agent that has been recruited for swarm activity.
        if (active_tab === 'ai' && agent.category === 'user') {
            matches_tab = agent.metadata?.has_participated_in_swarm === true || !!agent.active_mission;
        }

        const matches_search = agent.name.toLowerCase().includes(search_query.toLowerCase()) ||
            agent.role.toLowerCase().includes(search_query.toLowerCase());
        const matches_role = filter_role === 'all' || agent.role === filter_role;
        return matches_tab && matches_search && matches_role;
    });

    const roles = use_role_store(s => s.roles);
    const all_role_names = useMemo(() => Object.keys(roles).sort(), [roles]);

    const filter_roles = useMemo(() => Array.from(new Set([
        ...all_role_names,
        ...agents.filter(a => a.category === active_tab).map(a => a.role)
    ])).sort(), [all_role_names, agents, active_tab]);

    const handle_select_agent = useMemo(() => (agent: Agent) => {
        set_selected_agent(agent);
    }, []);

    return (
        <div className="h-full flex flex-col bg-zinc-950">
            {/* GEO Optimization: Structured Data & Semantic Header */}
            <script type="application/ld+json">
            {JSON.stringify({
              "@context": "https://schema.org",
              "@type": "SoftwareApplication",
              "name": "Tadpole OS Agent Manager",
              "description": "Unified orchestration layer for managing autonomous agent swarms, department allocations, and mission lifecycle.",
              "author": { "@type": "Organization", "name": "Sovereign Engineering" },
              "applicationCategory": "AI Management System",
              "operatingSystem": "Tadpole OS"
            })}
            </script>
            <h2 className="sr-only">Fleet Orchestration & Cluster Management</h2>
            <h2 className="sr-only">Departmental Resource Allocation</h2>
            {/* Standard Sticky Header */}
            <div className="py-4 px-6 border-b border-zinc-900 bg-zinc-950/80 backdrop-blur-md sticky top-0 z-40 flex items-center justify-between">
                <div>
                    <h1 className="text-xl font-bold text-zinc-100 flex items-center gap-2 uppercase tracking-tight">
                        <Users className="text-green-500" /> {i18n.t('agent_manager.title')}
                    </h1>
                    <div className="mt-1">
                    </div>
                </div>
                <div className="flex items-center gap-4">
                    <div className="flex flex-col items-center gap-1.5">
                        <Tooltip content={i18n.t('agent_manager.tooltip_add')} position="bottom">
                            <button
                                onClick={handle_add_new_click}
                                className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-500 text-white font-bold rounded-lg text-xs transition-all shadow-[0_0_15px_rgba(34,197,94,0.2)] hover:shadow-[0_0_20px_rgba(34,197,94,0.4)]"
                            >
                                <UserPlus size={14} />
                                {i18n.t('agent_manager.btn_add')}
                            </button>
                        </Tooltip>
                        <div className="hidden">
                            <Tooltip content={i18n.t('agent_manager.tooltip_capacity')} position="bottom">
                                <div className="flex items-center gap-1.5 opacity-80 cursor-help">
                                    <Info size={10} className="text-green-400" />
                                    <span className="text-[10px] text-zinc-500 font-bold uppercase tracking-tighter">{i18n.t('agent_manager.label_max_capacity', { current: agents.length })}</span>
                                </div>
                            </Tooltip>
                        </div>
                    </div>

                    <div className="h-6 w-px bg-zinc-800 mx-1" />

                    {/* Sector Switching Tabs */}
                    <div className="flex items-center p-1 bg-zinc-900 rounded-lg border border-zinc-800 self-center">
                        <button
                            onClick={() => set_active_tab('user')}
                            className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all flex items-center gap-2 ${active_tab === 'user'
                                ? 'bg-green-600 text-white shadow-lg'
                                : 'text-zinc-500 hover:text-zinc-300'
                                }`}
                        >
                            <Users size={12} />
                            User Sector
                            <span className={`px-1 rounded ${active_tab === 'user' ? 'bg-black/20 text-white' : 'bg-zinc-800 text-zinc-600'}`}>
                                {agents.filter(a => a.category === 'user').length}
                            </span>
                        </button>
                        <button
                            onClick={() => set_active_tab('ai')}
                            className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all flex items-center gap-2 ${active_tab === 'ai'
                                ? 'bg-emerald-600 text-white shadow-lg'
                                : 'text-zinc-500 hover:text-zinc-300'
                                }`}
                        >
                            <Cpu size={12} />
                            AI Swarm
                            <span className={`px-1 rounded ${active_tab === 'ai' ? 'bg-black/20 text-white' : 'bg-zinc-800 text-zinc-600'}`}>
                                {agents.filter(a => a.category === 'ai' || a.metadata?.has_participated_in_swarm === true).length}
                            </span>
                        </button>
                    </div>

                    <div className="h-6 w-px bg-zinc-800 mx-1" />

                    {/* Role Filter */}
                    <div className="relative">
                                <Tooltip content={i18n.t('agent_manager.tooltip_filter')} position="bottom">
                                    <div className="relative">
                                        <Filter className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" size={14} />
                                        <select
                                            value={filter_role}
                                            onChange={(e) => set_filter_role(e.target.value)}
                                            className="bg-zinc-900 border border-zinc-800 text-zinc-200 text-xs rounded-lg pl-9 pr-8 py-2 appearance-none focus:outline-none focus:border-green-500/50 cursor-pointer uppercase font-bold tracking-wider"
                                        >
                                            <option value="all">{i18n.t('agent_manager.filter_all')}</option>
                                    {filter_roles.map(role => (
                                        <option key={role} value={role}>{role}</option>
                                    ))}
                                </select>
                                <div className="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-zinc-500">
                                    <svg width="10" height="6" viewBox="0 0 10 6" fill="currentColor"><path d="M0 0.5L5 5.5L10 0.5H0Z" /></svg>
                                </div>
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
                                    onChange={(e) => set_search_query(e.target.value)}
                                    className="bg-zinc-900 border border-zinc-800 text-zinc-200 text-xs rounded-lg pl-9 pr-3 py-2 w-64 focus:outline-none focus:border-green-500/50 transition-colors placeholder:text-zinc-600"
                                    aria-label={i18n.t('agent_manager.aria_search_agents')}
                                />
                            </div>
                        </Tooltip>
                    </div>
                </div>
            </div>

            <div className="flex-1 overflow-y-auto custom-scrollbar p-6">
                <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
                    {filtered_agents.map(agent => (
                        <Agent_Card_Memo key={agent.id} agent={agent} on_select={handle_select_agent} />
                    ))}
                </div>
            </div>

            {selected_agent && (
                <AgentConfigPanel
                    agent={selected_agent}
                    onClose={() => {
                        set_selected_agent(null);
                        set_is_creating(false);
                    }}
                    onUpdate={handle_update_agent}
                    isNew={is_creating}
                />
            )}
        </div>
    );
}

export const Agent_Card = ({ agent, on_select }: { agent: Agent; on_select: (agent: Agent) => void }) => {
    return (
        <div
            role="button"
            tabIndex={0}
            onClick={() => on_select(agent)}
            onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && on_select(agent)}
            className="group bg-zinc-900 border border-zinc-800 hover:border-green-500/30 rounded-xl p-4 cursor-pointer transition-all hover:shadow-lg hover:shadow-green-900/5 relative overflow-hidden"
            aria-label={i18n.t('agent_manager.aria_configure_agent', { name: agent.name })}
        >
            <div className="absolute top-0 right-0 p-3 opacity-0 group-hover:opacity-100 transition-opacity">
                <Tooltip content={i18n.t('agent_manager.tooltip_configure')} position="left">
                    <Sliders size={16} className="text-green-500" />
                </Tooltip>
            </div>

            <div className="flex items-start gap-4">
                <div
                    className="w-10 h-10 rounded-lg flex items-center justify-center font-bold text-lg shrink-0"
                    style={{
                        backgroundColor: agent.status === 'active' ? `${agent.theme_color || '#22c55e'}15` : '#27272a',
                        color: agent.status === 'active' ? (agent.theme_color || '#22c55e') : '#71717a',
                        border: `1px solid ${agent.status === 'active' ? `${agent.theme_color || '#22c55e'}30` : '#3f3f46'}`
                    }}
                >
                    {(agent.name || '?')[0]}
                </div>
                <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                        <h3
                            className="text-sm font-bold text-zinc-200 truncate group-hover:text-emerald-400 transition-colors"
                            style={agent.status === 'active' ? { transition: 'color 0.2s' } : {}}
                        >
                            {agent.name}
                        </h3>
                        {agent.status === 'active' && (
                            <Tooltip content={i18n.t('agent_manager.tooltip_active')} position="top">
                                <span className="flex items-center gap-1 text-[9px] font-bold text-green-500 uppercase tracking-wider bg-green-500/10 px-1.5 py-0.5 rounded cursor-help">
                                    <Activity size={8} /> {i18n.t('agent_manager.badge_active')}
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

                    <div className="mt-4 flex flex-wrap gap-2">
                        <Tooltip content={`LLM: ${get_active_model_name(agent)} `} position="top">
                            <div className="bg-zinc-900 border border-zinc-800 rounded px-2 py-1 flex items-center gap-1.5 cursor-help">
                                <Cpu size={10} className="text-green-400" />
                                <span className="text-[10px] text-zinc-400 font-mono">{(get_active_model_name(agent) || 'Unknown').split(' ').pop()}</span>
                            </div>
                        </Tooltip>
                        <Tooltip content={i18n.t('agent_manager.tooltip_temp')} position="top">
                            <div className="bg-zinc-900 border border-zinc-800 rounded px-2 py-1 flex items-center gap-1.5 cursor-help">
                                <Zap size={10} className="text-amber-400" />
                                <span className="text-[10px] text-zinc-400 font-mono">{(agent.model_config?.temperature || 0.7).toFixed(1)} TEMP</span>
                            </div>
                        </Tooltip>
                        <Tooltip content={i18n.t('agent_manager.tooltip_credits')} position="top">
                            <div className="bg-zinc-900 border border-zinc-800 rounded px-2 py-1 flex items-center gap-1.5 cursor-help">
                                <DollarSign size={10} className="text-green-400" />
                                <span className="text-[10px] text-zinc-400 font-mono">
                                    ${(agent.cost_usd || 0).toFixed(3)} / ${agent.budget_usd || '0'}
                                </span>
                            </div>
                        </Tooltip>

                        {(agent.skills?.length || 0) > 0 && (
                            <Tooltip content={`${agent.skills?.join(', ')}`} position="top">
                                <div className="bg-green-500/10 border border-green-500/20 rounded px-2 py-1 flex items-center gap-1.5 cursor-help">
                                    <Code size={10} className="text-green-400" />
                                    <span className="text-[10px] text-green-400 font-mono">{agent.skills?.length} SKILLS</span>
                                </div>
                            </Tooltip>
                        )}

                        {(agent.workflows?.length || 0) > 0 && (
                            <Tooltip content={`${agent.workflows?.join(', ')}`} position="top">
                                <div className="bg-amber-500/10 border border-amber-500/20 rounded px-2 py-1 flex items-center gap-1.5 cursor-help">
                                    <FileText size={10} className="text-amber-400" />
                                    <span className="text-[10px] text-amber-400 font-mono">{agent.workflows?.length} WFK</span>
                                </div>
                            </Tooltip>
                        )}

                        {(agent.mcp_tools?.length || 0) > 0 && (
                            <Tooltip content={`${agent.mcp_tools?.join(', ')}`} position="top">
                                <div className="bg-cyan-500/10 border border-cyan-500/20 rounded px-2 py-1 flex items-center gap-1.5 cursor-help">
                                    <Terminal size={10} className="text-cyan-400" />
                                    <span className="text-[10px] text-cyan-400 font-mono">{agent.mcp_tools?.length} MCP</span>
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
        p.cost_usd === n.cost_usd &&
        p.theme_color === n.theme_color &&
        p.name === n.name &&
        p.role === n.role &&
        p.model === n.model &&
        p.model_2 === n.model_2 &&
        p.model_3 === n.model_3 &&
        p.active_model_slot === n.active_model_slot &&
        p.model_config?.temperature === n.model_config?.temperature &&
        p.skills?.length === n.skills?.length &&
        p.workflows?.length === n.workflows?.length &&
        p.mcp_tools?.length === n.mcp_tools?.length &&
        p.metadata?.has_participated_in_swarm === n.metadata?.has_participated_in_swarm &&
        p.active_mission?.id === n.active_mission?.id
    );
});


// Metadata: [Agent_Manager]

// Metadata: [Agent_Manager]
