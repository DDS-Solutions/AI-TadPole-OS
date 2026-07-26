/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Visual hierarchy for the agent swarm organization. 
 * Orchestrates the rendering of reporting structures and team/departmental relationships.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Recursive rendering loop for circular reporting structures, or node occlusion on large swarms.
 * - **Telemetry Link**: Search for `[Org_Chart]` or HIERARCHY_RENDER in UI logs.
 */

import { useState, useEffect, useMemo, memo, useCallback, useRef } from 'react';
import { resolve_provider } from '../utils/model_utils';
import { agent_service } from '../services/agent_service';
import type { Agent } from '../types';
import { Hierarchy_Node } from '../components/Hierarchy_Node';
import AgentConfigPanel from '../components/AgentConfigPanel';
import { agents as mock_agents } from '../data/mock_agents';
import { use_workspace_store } from '../stores/workspace_store';
import { use_dropdown_store } from '../stores/dropdown_store';
import { use_agent_store } from '../stores/agent_store';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { i18n } from '../i18n';

const DEFAULT_MODEL = 'gemini-1.5-flash';
const ROOT_CLUSTER_ID = 'cl-command';

type ThemeKey = 'cyan' | 'zinc' | 'amber';

const THEME_CLASSES: Record<ThemeKey, { bg: string; pulse: string; heading: string }> = {
    cyan: { bg: 'bg-cyan-500/20', pulse: 'vertical-pulse text-cyan-500', heading: 'text-cyan-400' },
    zinc: { bg: 'bg-zinc-500/20', pulse: 'vertical-pulse text-zinc-500', heading: 'text-zinc-400' },
    amber: { bg: 'bg-amber-500/20', pulse: 'vertical-pulse text-amber-500', heading: 'text-amber-400' },
};

const getThemeClasses = (theme: string) => {
    const key = (theme === 'cyan' || theme === 'amber' ? theme : 'zinc') as ThemeKey;
    return THEME_CLASSES[key];
};

const JSON_LD = JSON.stringify({
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    "name": "Tadpole OS Organization Chart",
    "description": "Visual hierarchy and reporting structure for the autonomous agent swarm. Displays command-and-control relationships and cluster allocations.",
    "author": { "@type": "Organization", "name": "Sovereign Engineering" },
    "applicationCategory": "Organization Management",
    "operatingSystem": "Tadpole OS"
});

/**
 * Org_Chart
 * The Neural Command Hierarchy page.
 */
export default function Org_Chart() {
    const { clusters } = use_workspace_store();
    const agents_list = use_agent_store(s => s.agents);
    const dropdown_open_id = use_dropdown_store(s => s.open_id);
    const close_dropdowns = use_dropdown_store(s => s.close_dropdown);

    const [config_agent_id, set_config_agent_id] = useState<string | null>(null);
    const hasLoadedRef = useRef(false);
    const hasWarnedCycleRef = useRef(false);

    // O(1)-efficient Map caching of agents list
    const agentMap = useMemo(() => new Map(agents_list.map(a => [a.id, a])), [agents_list]);
    const effective_config_agent_id = useMemo(() => {
        if (!config_agent_id || config_agent_id === 'new') return config_agent_id;
        return agentMap.has(config_agent_id) ? config_agent_id : null;
    }, [agentMap, config_agent_id]);

    useEffect(() => {
        if (agents_list.length === 0 && !hasLoadedRef.current) {
            hasLoadedRef.current = true;
            agent_service.load_agents_into_store().catch(e => {
                console.error('[Org_Chart] load_agents_into_store failed:', e);
            });
        }
    }, [agents_list.length]);

    // Populate available roles from both mock fallback and live registry
    const available_roles = useMemo(() =>
        Array.from(new Set([
            ...mock_agents.map(a => a.role),
            ...agents_list.map(a => a.role)
        ])).sort()
        , [agents_list]);

    // ── Memoized Handlers ──────────────────────────────────────────────

    const handle_agent_update = useCallback((agent_id: string, updates: Partial<Agent>) => {
        agent_service.update_agent(agent_id, updates).catch(e => {
            console.error('[Org_Chart] handle_agent_update failed:', e);
        });
    }, []);

    const handle_role_change = useCallback((agent_id: string, new_role: string) => {
        handle_agent_update(agent_id, { role: new_role });
    }, [handle_agent_update]);

    const handle_skill_trigger = useCallback(async (agent_id: string, skill: string, slot: 1 | 2 | 3 = 1) => {
        const agents = use_agent_store.getState().agents;
        const agent = agents.find(a => a.id === agent_id);
        if (!agent) return;

        void agent_service.update_agent(agent_id, {
            status: 'active' as const,
            current_task: i18n.t('org_chart.executing_skill', { skill, defaultValue: `⚡ Executing: ${skill}...` }),
            active_model_slot: slot
        }).catch(e => console.error('[Org_Chart] Skill status update failed:', e));

        try {
            const freshAgent = use_agent_store.getState().agents.find(a => a.id === agent_id) || agent;
            let model_id = freshAgent.model || DEFAULT_MODEL;
            let provider = freshAgent.model_config?.provider || 'google';

            if (slot === 2) {
                model_id = freshAgent.model_2 || model_id;
                provider = freshAgent.model_config2?.provider || provider;
            } else if (slot === 3) {
                model_id = freshAgent.model_3 || model_id;
                provider = freshAgent.model_config3?.provider || provider;
            }

            const department = freshAgent.department || 'Operations';

            const activeClusters = use_workspace_store.getState().clusters;
            const agent_cluster = activeClusters.find(c => c.collaborators.includes(agent_id));
            const success = await tadpole_os_service.send_command(
                agent_id,
                skill,
                model_id,
                provider,
                agent_cluster?.id,
                department,
                agent_cluster?.budget_usd
            );

            if (!success) {
                void agent_service.update_agent(agent_id, { status: 'idle' }).catch(() => {});
            }
        } catch (e) {
            console.error("❌ [OrgChart] Failed to trigger skill:", e);
            void agent_service.update_agent(agent_id, { status: 'idle' }).catch(() => {});
        }
    }, []);

    // Parameterized Model Change Handler
    const handle_model_change_by_slot = useCallback((agent_id: string, new_model: string, slot: 1 | 2 | 3 = 1) => {
        const agent = use_agent_store.getState().agents.find(a => a.id === agent_id);
        if (!agent) return;
        const provider = resolve_provider(new_model);

        const configKey = slot === 1 ? 'model_config' : slot === 2 ? 'model_config2' : 'model_config3';
        const modelKey = slot === 1 ? 'model' : slot === 2 ? 'model_2' : 'model_3';

        const currentConfig = agent[configKey];
        const model_config = currentConfig
            ? { ...currentConfig, modelId: new_model, provider }
            : { modelId: new_model, provider };

        handle_agent_update(agent_id, {
            [modelKey]: new_model,
            [configKey]: model_config
        });
    }, [handle_agent_update]);

    const handle_model_change = useCallback((id: string, m: string) => handle_model_change_by_slot(id, m, 1), [handle_model_change_by_slot]);
    const handle_model_2_change = useCallback((id: string, m: string) => handle_model_change_by_slot(id, m, 2), [handle_model_change_by_slot]);
    const handle_model_3_change = useCallback((id: string, m: string) => handle_model_change_by_slot(id, m, 3), [handle_model_change_by_slot]);

    const handle_configure_click = useCallback((id: string) => {
        set_config_agent_id(id);
    }, []);

    // Bundle shared node callbacks to eliminate redundant prop drilling across tree levels
    const shared_node_props = useMemo(() => ({
        available_roles,
        on_role_change: handle_role_change,
        on_skill_trigger: handle_skill_trigger,
        on_configure_click: handle_configure_click,
        on_model_change: handle_model_change,
        on_model_2_change: handle_model_2_change,
        on_model_3_change: handle_model_3_change,
        on_update: handle_agent_update,
    }), [available_roles, handle_role_change, handle_skill_trigger, handle_configure_click, handle_model_change, handle_model_2_change, handle_model_3_change, handle_agent_update]);

    // ── Data Partitioning ──────────────────────────────────
    const hierarchy_data = useMemo(() => {
        if (agents_list.length === 0) return null;

        const combined_agents = [...agents_list];

        // Resolve Root Alpha (CEO / Lead Agent) and Nexus (COO / Sub-Lead) via role & name matching before falling back to ID
        const alpha = combined_agents.find(a => a.role === 'CEO' || a.name === 'Agent of Nine')
            || combined_agents.find(a => a.id === '1')
            || combined_agents[0];

        const nexus = combined_agents.find(a => (a.role === 'COO' || a.name === 'Tadpole Alpha' || a.name === 'Tadpole') && a.id !== alpha.id)
            || combined_agents.find(a => a.id === '2' && a.id !== alpha.id)
            || combined_agents.find(a => a.id !== alpha.id);

        const used_higher_ids = new Set([alpha.id, nexus?.id].filter(Boolean));

        // Map remaining clusters to chains (excluding cl-command which forms the root/nexus)
        const chain_clusters = clusters.filter(c => c.id !== ROOT_CLUSTER_ID);
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
                .filter((a): a is Agent => Boolean(a))
        }));

        return { alpha, nexus, chains };
    }, [agents_list, clusters]);

    // Cycle Detection: Warn if any loop brings Alpha or Nexus back into collaborator chains
    useEffect(() => {
        if (!hierarchy_data) return;
        const { alpha, nexus, chains } = hierarchy_data;
        const all_chain_ids = chains.flatMap(c => c.agents.map(a => a.id));
        const hasCycle = all_chain_ids.includes(alpha.id) || (nexus && all_chain_ids.includes(nexus.id));

        if (hasCycle) {
            if (!hasWarnedCycleRef.current) {
                console.warn('[Org_Chart] Cycle detected in cluster hierarchy: Alpha or Nexus is present in collaborator chains.');
                hasWarnedCycleRef.current = true;
            }
        } else {
            hasWarnedCycleRef.current = false;
        }
    }, [hierarchy_data]);

    const config_agent = useMemo(() => {
        if (!effective_config_agent_id || effective_config_agent_id === 'new') return undefined;
        return agentMap.get(effective_config_agent_id);
    }, [effective_config_agent_id, agentMap]);

    const alphaCluster = useMemo(() => {
        if (!hierarchy_data?.alpha?.id) return undefined;
        const alphaId = hierarchy_data.alpha.id;
        return clusters.find(c => c.collaborators.includes(alphaId));
    }, [clusters, hierarchy_data]);

    const nexusCluster = useMemo(() => {
        if (!hierarchy_data?.nexus?.id) return undefined;
        const nexusId = hierarchy_data.nexus.id;
        return clusters.find(c => c.collaborators.includes(nexusId));
    }, [clusters, hierarchy_data]);

    const xCoords = useMemo(() => {
        if (!hierarchy_data) return [];
        const count = hierarchy_data.chains.length;
        if (count === 0) return [];
        if (count === 1) return [500];
        const startX = 100;
        const endX = 900;
        const coords = [];
        for (let i = 0; i < count; i++) {
            coords.push(startX + i * ((endX - startX) / (count - 1)));
        }
        return coords;
    }, [hierarchy_data]);

    const branchPath = useMemo(() => {
        if (xCoords.length === 0) return '';
        return 'M 500 0 V 20' + xCoords.map(x => {
            if (Math.abs(x - 500) < 1) return ' M 500 20 V 52';
            const isLeft = x < 500;
            const turnX1 = isLeft ? 480 : 520;
            const turnX2 = isLeft ? x + 20 : x - 20;
            return ` M 500 20 Q 500 30 ${turnX1} 30 H ${turnX2} Q ${x} 30 ${x} 40 V 52`;
        }).join('');
    }, [xCoords]);

    if (!hierarchy_data) {
        return (
            <div className="h-full flex items-center justify-center text-zinc-500 animate-pulse font-mono text-xs uppercase tracking-widest">
                {i18n.t('org_chart.label_initializing')}
            </div>
        );
    }

    return (
        <div className="h-full flex flex-col bg-[color:var(--color-background)]">
            <script
                type="application/ld+json"
                dangerouslySetInnerHTML={{ __html: JSON_LD }} // security-scan:ignore
            />
            <h1 className="sr-only">Tadpole OS Neural Command Hierarchy & Swarm Organization</h1>

            <div
                className="flex-1 overflow-auto p-8 custom-scrollbar relative"
                onClick={close_dropdowns}
            >
                <div className="neural-grid" />

                <div className="min-w-max pt-1 pb-12 px-12 flex flex-col items-center gap-12 relative">

                    {/* Level 1: Root (Alpha) */}
                    <div className={`relative group w-[350px] ${dropdown_open_id === hierarchy_data.alpha?.id ? 'z-[100]' : 'z-30'}`}>
                        <div className="mb-4 text-center">
                            <h3 className="text-[10px] font-bold uppercase tracking-[0.2em] mb-1 text-green-400">
                                {i18n.t('org_chart.label_command_chain')}
                            </h3>
                            <p className="text-[9px] text-zinc-500 font-medium">{i18n.t('org_chart.label_strategic_command')}</p>
                        </div>
                        <div className="absolute -inset-4 bg-green-500/10 blur-2xl rounded-full opacity-0 group-hover:opacity-100 transition-opacity duration-700" />

                        <Hierarchy_Node
                            is_root
                            is_alpha
                            agent={hierarchy_data.alpha}
                            theme_color="blue"
                            is_active={alphaCluster?.is_active}
                            mission_objective={alphaCluster?.objective}
                            {...shared_node_props}
                        />

                        {/* Connection to Nexus */}
                        <div
                            aria-hidden="true"
                            className={`absolute top-full left-1/2 -translate-x-1/2 h-[30px] w-px bg-gradient-to-b from-green-500/50 to-green-500/20 ${(hierarchy_data.nexus?.status !== 'offline' && hierarchy_data.nexus?.status !== 'idle') || hierarchy_data.chains.some(c => c.is_active) ? 'vertical-pulse text-green-500' : ''}`}
                        />
                    </div>

                    {/* Level 2: Nexus (Coordinator) */}
                    <div className={`relative pt-0 mt-[-18px] w-[350px] ${dropdown_open_id === hierarchy_data.nexus?.id ? 'z-[100]' : 'z-20'}`}>
                        <div className="absolute -inset-4 bg-zinc-500/5 blur-xl rounded-full opacity-50" />

                        <Hierarchy_Node
                            agent={hierarchy_data.nexus}
                            theme_color="zinc"
                            is_active={nexusCluster?.is_active}
                            mission_objective={nexusCluster?.objective}
                            {...shared_node_props}
                        />

                        {/* Branching SVG (Fluid Neural Pathing) */}
                        <svg
                            aria-hidden="true"
                            className="absolute top-[100%] left-1/2 -translate-x-1/2 w-[1000px] h-[52px] overflow-visible pointer-events-none"
                        >
                            <defs>
                                <linearGradient id="neural-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
                                    <stop offset="0%" stopColor="rgba(113, 113, 122, 0.1)" />
                                    <stop offset="50%" stopColor="rgba(16, 185, 129, 0.4)" />
                                    <stop offset="100%" stopColor="rgba(113, 113, 122, 0.1)" />
                                </linearGradient>
                            </defs>
                            <path
                                d={branchPath}
                                fill="none"
                                stroke="rgba(113, 113, 122, 0.2)"
                                strokeWidth="1.5"
                                className="transition-all duration-1000"
                            />
                            {hierarchy_data.chains.some(c => c.is_active) && (
                                <path
                                    d={branchPath}
                                    fill="none"
                                    stroke="url(#neural-gradient)"
                                    strokeWidth="2"
                                    strokeDasharray="10 20"
                                    className="animate-[dash_3s_linear_infinite]"
                                />
                            )}
                            {xCoords.map(x => (
                                <circle key={x} cx={x} cy={x === 500 ? 20 : 30} r="1.5" fill="rgba(113, 113, 122, 0.4)" />
                            ))}
                        </svg>
                    </div>

                    {/* Level 3: Chains */}
                    <div className={`flex gap-16 relative ${hierarchy_data.chains.some(c => c.agents.some(a => a.id === dropdown_open_id)) ? 'z-[100]' : 'z-10'}`}>
                        {hierarchy_data.chains.map(chain => (
                            <Agent_Chain
                                key={chain.id || `chain-${chain.agents[0]?.id ?? 'empty'}`}
                                chain={chain}
                                dropdown_open_id={dropdown_open_id}
                                clusters={clusters}
                                shared_node_props={shared_node_props}
                            />
                        ))}
                    </div>

                    {/* Agent Config Panel Overlay */}
                    {effective_config_agent_id && (effective_config_agent_id === 'new' || config_agent) && (
                        <AgentConfigPanel
                            agent={config_agent}
                            onClose={() => set_config_agent_id(null)}
                            onUpdate={handle_agent_update}
                        />
                    )}

                    {/* Overlay Indicators */}
                    <div className="fixed bottom-8 right-8 flex flex-col gap-2 items-end">
                        <div className="px-3 py-1 bg-black/60 border border-[color:var(--color-border)] rounded-full backdrop-blur-md flex items-center gap-2 shadow-2xl">
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

Org_Chart.displayName = 'Org_Chart';

type SharedNodeProps = {
    available_roles: string[];
    on_role_change: (id: string, role: string) => void;
    on_skill_trigger: (id: string, skill: string, slot?: 1 | 2 | 3) => void;
    on_configure_click: (id: string) => void;
    on_model_change: (id: string, m: string) => void;
    on_model_2_change: (id: string, m: string) => void;
    on_model_3_change: (id: string, m: string) => void;
    on_update: (id: string, updates: Partial<Agent>) => void;
};

type AgentChainProps = {
    chain: {
        id: string;
        name: string;
        theme: string;
        alpha_id?: string;
        objective?: string;
        is_active?: boolean;
        agents: Agent[];
    };
    dropdown_open_id: string | null;
    clusters: { id: string; name: string; theme: string; alpha_id?: string; collaborators: string[]; is_active?: boolean }[];
    shared_node_props: SharedNodeProps;
};

/**
 * Agent_Chain
 * Optimized sub-component for rendering an individual agent chain.
 */
const Agent_Chain = memo(({
    chain,
    dropdown_open_id,
    clusters,
    shared_node_props
}: AgentChainProps) => {
    const chainTheme = chain.theme || 'zinc';
    const currentThemeClasses = getThemeClasses(chainTheme);

    return (
        <div className="flex flex-col items-center gap-12 relative">
            <div className="mb-4 text-center">
                <h3 className={`text-[10px] font-bold uppercase tracking-[0.2em] mb-1 ${currentThemeClasses.heading}`}>
                    Chain {chain.id}
                </h3>
                <p className="text-[9px] text-zinc-500 font-medium">{chain.name}</p>
            </div>

            <div className="flex flex-col gap-12 relative">
                {chain.agents.map((agent: Agent, idx: number) => (
                    <div key={agent.id} className="relative w-[350px]" style={{ zIndex: dropdown_open_id === agent.id ? 110 : (100 - idx) }}>
                        <Hierarchy_Node
                            agent={agent}
                            theme_color={chainTheme}
                            is_alpha={agent.id === chain.alpha_id}
                            is_active={clusters.find(c => c.id === chain.id)?.is_active}
                            mission_objective={chain.objective}
                            {...shared_node_props}
                        />

                        {idx < chain.agents.length - 1 && (
                            <div
                                aria-hidden="true"
                                className={`absolute top-full left-1/2 -translate-x-1/2 h-12 w-px 
                                ${currentThemeClasses.bg}
                                ${chain.is_active || (chain.agents[idx].status !== 'offline' && chain.agents[idx].status !== 'idle') || (chain.agents[idx + 1].status !== 'offline' && chain.agents[idx + 1].status !== 'idle') ? currentThemeClasses.pulse : ''}`}
                            />
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
});

Agent_Chain.displayName = 'Agent_Chain';

// Metadata: [Org_Chart]
