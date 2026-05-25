/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[useMissionsManager]` in observability traces.
 */

import { useState, useEffect, useCallback, useMemo } from 'react';
import { use_workspace_store } from '../stores/workspace_store';
import { use_agent_store } from '../stores/agent_store';
import { agent_service } from '../services/agent_service';
import { use_trace_store } from '../stores/trace_store';
import { get_settings } from '../stores/settings_store';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { event_bus } from '../services/event_bus';
import { tadpole_os_socket, type Handoff_Event } from '../services/socket';
import { resolve_agent_model_config } from '../utils/model_utils';
import { i18n } from '../i18n';
import type { Agent } from '../types';

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
        receive_handoff
    } = use_workspace_store();

    const { agents, is_loading: agents_loading } = use_agent_store();
    
    const [selected_cluster_id, set_selected_cluster_id] = useState<string | null>(clusters[0]?.id || null);
    const [is_launching, set_is_launching] = useState(false);

    // Initial fetch and handoff subscription
    useEffect(() => {
        const controller = new AbortController();
        if (agents.length === 0) {
            void agent_service.load_agents_into_store();
        }

        const unsubscribeHandoff = tadpole_os_socket.subscribe_handoff((event: Handoff_Event) => {
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
    }, [agents.length, receive_handoff]);

    const active_cluster = useMemo(() => 
        (clusters || []).find(c => c.id === selected_cluster_id),
    [clusters, selected_cluster_id]);

    const assigned_agent_ids = useMemo(() => 
        new Set((clusters || []).flatMap(c => (c.collaborators || []))),
    [clusters]);

    const available_agents = useMemo(() => 
        agents.filter(a => !assigned_agent_ids.has(a.id)),
    [agents, assigned_agent_ids]);

    const handle_run_mission = useCallback(async () => {
        if (!active_cluster) return;

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

        const request_id = (typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : `tr-${Date.now()}`;
        const trace_id = request_id.replace(/-/g, '').padEnd(32, '0').slice(0, 32);
        use_trace_store.getState().set_active_trace(trace_id);
        set_is_launching(true);

        try {
            const settings = get_settings();
            const { model_id, provider } = resolve_agent_model_config(alpha_agent as Agent, settings.default_model);
            
            const success = await tadpole_os_service.send_command(
                alpha_agent.id,
                active_cluster.objective,
                model_id,
                provider,
                active_cluster.id,
                active_cluster.department as string,
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
    }, [active_cluster, agents, toggle_cluster_active]);

    const unassign_agent_from_cluster = useCallback((agent_id: string, cluster_id: string) => {
        store_unassign_agent_from_cluster(agent_id, cluster_id);
    }, [store_unassign_agent_from_cluster]);

    return {
        // State
        clusters, agents, agents_loading, active_cluster, available_agents,
        selected_cluster_id, active_proposals, is_launching,
        
        // Actions
        set_selected_cluster_id,
        assign_agent_to_cluster, unassign_agent_from_cluster,
        update_cluster_objective, set_alpha_node,
        delete_cluster, toggle_cluster_active,
        update_cluster_department, update_cluster_budget,
        toggle_mission_analysis,
        dismiss_proposal, apply_proposal, create_cluster,
        handle_run_mission
    };
}

// Metadata: [useMissionsManager]
