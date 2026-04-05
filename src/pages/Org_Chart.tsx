/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Visual hierarchy for the agent swarm organization. 
 * Orchestrates the rendering of reporting structures and team/departmental relationships.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Recursive rendering loop for circular reporting structures, or node occlusion on large swarms.
 * - **Telemetry Link**: Search for `[Org_Chart]` or `HIERARCHY_RENDER` in UI logs.
 */

/**
 * Org_Chart
 * The Neural Command Hierarchy page.
 * Renders a multi-level agent organization chart with real-time status updates,
 * dynamic pulsating connection animations, and integrated node configuration. 
 * Refactored for strict snake_case compliance for backend parity.
 */
import { useState, useEffect, useMemo, memo } from 'react';
import { resolve_provider } from '../utils/model_utils';
import type { Agent } from '../types';
import { Hierarchy_Node } from '../components/Hierarchy_Node';
import Agent_Config_Panel from '../components/Agent_Config_Panel';
import { agents as mock_agents } from '../data/mock_agents';
import { use_workspace_store } from '../stores/workspace_store';
import { use_dropdown_store } from '../stores/dropdown_store';
import { use_agent_store } from '../stores/agent_store';
import { i18n } from '../i18n';

export default function Org_Chart() {
    const { clusters } = use_workspace_store();
    const { agents: agents_list, fetch_agents, update_agent: store_update_agent } = use_agent_store();
    const dropdown_open_id = use_dropdown_store(s => s.open_id);
    const close_dropdowns = use_dropdown_store(s => s.close_dropdown);

    const [config_agent_id, set_config_agent_id] = useState<string | null>(null);

    const available_roles = useMemo(() =>
        Array.from(new Set(mock_agents.map(a => a.role))).sort()
        , []);

    useEffect(() => {
        if (agents_list.length === 0) {
            fetch_agents();
        }
    }, [agents_list.length, fetch_agents]);

    // ── Handlers ──────────────────────────────────────────────
    const handle_skill_trigger = (agent_id: string, skill: string) => {
        store_update_agent(agent_id, { 
            status: 'active' as const, 
            current_task: `⚡ Executing: ${skill}...` 
        });
    };

    const handle_role_change = (agent_id: string, new_role: string) => {
        handle_agent_update(agent_id, { role: new_role });
    };

    const handle_agent_update = (agent_id: string, updates: Partial<Agent>) => {
        store_update_agent(agent_id, updates);
    };

    const handle_model_change = (agent_id: string, new_model: string) => {
        const provider = resolve_provider(new_model);
        handle_agent_update(agent_id, { model: new_model, model_config: { model_id: new_model, provider } });
    };

    const handle_model_2_change = (agent_id: string, new_model: string) => {
        const provider = resolve_provider(new_model);
        handle_agent_update(agent_id, { model2: new_model, model_config2: { model_id: new_model, provider } });
    };

    const handle_model_3_change = (agent_id: string, new_model: string) => {
        const provider = resolve_provider(new_model);
        handle_agent_update(agent_id, { model3: new_model, model_config3: { model_id: new_model, provider } });
    };

    // ── Data Partitioning ──────────────────────────────────
    const hierarchy_data = useMemo(() => {
        if (agents_list.length === 0) return null;

        const combined_agents = [...agents_list];

        // Level 1: Alpha (Force Agent of Nine)
        const alpha = combined_agents.find(a => a.name === 'Agent of Nine' || a.id === '1') || combined_agents[0];

        const nexus = combined_agents.find(a => (a.name === 'Tadpole' || a.id === '2') && a.id !== alpha.id)
            || combined_agents.find(a => a.id !== alpha.id);

        const used_higher_ids = new Set([alpha.id, nexus?.id].filter(Boolean));

        // Map remaining clusters to chains (excluding cl-command which forms the root/nexus)
        const chain_clusters = clusters.filter(c => c.id !== 'cl-command');
        const chains = chain_clusters.slice(0, 3).map(cluster => ({
            id: cluster.id,
            name: cluster.name,
            theme: cluster.theme,
            alpha_id: cluster.alpha_id,
            objective: cluster.objective,
            is_active: cluster.is_active,
            agents: cluster.collaborators
                .filter(cid => !used_higher_ids.has(cid))
                .map(cid => combined_agents.find(a => a.id === cid))
                .filter(Boolean) as Agent[]
        }));

        return {
            alpha, nexus, chains
        };
    }, [agents_list, clusters]);

    if (!hierarchy_data) {
        return (
            <div className="h-full flex items-center justify-center text-zinc-500 animate-pulse font-mono text-xs uppercase tracking-widest">
                {i18n.t('org_chart.label_initializing')}
            </div>
        );
    }

    return (
        <div className="h-full flex flex-col bg-zinc-950">

            <div className="flex-1 overflow-auto p-8 custom-scrollbar relative" onClick={close_dropdowns}>
                <div className="neural-grid" />

                <div className="min-w-max pt-1 pb-12 px-12 flex flex-col items-center gap-12 relative">

                    {/* Level 1: Root (Alpha) */}
                    <div className={`relative group w-[350px] ${dropdown_open_id === hierarchy_data.alpha?.id ? 'z-[100]' : 'z-30'}`}>
                        <div className="mb-4 text-center">
                            <h3 className="text-[10px] font-bold uppercase tracking-[0.2em] mb-1 text-blue-400">
                                {i18n.t('org_chart.label_command_chain')}
                            </h3>
                            <p className="text-[9px] text-zinc-500 font-medium">{i18n.t('org_chart.label_strategic_command')}</p>
                        </div>
                        <div className="absolute -inset-4 bg-blue-500/10 blur-2xl rounded-full opacity-0 group-hover:opacity-100 transition-opacity duration-700" />
                        {(() => {
                            const cluster = clusters.find(c => c.collaborators.includes(hierarchy_data.alpha?.id));
                            return (
                                <Hierarchy_Node
                                    is_root
                                    is_alpha
                                    available_roles={available_roles}
                                    on_role_change={handle_role_change}
                                    agent={hierarchy_data.alpha}
                                    theme_color="blue"
                                    is_active={cluster?.is_active}
                                    mission_objective={cluster?.objective}
                                    on_skill_trigger={handle_skill_trigger}
                                    on_configure_click={(id) => set_config_agent_id(id)}
                                    on_model_change={handle_model_change}
                                    on_model_2_change={handle_model_2_change}
                                    on_model_3_change={handle_model_3_change}
                                    on_update={handle_agent_update}
                                />
                            );
                        })()}

                        {/* Connection to Nexus */}
                        <div 
                            aria-hidden="true"
                            className={`absolute top-full left-1/2 -translate-x-1/2 h-[30px] w-px bg-gradient-to-b from-blue-500/50 to-blue-500/20 ${(hierarchy_data.nexus?.status !== 'offline' && hierarchy_data.nexus?.status !== 'idle') || hierarchy_data.chains.some(c => c.is_active) ? 'vertical-pulse text-blue-500' : ''}`} 
                        />
                    </div>

                    {/* Level 2: Nexus (Coordinator) */}
                    <div className={`relative pt-0 mt-[-18px] w-[350px] ${dropdown_open_id === hierarchy_data.nexus?.id ? 'z-[100]' : 'z-20'}`}>
                        <div className="absolute -inset-4 bg-zinc-500/5 blur-xl rounded-full opacity-50" />
                        {(() => {
                            const cluster = clusters.find(c => hierarchy_data.nexus?.id && c.collaborators.includes(hierarchy_data.nexus.id));
                            return (
                                <Hierarchy_Node
                                    agent={hierarchy_data.nexus}
                                    available_roles={available_roles}
                                    on_role_change={handle_role_change}
                                    theme_color="zinc"
                                    is_active={cluster?.is_active}
                                    mission_objective={cluster?.objective}
                                    on_skill_trigger={handle_skill_trigger}
                                    on_configure_click={(id) => set_config_agent_id(id)}
                                    on_model_change={handle_model_change}
                                    on_model_2_change={handle_model_2_change}
                                    on_model_3_change={handle_model_3_change}
                                    on_update={handle_agent_update}
                                />
                            );
                        })()}

                        {/* Branching SVG */}
                        <svg 
                            aria-hidden="true"
                            className="absolute top-[100%] left-1/2 -translate-x-1/2 w-[1000px] h-[42px] overflow-visible pointer-events-none"
                        >
                            <path
                                d="M 500 0 L 500 30 M 500 30 L 100 30 M 500 30 L 900 30 M 100 30 L 100 42 M 500 30 L 500 42 M 900 30 L 900 42"
                                fill="none"
                                stroke="rgba(113, 113, 122, 0.3)"
                                strokeWidth="1"
                                className={hierarchy_data.chains.some(c => c.is_active || c.agents.some(a => a.status !== 'offline' && a.status !== 'idle')) ? 'neural-pulse text-zinc-500' : ''}
                            />
                            <circle cx="100" cy="30" r="2" fill="rgba(113, 113, 122, 0.5)" />
                            <circle cx="900" cy="30" r="2" fill="rgba(113, 113, 122, 0.5)" />
                        </svg>
                    </div>

                    {/* Level 3: Chains */}
                    <div className={`flex gap-16 relative ${hierarchy_data.chains.some(c => c.agents.some(a => a.id === dropdown_open_id)) ? 'z-[100]' : 'z-10'}`}>
                        {hierarchy_data.chains.map(chain => (
                            <Agent_Chain
                                key={chain.id}
                                chain={chain}
                                dropdown_open_id={dropdown_open_id}
                                available_roles={available_roles}
                                handle_role_change={handle_role_change}
                                handle_skill_trigger={handle_skill_trigger}
                                set_config_agent_id={set_config_agent_id}
                                handle_agent_update={handle_agent_update}
                                clusters={clusters}
                                handle_model_change={handle_model_change}
                                handle_model_2_change={handle_model_2_change}
                                handle_model_3_change={handle_model_3_change}
                            />
                        ))}
                    </div>

                    {/* Agent Config Panel Overlay */}
                    {config_agent_id && (() => {
                            const agent = agents_list.find(a => a.id === config_agent_id) || mock_agents.find(a => a.id === config_agent_id);
                            return agent ? (
                                <Agent_Config_Panel
                                    agent={agent as Agent}
                                    on_close={() => set_config_agent_id(null)}
                                    on_update={handle_agent_update}
                                />
                            ) : null;
                        })()}

                    {/* Overlay Indicators */}
                    <div className="fixed bottom-8 right-8 flex flex-col gap-2 items-end">
                        <div className="px-3 py-1 bg-black/60 border border-zinc-800 rounded-full backdrop-blur-md flex items-center gap-2 shadow-2xl">
                            <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]" />
                            <span className="text-[10px] font-mono text-zinc-400 uppercase tracking-widest">
                                {i18n.t('org_chart.label_swarm_active')}
                            </span>
                        </div>
                    </div>

                </div>
            </div>
        </div>
    );
}

/**
 * Agent_Chain
 * Optimized sub-component for rendering an individual agent chain.
 */
const Agent_Chain = memo(({
    chain,
    dropdown_open_id,
    available_roles,
    handle_role_change,
    handle_skill_trigger,
    set_config_agent_id,
    handle_agent_update,
    clusters,
    handle_model_change,
    handle_model_2_change,
    handle_model_3_change
}: {
    chain: {
        id: string;
        name: string;
        theme: string;
        alpha_id?: string;
        objective?: string;
        is_active?: boolean;
        agents: Agent[];
    },
    dropdown_open_id: string | null,
    available_roles: string[],
    handle_role_change: (id: string, role: string) => void,
    handle_skill_trigger: (id: string, skill: string) => void,
    set_config_agent_id: (id: string) => void,
    handle_agent_update: (id: string, updates: Partial<Agent>) => void,
    handle_model_change: (id: string, m: string) => void,
    handle_model_2_change: (id: string, m: string) => void,
    handle_model_3_change: (id: string, m: string) => void,
    clusters: { id: string, name: string, theme: string, alpha_id?: string, collaborators: string[], is_active?: boolean }[]
}) => {
    return (
        <div className="flex flex-col items-center gap-12 relative">
            <div className="mb-4 text-center">
                <h3 className={`text-[10px] font-bold uppercase tracking-[0.2em] mb-1 
                ${chain.theme === 'cyan' ? 'text-cyan-400' :
                        chain.theme === 'zinc' ? 'text-zinc-400' : 'text-amber-400'}`}>
                    Chain {chain.id}
                </h3>
                <p className="text-[9px] text-zinc-500 font-medium">{chain.name}</p>
            </div>

            <div className="flex flex-col gap-12 relative">
                {chain.agents.map((agent: Agent, idx: number) => (
                    <div key={agent.id} className="relative w-[350px]" style={{ zIndex: dropdown_open_id === agent.id ? 110 : (100 - idx) }}>
                        <Hierarchy_Node
                            agent={agent}
                            available_roles={available_roles}
                            on_role_change={handle_role_change}
                            theme_color={chain.theme}
                            is_alpha={agent.id === chain.alpha_id}
                            is_active={clusters.find(c => c.id === chain.id)?.is_active}
                            mission_objective={chain.objective}
                            on_skill_trigger={handle_skill_trigger}
                            on_configure_click={(id) => set_config_agent_id(id)}
                            on_model_change={handle_model_change}
                            on_model_2_change={handle_model_2_change}
                            on_model_3_change={handle_model_3_change}
                            on_update={handle_agent_update}
                        />

                        {idx < chain.agents.length - 1 && (
                            <div 
                                aria-hidden="true"
                                className={`absolute top-full left-1/2 -translate-x-1/2 h-12 w-px 
                                ${chain.theme === 'cyan' ? 'bg-cyan-500/20' :
                                        chain.theme === 'zinc' ? 'bg-zinc-500/20' : 'bg-amber-500/20'}
                                ${chain.is_active || (chain.agents[idx].status !== 'offline' && chain.agents[idx].status !== 'idle') || (chain.agents[idx + 1].status !== 'offline' && chain.agents[idx + 1].status !== 'idle') ? `vertical-pulse text-${chain.theme}-500` : ''}`}
                            />
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
});

