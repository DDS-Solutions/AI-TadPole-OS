/**
 * @docs ARCHITECTURE:Interface
 * @docs OPERATIONS_MANUAL:Navigation
 * 
 * ### AI Assist Note
 * **Root View**: The central command-and-control center for the Tadpole OS agent swarm. 
 * Orchestrates real-time telemetry, swarm visualization, and multi-agent task dispatching. 
 * Integrates `use_dashboard_data` for unified state management and handles complex deployment/skill-triggering logic.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Deployment error (logged to `event_bus`), skill trigger timeout (API 408/504), or UI freezing during high-velocity swarm pulses.
 * - **Telemetry Link**: Search for `[OpsDashboard]` in component traces or check `Swarm_Visualizer` health.
 */

import { useState, useEffect, useMemo, useCallback } from 'react';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { resolve_provider } from '../utils/model_utils';
import { event_bus } from '../services/event_bus';
import { use_header_store, type Header_State } from '../stores/header_store';
import { useDashboardData } from '../hooks/use_dashboard_data';
import { use_dropdown_store, type Dropdown_State } from '../stores/dropdown_store';
import { use_role_store } from '../stores/role_store';
import { i18n } from '../i18n';

import TerminalComponent from '../components/Terminal';
import Agent_Config_Panel from '../components/Agent_Config_Panel';
import Error_Boundary from '../components/Error_Boundary';

import { Dashboard_Header_Actions } from '../components/dashboard/Dashboard_Header_Actions';
import { Stat_Metrics } from '../components/dashboard/Stat_Metrics';
import { Agent_Status_Grid } from '../components/dashboard/Agent_Status_Grid';
import { Swarm_Visualizer } from '../components/Swarm_Visualizer';
import type { Agent } from '../types';

/**
 * Ops_Dashboard
 * 
 * The central command-and-control center for the Tadpole OS agent swarm.
 */
export default function Ops_Dashboard() {
    const {
        agents_list, agents_count, active_agents, total_cost, total_tokens, budget_util,
        nodes, nodes_loading, assigned_agent_ids, available_roles,
        clusters, toggle_cluster_active, update_agent, add_agent, discover_nodes, recruit_velocity
    } = useDashboardData();

    const [deploying_target, set_deploying_target] = useState<string | null>(null);
    const [config_agent_id, set_config_agent_id] = useState<string | null>(null);

    const set_header_actions = use_header_store((s: Header_State) => s.set_header_actions);
    const close_dropdowns = use_dropdown_store((s: Dropdown_State) => s.close_dropdown);

    // ── Handlers ──────────────────────────────────────────────

    const handle_agent_update = (id: string, updates: Partial<Agent>) => {
        update_agent(id, updates);
    };

    const handle_create_agent = async (params: Partial<Agent>) => {
        try {
            const new_id = `agent-${Math.random().toString(36).substring(2, 11)}`;
            const new_agent: Agent = {
                id: new_id,
                name: params.name || i18n.t('ops.placeholder_name'),
                role: params.role || 'assistant',
                department: params.department || 'Operations',
                status: 'idle',
                tokens_used: 0,
                model: params.model || 'gemini-1.5-flash',
                skills: params.skills || [],
                workflows: params.workflows || [],
                cost_usd: 0,
                budget_usd: params.budget_usd || 0,
                theme_color: params.theme_color || '#10b981',
                valence: 0.5,
                is_loading: false,
                last_pulse: new Date().toISOString(),
                category: 'user',
                ...params
            } as Agent;

            const success = await add_agent(new_agent);
            if (success) {
                event_bus.emit_log({ text: i18n.t('ops.event_agent_init', { name: new_agent.name }), severity: 'success', source: 'System' });
            }
        } catch (error) {
            console.error('Failed to create agent:', error);
            event_bus.emit_log({ text: i18n.t('ops.event_agent_fail'), severity: 'error', source: 'System' });
        }
    };

    const handle_role_change = (agent_id: string, new_role: string) => {
        const roles = use_role_store.getState().roles;
        const new_actions = roles[new_role] || { skills: [], workflows: [] };
        handle_agent_update(agent_id, {
            role: new_role,
            skills: new_actions.skills,
            workflows: new_actions.workflows
        });
    };

    const handle_skill_trigger = async (agent_id: string, skill: string, slot: 1 | 2 | 3 = 1) => {
        const agent = agents_list.find(a => a.id === agent_id);
        if (!agent) return;

        update_agent(agent_id, {
            status: 'active' as const,
            current_task: i18n.t('ops.event_executing', { skill }),
            active_model_slot: slot
        });

        let model_id = agent.model;
        let provider = agent.model_config?.provider;

        if (slot === 2) {
            model_id = agent.model2 || model_id;
            provider = agent.model_config2?.provider || provider;
        } else if (slot === 3) {
            model_id = agent.model3 || model_id;
            provider = agent.model_config3?.provider || provider;
        }

        try {
            const agent_cluster = clusters.find(c => c.collaborators.includes(agent_id));
            await tadpole_os_service.send_command(agent_id, skill, model_id || 'gemini-1.5-flash', provider || 'google', agent_cluster?.id, agent.department, agent_cluster?.budget_usd);
        } catch (e) {
            console.error("❌ [Ops_Dashboard] Failed to trigger skill:", e);
            event_bus.emit_log({
                text: i18n.t('ops.event_trigger_fail', { skill, name: agent.name, error: String(e) }),
                severity: 'error',
                source: 'System'
            });
        }
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

    const handle_deploy = useCallback(async (node_id: string, node_name: string) => {
        set_deploying_target(node_id);
        const target_number = node_id === 'bunker-1' ? 1 : node_id === 'bunker-2' ? 2 : 1;

        event_bus.emit_log({ text: i18n.t('ops.event_deploy_start', { name: node_name }), severity: 'info', source: 'System' });
        try {
            const data = await tadpole_os_service.deploy_engine(target_number);
            event_bus.emit_log({ text: i18n.t('ops.event_deploy_success', { name: node_name }), severity: 'success', source: 'System' });
            if (data.output) {
                event_bus.emit_log({ text: data.output.slice(-500), severity: 'info', source: 'System' });
            }
        } catch (e: unknown) {
            const error_msg = e instanceof Error ? e.message : String(e);
            event_bus.emit_log({ text: i18n.t('ops.event_deploy_error', { error: error_msg }), severity: 'error', source: 'System' });
        } finally {
            set_deploying_target(null);
        }
    }, []);

    const header_actions = useMemo(() => (
        <Dashboard_Header_Actions
            on_discover={discover_nodes}
            nodes_loading={nodes_loading}
            nodes={nodes}
            on_deploy={handle_deploy}
            deploying_target={deploying_target}
            on_initialize_agent={() => set_config_agent_id('new')}
        />
    ), [discover_nodes, nodes_loading, nodes, deploying_target, handle_deploy]);

    useEffect(() => {
        set_header_actions(header_actions);
    }, [header_actions, set_header_actions]);

    return (
        <Error_Boundary>
            <div className="flex flex-col h-full gap-6">
                <div 
                    className="flex-1 min-h-0"
                    onClick={() => close_dropdowns()}
                >
                    <div className="flex flex-col gap-6 min-h-0 h-full">
                        <Stat_Metrics
                            active_agents={active_agents}
                            agents_count={agents_count}
                            total_cost={total_cost}
                            total_tokens={total_tokens}
                            budget_util={budget_util}
                            recruit_velocity={recruit_velocity}
                        />

                        <div className="h-[400px] w-full">
                            <Swarm_Visualizer />
                        </div>

                        <Agent_Status_Grid
                            agents={agents_list}
                            assigned_agent_ids={assigned_agent_ids}
                            available_roles={available_roles}
                            clusters={clusters}
                            on_skill_trigger={handle_skill_trigger}
                            on_model_change={handle_model_change}
                            on_model_2_change={handle_model_2_change}
                            on_model_3_change={handle_model_3_change}
                            on_role_change={handle_role_change}
                            on_configure_click={(id: string) => set_config_agent_id(id)}
                            handle_agent_update={handle_agent_update}
                            on_toggle_cluster={toggle_cluster_active}
                        />
                    </div>
                </div>

                {config_agent_id && (
                    <Agent_Config_Panel
                        agent={config_agent_id === 'new' ? undefined : (agents_list.find(a => a.id === config_agent_id) as Agent)} 
                        on_close={() => set_config_agent_id(null)}
                        on_update={(id: string, updates: Partial<Agent>) => {
                            if (id === 'new') {
                                // Create new agent logic
                                handle_create_agent(updates);
                            } else {
                                handle_agent_update(id, updates);
                            }
                        }}
                        is_new={config_agent_id === 'new'}
                    />
                )}

                <TerminalComponent agents={agents_list} />
            </div>
        </Error_Boundary>
    );
}

