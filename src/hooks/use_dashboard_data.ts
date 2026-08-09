/**
 * @docs ARCHITECTURE:Logic
 * 
 * ### AI Assist Note
 * **UI State Aggregator**: Central hook for orchestrating dashboard telemetry, agent registries, and node lifecycle. 
 * Synchronizes local state with `agent_store`, `node_store`, and `event_bus` to provide a unified data flow for the main dashboard views.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Stale recruitment velocity (if `now` isn't stable), log buffer overflow (exceeding 100 entries), or telemetry sync lag if init_telemetry fails.
 * - **Telemetry Link**: Search for `[useDashboardData]` in component logs or check `Total Cost` / `Total Tokens` in UI audits.
 */

import { useEffect, useMemo, useCallback } from 'react';
import type { Agent } from '../types';
import { use_agent_store } from '../stores/agent_store';
import { agent_service } from '../services/agent_service';
import { use_node_store } from '../stores/node_store';
import { useEngineStatus } from '../hooks/use_engine_status';
import { useLogs } from '../hooks/use_logs';
import { use_workspace_store } from '../stores/workspace_store';
import { use_role_store, type Role_State } from '../stores/role_store';
import { useAgentMetrics } from './use_agent_metrics';
import { useRecruitmentVelocity } from './use_recruitment_velocity';

export function useDashboardData() {
    const { is_online } = useEngineStatus();
    const agents_list = use_agent_store(s => s.agents);

    const { logs, logs_end_ref } = useLogs();

    const nodes = use_node_store(s => s.nodes);
    const fetch_nodes = use_node_store(s => s.fetch_nodes);
    const discover_nodes = use_node_store(s => s.discover_nodes);
    const nodes_loading = use_node_store(s => s.is_loading);

    const agents_count = Array.isArray(agents_list) ? agents_list.length : 0;

    useEffect(() => {
        const controller = new AbortController();
        const { signal } = controller;

        agent_service.load_agents_into_store()?.catch?.(err => {
            console.error('[useDashboardData] Failed to load agent registry into store:', err);
        });

        fetch_nodes({ signal })?.catch?.(err => {
            if (err?.name !== 'AbortError') {
                console.error('[useDashboardData] Failed to fetch node topology:', err);
            }
        });

        return () => {
            controller.abort();
        };
    }, [fetch_nodes]);

    const clusters = use_workspace_store(s => s.clusters);
    const toggle_cluster_active = use_workspace_store(s => s.toggle_cluster_active);
    const assigned_agent_ids = useMemo(() => new Set((clusters || []).flatMap(c => (c.collaborators || [])).map(String)), [clusters]);

    const roles = use_role_store((s: Role_State) => s.roles);
    const roles_key_fingerprint = useMemo(() => Object.keys(roles).sort().join(','), [roles]);
    const available_roles = useMemo(() => roles_key_fingerprint ? roles_key_fingerprint.split(',') : [], [roles_key_fingerprint]);

    const metrics = useAgentMetrics({ agents_list, assigned_agent_ids });
    const recruit_velocity = useRecruitmentVelocity(agents_list);

    const nodes_refined = useMemo(() => {
        if (!nodes) return [];
        if (nodes.every(n => Array.isArray(n.running_agents))) return nodes;
        return nodes.map(n => n.running_agents ? n : { ...n, running_agents: [] });
    }, [nodes]);

    const update_agent = useCallback((id: string, updates: Partial<Agent>) => {
        return agent_service.update_agent(id, updates);
    }, []);

    const add_agent = useCallback((agent: Agent) => {
        use_agent_store.setState(state => {
            if (state.agents.some(a => a.id === agent.id)) return state;
            return { agents: [...state.agents, agent] };
        });
        return true;
    }, []);

    const delete_agent = useCallback((id: string) => {
        return agent_service.delete_agent(id);
    }, []);

    return {
        is_online,
        agents_list,
        agents_count,
        ...metrics,
        recruit_velocity,
        nodes: nodes_refined,
        nodes_loading,
        logs,
        logs_end_ref,
        assigned_agent_ids,
        available_roles,
        clusters,
        toggle_cluster_active,
        update_agent,
        add_agent,
        delete_agent,
        fetch_nodes,
        discover_nodes
    };
}

// Metadata: [use_dashboard_data]
