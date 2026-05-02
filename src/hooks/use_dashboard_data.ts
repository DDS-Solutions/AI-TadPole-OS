/**
 * @docs ARCHITECTURE:Logic
 * 
 * ### AI Assist Note
 * **UI State Aggregator**: Central hook for orchestrating dashboard telemetry, agent registries, and node lifecycle. 
 * Synchronizes local state with `agent_store`, `node_store`, and `event_bus` to provide a unified data flow for the main dashboard views.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Stale recruitment velocity (if `now` isn't stable), log buffer overflow (exceeding 100 entries), or telemetry sync lag if `init_telemetry` fails.
 * - **Telemetry Link**: Search for `[useDashboardData]` in component logs or check `Total Cost` / `Total Tokens` in UI audits.
 */

import { useState, useEffect, useMemo, useRef } from 'react';
import type { Agent } from '../types';
import { use_agent_store } from '../stores/agent_store';
import { use_node_store } from '../stores/node_store';
import { useEngineStatus } from '../hooks/use_engine_status';
import { event_bus, type log_entry } from '../services/event_bus';
import { use_workspace_store } from '../stores/workspace_store';
import { use_role_store, type Role_State } from '../stores/role_store';

export function useDashboardData() {
    const { is_online } = useEngineStatus();
    const { agents: agents_list, fetch_agents, update_agent, add_agent, init_telemetry } = use_agent_store();
    const { nodes, fetch_nodes, discover_nodes, is_loading: nodes_loading } = use_node_store();
    const [logs, set_logs] = useState<log_entry[]>(() => event_bus.get_history());
    const logs_end_ref = useRef<HTMLDivElement>(null);

    const agents_count = (agents_list || []).length;

    useEffect(() => {
        const controller = new AbortController();
        const { signal } = controller;

        fetch_agents({ signal });
        fetch_nodes({ signal });
        const unsubscribe_telemetry = init_telemetry();

        const unsubscribe_logs = event_bus.subscribe_logs((entry) => {
            set_logs(prev => [...prev, entry].slice(-100));
        });

        return () => {
            controller.abort();
            unsubscribe_logs();
            unsubscribe_telemetry();
        };
    }, [fetch_agents, fetch_nodes, init_telemetry]);

    // Auto-scroll to bottom of logs
    useEffect(() => {
        if (logs_end_ref.current) {
            logs_end_ref.current.scrollIntoView({ behavior: 'smooth' });
        }
    }, [logs]);

    const { clusters, toggle_cluster_active } = use_workspace_store();
    const assigned_agent_ids = useMemo(() => new Set((clusters || []).flatMap(c => (c.collaborators || [])).map(String)), [clusters]);

    const roles = use_role_store((s: Role_State) => s.roles);
    const available_roles = useMemo(() => Object.keys(roles).sort(), [roles]);

    const active_agents = useMemo(() =>
        (agents_list || []).filter((a: Agent) => (a.status === 'active' || a.status === 'speaking') && assigned_agent_ids.has(a.id)).length,
        [agents_list, assigned_agent_ids]);
 
    const total_cost = useMemo(() => (agents_list || []).reduce((acc: number, curr: Agent) => acc + (curr.cost_usd || 0), 0), [agents_list]);
    const total_budget = useMemo(() => (agents_list || []).reduce((acc: number, curr: Agent) => acc + (curr.budget_usd || 0), 0), [agents_list]);
    const budget_util = total_budget > 0 ? (total_cost / total_budget) * 100 : 0;
 
    const total_tokens = useMemo(() => (agents_list || []).reduce((acc: number, curr: Agent) => acc + (curr.tokens_used || 0), 0), [agents_list]);
    const total_input_tokens = useMemo(() => (agents_list || []).reduce((acc: number, curr: Agent) => acc + (curr.input_tokens || 0), 0), [agents_list]);
    const total_output_tokens = useMemo(() => (agents_list || []).reduce((acc: number, curr: Agent) => acc + (curr.output_tokens || 0), 0), [agents_list]);
 
    // Calculate Recruitment Velocity (Agents created in the last 24 hours)
    // We use a stable reference for "now" to satisfy React purity rules for useMemo.
    const [now] = useState(() => Date.now());
    const recruit_velocity = useMemo(() => {
        const twenty_four_hours_ago = now - (24 * 60 * 60 * 1000);
        return (agents_list || []).filter((a: Agent) => {
            const created_time = a.created_at ? new Date(a.created_at).getTime() : 0;
            return created_time > twenty_four_hours_ago;
        }).length;
    }, [agents_list, now]);


    const nodes_refined = useMemo(() => (nodes || []).map(n => ({
        ...n,
        running_agents: n.running_agents || []
    })), [nodes]);

    return {
        is_online,
        agents_list,
        agents_count,
        active_agents,
        total_cost,
        total_tokens,
        total_input_tokens,
        total_output_tokens,
        budget_util,
        recruit_velocity,
        nodes: nodes_refined,
        nodes_loading,
        logs,
        logs_end_ref,
        assigned_agent_ids,
        available_roles,
        clusters,
        toggle_cluster_active,
        fetch_agents,
        update_agent,
        add_agent,
        fetch_nodes,
        discover_nodes
    };
}

// Metadata: [use_dashboard_data]

// Metadata: [use_dashboard_data]
