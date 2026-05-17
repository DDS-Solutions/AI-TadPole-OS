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
import { agent_service } from '../services/agent_service';
import { use_node_store } from '../stores/node_store';
import { useEngineStatus } from '../hooks/use_engine_status';
import { event_bus, type log_entry } from '../services/event_bus';
import { use_workspace_store } from '../stores/workspace_store';
import { use_role_store, type Role_State } from '../stores/role_store';

export function useDashboardData() {
    const { is_online } = useEngineStatus();
    const agents_list = use_agent_store(s => s.agents);

    const nodes = use_node_store(s => s.nodes);
    const fetch_nodes = use_node_store(s => s.fetch_nodes);
    const discover_nodes = use_node_store(s => s.discover_nodes);
    const nodes_loading = use_node_store(s => s.is_loading);
    const [logs, set_logs] = useState<log_entry[]>(() => event_bus.get_history());
    const logs_end_ref = useRef<HTMLDivElement>(null);

    const agents_count = Array.isArray(agents_list) ? agents_list.length : 0;

    useEffect(() => {
        const controller = new AbortController();
        const { signal } = controller;

        void agent_service.load_agents_into_store();
        fetch_nodes({ signal });

        const unsubscribe_logs = event_bus.subscribe_logs((entry) => {
            set_logs(prev => [...prev, entry].slice(-100));
        });

        return () => {
            controller.abort();
            unsubscribe_logs();
        };
    }, [fetch_nodes]);

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
        (Array.isArray(agents_list) ? agents_list : []).filter((a: Agent) => 
            ['active', 'thinking', 'coding', 'speaking'].includes(a.status) && assigned_agent_ids.has(a.id)
        ).length,
        [agents_list, assigned_agent_ids]);

    const online_count = useMemo(() => 
        (Array.isArray(agents_list) ? agents_list : []).filter((a: Agent) => a.status !== 'offline').length,
        [agents_list]);
 
    const total_cost = useMemo(() => (Array.isArray(agents_list) ? agents_list : []).reduce((acc: number, curr: Agent) => acc + (curr.cost_usd || 0), 0), [agents_list]);
    const total_budget = useMemo(() => (Array.isArray(agents_list) ? agents_list : []).reduce((acc: number, curr: Agent) => acc + (curr.budget_usd || 0), 0), [agents_list]);
    const budget_util = total_budget > 0 ? (total_cost / total_budget) * 100 : 0;
 
    const total_tokens = useMemo(() => (Array.isArray(agents_list) ? agents_list : []).reduce((acc: number, curr: Agent) => acc + (curr.tokens_used || 0), 0), [agents_list]);
    const total_input_tokens = useMemo(() => (Array.isArray(agents_list) ? agents_list : []).reduce((acc: number, curr: Agent) => acc + (curr.input_tokens || 0), 0), [agents_list]);
    const total_output_tokens = useMemo(() => (Array.isArray(agents_list) ? agents_list : []).reduce((acc: number, curr: Agent) => acc + (curr.output_tokens || 0), 0), [agents_list]);
 
    // Calculate Recruitment Velocity (Agents created in the last 24 hours)
    // We use a ticking state for "now" to ensure the rolling 24-hour window 
    // remains accurate during long browser sessions.
    const [now, set_now] = useState(() => Date.now());

    useEffect(() => {
        const interval = setInterval(() => {
            set_now(Date.now());
        }, 60000); // Synchronize window every 60 seconds
        return () => clearInterval(interval);
    }, []);

    const recruit_velocity = useMemo(() => {
        const twenty_four_hours_ago = now - (24 * 60 * 60 * 1000);
        return (Array.isArray(agents_list) ? agents_list : []).filter((a: Agent) => {
            if (!a.created_at) return false;
            const created_time = new Date(a.created_at).getTime();
            return !isNaN(created_time) && created_time > twenty_four_hours_ago;
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
        online_count,
        total_cost,
        total_tokens,
        total_input_tokens,
        total_output_tokens,
        total_budget,
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
        update_agent: (id: string, updates: Partial<Agent>) => agent_service.update_agent(id, updates),
        add_agent: (agent: Agent) => agent_service.broadcast_update(agent.id, agent), // For local adds
        fetch_nodes,
        discover_nodes
    };
}

// Metadata: [use_dashboard_data]

// Metadata: [use_dashboard_data]
