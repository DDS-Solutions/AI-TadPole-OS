/**
 * @docs ARCHITECTURE:UI-Hooks
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useMissionsManager
 * - **Primary Entrypoints**: `useMissionsManager`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useState, useEffect, useCallback, useMemo } from 'react';
import { use_workspace_store } from '../stores/workspace_store';
import { use_agent_store } from '../stores/agent_store';
import { agent_service } from '../services/agent_service';
import { use_trace_store } from '../stores/trace_store';
import { get_settings } from '../stores/settings_store';
import { agent_api_service } from '../services/agent';
import { event_bus } from '../services/event_bus';
import { get_tadpole_os_socket, type Handoff_Event } from '../services/socket';
import { resolve_agent_model_config } from '../utils/model_utils';
import { i18n } from '../i18n';

export function useMissionsManager() {
    const {
        clusters,
        assign_agent_to_cluster,
        unassign_agent_from_cluster: store_unassign_agent_from_cluster,
        update_cluster_objective,
        set_alpha_node,
        delete_cluster,
        toggle_cluster_active,
        update_cluster_department,
        update_cluster_budget,
        toggle_mission_analysis,
        active_proposals,
        dismiss_proposal,
        apply_proposal,
        create_cluster,
        receive_handoff,
        load_quotas
    } = use_workspace_store();

    const { agents, is_loading: agents_loading } = use_agent_store();
    
    const [selected_cluster_id, set_selected_cluster_id] = useState<string | null>(clusters[0]?.id || null);
    const [is_launching, set_is_launching] = useState(false);

    // Initial state sync: automatically select first cluster if selection is empty and clusters exist
    useEffect(() => {
        if (!selected_cluster_id && clusters.length > 0) {
            queueMicrotask(() => set_selected_cluster_id(clusters[0].id));
        }
    }, [clusters, selected_cluster_id]);

    // Initial fetch, quota & governance sync, and handoff subscription
    useEffect(() => {
        const controller = new AbortController();
        if (agents.length === 0) {
            void agent_service.load_agents_into_store();
        }
        void load_quotas();

        const unsubscribeHandoff = get_tadpole_os_socket().subscribe('handoff', (event: Handoff_Event) => {
            const tgt = event.to_cluster || 'unknown';
            const desc = (event.payload?.description as string) || `Cross-cluster task handoff triggered for agent ${event.agent_id}.`;

            receive_handoff(event.from_cluster || 'unknown', tgt, desc);
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('missions.event_handoff', { tgt }),
                severity: 'info',
                mission_id: tgt
            });
        });

        return () => {
            controller.abort();
            unsubscribeHandoff();
        };
    }, [agents.length, receive_handoff, load_quotas]);

    const active_cluster = useMemo(() => 
        (clusters || []).find(c => c.id === selected_cluster_id) || clusters[0] || null,
    [clusters, selected_cluster_id]);

    const assigned_agent_ids = useMemo(() => 
        new Set((clusters || []).flatMap(c => (c.collaborators || []))),
    [clusters]);

    const available_agents = useMemo(() => 
        agents.filter(a => !assigned_agent_ids.has(a.id)),
    [agents, assigned_agent_ids]);

    const cluster_agent_ids = useMemo(() => {
        if (!active_cluster) return [];
        const ids = [...(active_cluster.collaborators || [])];
        if (active_cluster.alpha_id && !ids.includes(active_cluster.alpha_id)) {
            ids.push(active_cluster.alpha_id);
        }
        return ids;
    }, [active_cluster]);

    const has_halted_agents = useMemo(() => {
        if (!active_cluster) return false;
        const cluster_agents = agents.filter(a => cluster_agent_ids.includes(a.id));
        return cluster_agents.some(a => 
            a.status === 'suspended' || 
            a.status === 'offline' || 
            a.status === 'failed' ||
            (a.status === 'thinking' && !active_cluster.is_active)
        );
    }, [active_cluster, agents, cluster_agent_ids]);

    // Traceability helper for distributed tracing across mission launches and interactive turns
    const create_trace = useCallback(() => {
        const request_id = (typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : `tr-${Date.now()}`;
        const trace_id = request_id.replace(/-/g, '').padEnd(32, '0').slice(0, 32);
        use_trace_store.getState().set_active_trace(trace_id);
        return { request_id, trace_id };
    }, []);

    const handle_run_mission = useCallback(async () => {
        if (!active_cluster || is_launching) return;

        if (!active_cluster.alpha_id) {
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('missions.event_fail_alpha', { name: active_cluster.name }),
                severity: 'error',
                mission_id: active_cluster.id
            });
            return;
        }

        if (!active_cluster.objective) {
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('missions.event_fail_objective', { name: active_cluster.name }),
                severity: 'error',
                mission_id: active_cluster.id
            });
            return;
        }

        const alpha_agent = agents.find(a => a.id === active_cluster.alpha_id);
        if (!alpha_agent) {
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('missions.event_error_alpha_not_found'),
                severity: 'error',
                mission_id: active_cluster.id
            });
            return;
        }

        event_bus.emit_log({
            source: 'System',
            text: i18n.t('missions.event_launching', { objective: active_cluster.objective }),
            severity: 'warning',
            mission_id: active_cluster.id
        });

        const { request_id } = create_trace();
        set_is_launching(true);

        try {
            const settings = get_settings();
            const { model_id, provider } = resolve_agent_model_config(alpha_agent, settings.default_model);
            
            const success = await agent_api_service.send_command(
                alpha_agent.id,
                active_cluster.objective,
                model_id,
                provider,
                active_cluster.id,
                active_cluster.department || 'General',
                active_cluster.budget_usd,
                undefined,
                undefined,
                active_cluster.analysis_enabled,
                request_id
            );
            
            if (success) {
                void agent_service.update_agent(alpha_agent.id, { 
                    status: 'active', 
                    current_task: active_cluster.objective 
                }, true);
                event_bus.emit_log({ 
                    source: 'System', 
                    text: i18n.t('missions.event_dispatched', { name: alpha_agent.name }), 
                    severity: 'success',
                    mission_id: active_cluster.id
                });
                
                if (!active_cluster.is_active) {
                    toggle_cluster_active(active_cluster.id);
                }
            } else {
                throw new Error("Engine rejected the command.");
            }
        } catch (err: unknown) {
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('missions.event_launch_fail', { error: err instanceof Error ? err.message : String(err) }),
                severity: 'error',
                mission_id: active_cluster.id
            });
        } finally {
            set_is_launching(false);
        }
    }, [active_cluster, agents, is_launching, create_trace, toggle_cluster_active]);

    const handle_pause_resume_mission = useCallback(async () => {
        if (!active_cluster) return;

        // Case A: Running normally -> Pause cluster
        if (active_cluster.is_active && !has_halted_agents) {
            toggle_cluster_active(active_cluster.id);
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('missions.event_cluster_paused', { name: active_cluster.name }),
                severity: 'warning',
                mission_id: active_cluster.id
            });
            return;
        }

        // Case B: Paused or Has Halted/Crashed Agents -> Perform Parallel Recovery & Resume
        set_is_launching(true);
        try {
            const recovery_results = await Promise.allSettled(
                cluster_agent_ids.map(async (agent_id) => {
                    const agent = agents.find(a => a.id === agent_id);
                    if (agent && (agent.status === 'suspended' || agent.status === 'offline' || agent.status === 'thinking')) {
                        const success = await agent_service.resume_agent(agent_id);
                        if (success) {
                            void agent_service.update_agent(agent_id, { status: 'idle', current_task: undefined }, true);
                            return true;
                        }
                    }
                    return false;
                })
            );

            const recovered_count = recovery_results.filter(r => r.status === 'fulfilled' && r.value).length;

            if (!active_cluster.is_active) {
                toggle_cluster_active(active_cluster.id);
            }

            if (recovered_count > 0) {
                event_bus.emit_log({
                    source: 'System',
                    text: i18n.t('missions.event_cluster_recovered', { count: recovered_count, name: active_cluster.name }),
                    severity: 'success',
                    mission_id: active_cluster.id
                });
            } else {
                event_bus.emit_log({
                    source: 'System',
                    text: i18n.t('missions.event_cluster_resumed', { name: active_cluster.name }),
                    severity: 'success',
                    mission_id: active_cluster.id
                });
            }
        } catch (err: unknown) {
            event_bus.emit_log({
                source: 'System',
                text: `Failed to resume/recover cluster: ${err instanceof Error ? err.message : String(err)}`,
                severity: 'error',
                mission_id: active_cluster.id
            });
        } finally {
            set_is_launching(false);
        }
    }, [active_cluster, has_halted_agents, cluster_agent_ids, agents, toggle_cluster_active]);

    const handle_cancel_mission = useCallback(async () => {
        if (!active_cluster) return;

        set_is_launching(true);
        try {
            await Promise.allSettled(
                cluster_agent_ids.map(async (agent_id) => {
                    const agent = agents.find(a => a.id === agent_id);
                    if (agent && (agent.status === 'active' || agent.status === 'thinking' || agent.status === 'coding')) {
                        await agent_service.pause_agent(agent_id);
                        void agent_service.update_agent(agent_id, { status: 'idle', current_task: undefined }, true);
                    }
                })
            );

            if (active_cluster.is_active) {
                toggle_cluster_active(active_cluster.id);
            }

            event_bus.emit_log({
                source: 'System',
                text: i18n.t('missions.event_cluster_cancelled', { name: active_cluster.name }),
                severity: 'error',
                mission_id: active_cluster.id
            });
        } catch (err: unknown) {
            event_bus.emit_log({
                source: 'System',
                text: `Failed to cancel cluster mission: ${err instanceof Error ? err.message : String(err)}`,
                severity: 'error',
                mission_id: active_cluster.id
            });
        } finally {
            set_is_launching(false);
        }
    }, [active_cluster, cluster_agent_ids, agents, toggle_cluster_active]);

    const send_socratic_response = useCallback(async (response: string) => {
        if (!active_cluster || is_launching) return;
        const alpha_agent = agents.find(a => a.id === active_cluster.alpha_id);
        if (!alpha_agent) return;

        const { request_id } = create_trace();
        set_is_launching(true);
        try {
            const settings = get_settings();
            const { model_id, provider } = resolve_agent_model_config(alpha_agent, settings.default_model);
            
            const success = await agent_api_service.send_command(
                alpha_agent.id,
                response,
                model_id,
                provider,
                active_cluster.id,
                active_cluster.department || 'General',
                active_cluster.budget_usd,
                undefined,
                undefined,
                active_cluster.analysis_enabled,
                request_id
            );
            if (success) {
                event_bus.emit_log({ 
                    source: 'System', 
                    text: `Response sent to swarm: "${response}"`, 
                    severity: 'success',
                    mission_id: active_cluster.id
                });
            }
        } catch (err: unknown) {
            event_bus.emit_log({
                source: 'System',
                text: `Failed to send response: ${err instanceof Error ? err.message : String(err)}`,
                severity: 'error',
                mission_id: active_cluster.id
            });
        } finally {
            set_is_launching(false);
        }
    }, [active_cluster, agents, is_launching, create_trace]);

    const handle_clone_mission = useCallback(() => {
        if (!active_cluster) return;
        const new_name = `${active_cluster.name} (Copy)`;
        create_cluster({
            name: new_name,
            department: active_cluster.department,
            path: active_cluster.path,
            objective: active_cluster.objective ? `[Copy] ${active_cluster.objective}` : '',
            theme: active_cluster.theme
        });
        event_bus.emit_log({
            source: 'System',
            text: `Cloned mission: "${active_cluster.name}" into a fresh unique ID.`,
            severity: 'info',
            mission_id: active_cluster.id
        });
    }, [active_cluster, create_cluster]);

    const unassign_agent_from_cluster = store_unassign_agent_from_cluster;

    return {
        // State
        clusters, agents, agents_loading, active_cluster, available_agents,
        selected_cluster_id, active_proposals, is_launching, has_halted_agents,
        
        // Actions
        set_selected_cluster_id,
        assign_agent_to_cluster, unassign_agent_from_cluster,
        update_cluster_objective, set_alpha_node,
        delete_cluster, toggle_cluster_active,
        update_cluster_department, update_cluster_budget,
        toggle_mission_analysis,
        dismiss_proposal, apply_proposal, create_cluster,
        handle_run_mission, handle_pause_resume_mission, handle_cancel_mission,
        handle_clone_mission,
        send_socratic_response
    };
}
