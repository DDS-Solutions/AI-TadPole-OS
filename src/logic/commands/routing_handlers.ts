/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / routing_handlers
 * - **Primary Entrypoints**: `handle_agent_routing`, `handle_cluster_routing`, `handle_swarm_broadcast`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { event_bus } from '../../services/event_bus';
import { agent_api_service } from '../../services/agent';
import { resolve_agent_model_config } from '../../utils/model_utils';
import { get_settings } from '../../stores/settings_store';
import { use_sovereign_store } from '../../stores/sovereign_store';
import { use_workspace_store } from '../../stores/workspace_store';
import { use_agent_store } from '../../stores/agent_store';
import type { Command_Context, Command_Result } from './types';
import { find_agent } from './utils';

/**
 * Special handler for @agent targeting.
 * Supports multi-word agent names (e.g., "@Agent of Nine status") via longest-prefix matching
 * before falling back to single-token ranked search.
 */
export async function handle_agent_routing(cmd: string, ctx: Command_Context): Promise<Command_Result> {
    const first_token = cmd.substring(1);
    const all_tokens = [first_token, ...ctx.args];
    const candidate_pool = [
        ...ctx.agents,
        ...(use_agent_store.getState().agents || []),
    ];

    let agent = null;
    let remaining_args = ctx.args;

    // 1. Check longest multi-word exact name/ID prefix match first (when args are present)
    for (let len = all_tokens.length; len >= 2; len--) {
        const prefix = all_tokens.slice(0, len).join(' ').toLowerCase();
        const match = candidate_pool.find(
            a => (a.name && a.name.toLowerCase() === prefix) || (a.id && a.id.toLowerCase() === prefix)
        );
        if (match) {
            agent = match;
            remaining_args = all_tokens.slice(len);
            break;
        }
    }

    // 2. Fallback to single-token ranked lookup
    if (!agent) {
        agent = find_agent(first_token, ctx.agents);
        remaining_args = ctx.args;
    }

    if (!agent) {
        return { should_clear_logs: false, handled: true };
    }

    const message = remaining_args.join(' ').trim();
    if (!message) {
        event_bus.emit_log({
            source: 'System',
            text: `Missing directive for agent @${agent.name}. Usage: @${agent.name} <message>`,
            severity: 'error'
        });
        return { should_clear_logs: false, handled: true };
    }

    const live_agent = use_agent_store.getState().agents.find(a => a.id === agent!.id || a.name === agent!.name);
    if (live_agent) agent = live_agent;

    // Synchronize UI chat state with targeted agent so incoming responses and telemetry are visible
    const sovereign_state = use_sovereign_store.getState();
    sovereign_state.set_selected_agent_id?.(agent.id);
    sovereign_state.set_target_agent?.(agent.name);
    sovereign_state.set_scope?.('agent');

    const settings = get_settings();
    const { model_id, provider } = resolve_agent_model_config(agent, settings.default_model);

    event_bus.emit_log({ source: 'User', text: `→ @${agent.name}: ${message}`, severity: 'info' });

    const target_agent = agent;

    setTimeout(() => {
        const reply = `Neural Link: Routing directive to ${target_agent.name}...`;
        event_bus.emit_log({
            source: 'System',
            text: reply,
            severity: 'info'
        });
        use_sovereign_store.getState().add_message({
            sender_id: 'system',
            sender_name: 'Neural System',
            agent_id: target_agent.id,
            text: reply,
            scope: 'agent'
        });
    }, 100);

    const dispatch_promise = (ctx.execution_mode || ctx.recent_findings)
        ? agent_api_service.send_command({
            agent_id: agent.id,
            message,
            model_id,
            provider,
            safe_mode: !!ctx.is_safe_mode,
            execution_mode: ctx.execution_mode,
            recent_findings: ctx.recent_findings,
        })
        : agent_api_service.send_command(agent.id, message, model_id, provider, undefined, undefined, undefined, undefined, !!ctx.is_safe_mode);

    dispatch_promise
        .catch(err => {
            const err_msg = err instanceof Error ? err.message : String(err);
            const is_engine_stopped = err_msg.toLowerCase().includes('engine dispatch is stopped');
            const text = is_engine_stopped
                ? 'Neural link failed: Engine dispatch is stopped by emergency latch. Type "/resume engine" or open the Oversight Dashboard to resume.'
                : `Neural link failed: ${err_msg}`;

            event_bus.emit_log({
                source: 'System',
                text,
                severity: 'error'
            });
            use_sovereign_store.getState().add_message({
                sender_id: 'system',
                sender_name: 'Neural System',
                agent_id: target_agent.id,
                text,
                scope: 'agent'
            });
        });

    return { should_clear_logs: false, handled: true };
}

/**
 * Special handler for #cluster targeting.
 * Supports multi-word cluster names (e.g., "#Strategic Command sync") via longest-prefix matching.
 */
export async function handle_cluster_routing(cmd: string, ctx: Command_Context): Promise<Command_Result> {
    const first_token = cmd.substring(1);
    const all_tokens = [first_token, ...ctx.args];
    const workspace_store = use_workspace_store.getState();
    const clusters = workspace_store.clusters || [];

    let cluster = undefined;
    let remaining_args = ctx.args;

    for (let len = all_tokens.length; len >= 1; len--) {
        const candidate = all_tokens.slice(0, len).join(' ').toLowerCase();
        const found = clusters.find(
            c => (c.name && c.name.toLowerCase() === candidate) || (c.id && c.id.toLowerCase() === candidate)
        );
        if (found) {
            cluster = found;
            remaining_args = all_tokens.slice(len);
            break;
        }
    }

    const cluster_name = first_token.toLowerCase();

    if (!cluster) {
        event_bus.emit_log({
            source: 'System',
            text: `Cluster "${cluster_name}" not found.`,
            severity: 'error'
        });
        return { should_clear_logs: false, handled: true };
    }

    if (!cluster.alpha_id) {
        event_bus.emit_log({
            source: 'System',
            text: `Cluster "${cluster.name}" lacks a designated Alpha leader node.`,
            severity: 'error'
        });
        return { should_clear_logs: false, handled: true };
    }

    const message = remaining_args.join(' ').trim();
    if (!message) {
        event_bus.emit_log({
            source: 'System',
            text: `Missing directive for cluster #${cluster.name}. Usage: #${cluster.name} <message>`,
            severity: 'error'
        });
        return { should_clear_logs: false, handled: true };
    }

    let alpha_agent = ctx.agents.find(a => a.id === cluster.alpha_id);
    const live_agent = use_agent_store.getState().agents.find(a => a.id === cluster.alpha_id);
    if (live_agent) alpha_agent = live_agent;

    if (!alpha_agent) {
        event_bus.emit_log({
            source: 'System',
            text: `Alpha node for cluster "${cluster.name}" (ID: ${cluster.alpha_id}) is offline or not found in active agents.`,
            severity: 'error'
        });
        return { should_clear_logs: false, handled: true };
    }

    // Synchronize UI chat state with targeted cluster
    const cluster_sovereign_state = use_sovereign_store.getState();
    cluster_sovereign_state.set_target_cluster?.(cluster.name);
    cluster_sovereign_state.set_scope?.('cluster');

    const settings = get_settings();
    const { model_id, provider } = resolve_agent_model_config(alpha_agent, settings.default_model);

    event_bus.emit_log({ source: 'User', text: `→ #${cluster.name}: ${message}`, severity: 'info' });

    setTimeout(() => {
        const reply = `Neural Link: Distributing directive to ${cluster.name}...`;
        event_bus.emit_log({
            source: 'System',
            text: reply,
            severity: 'info'
        });
        use_sovereign_store.getState().add_message({
            sender_id: 'system',
            sender_name: 'Neural System',
            text: reply,
            scope: 'cluster',
            target_node: cluster.name
        });
    }, 100);

    const cluster_dispatch = (ctx.execution_mode || ctx.recent_findings)
        ? agent_api_service.send_command({
            agent_id: alpha_agent.id,
            message,
            model_id,
            provider,
            cluster_id: cluster.id,
            department: cluster.department,
            safe_mode: ctx.is_safe_mode,
            execution_mode: ctx.execution_mode,
            recent_findings: ctx.recent_findings,
        })
        : agent_api_service.send_command(alpha_agent.id, message, model_id, provider, cluster.id, cluster.department, undefined, undefined, ctx.is_safe_mode);

    cluster_dispatch
        .catch(err => {
            const err_msg = err instanceof Error ? err.message : String(err);
            const is_engine_stopped = err_msg.toLowerCase().includes('engine dispatch is stopped');
            const text = is_engine_stopped
                ? 'Cluster link failed: Engine dispatch is stopped by emergency latch. Type "/resume engine" or open the Oversight Dashboard to resume.'
                : `Cluster link failed: ${err_msg}`;

            event_bus.emit_log({
                source: 'System',
                text,
                severity: 'error'
            });
            use_sovereign_store.getState().add_message({
                sender_id: 'system',
                sender_name: 'Neural System',
                text,
                scope: 'cluster',
                target_node: cluster.name
            });
        });

    return { should_clear_logs: false, handled: true };
}

/**
 * Special handler for swarm broadcast (no prefix).
 * Dispatches broadcast directive to the primary swarm Alpha/Lead node in addition to emitting telemetry.
 */
export async function handle_swarm_broadcast(parts: string[], ctx?: Command_Context): Promise<Command_Result> {
    const message = parts.join(' ').trim();

    event_bus.emit_log({ source: 'User', text: `Swarm Broadcast: ${message}`, severity: 'info' });

    const reply = `Broadcasting to swarm: ${message.substring(0, 30)}...`;
    event_bus.emit_log({
        source: 'System',
        text: reply,
        severity: 'info'
    });
    use_sovereign_store.getState().add_message({
        sender_id: 'system',
        sender_name: 'Neural System',
        text: reply,
        scope: 'swarm'
    });

    const live_agents = use_agent_store.getState().agents || [];
    const context_agents = ctx?.agents || [];
    const pool = context_agents.length > 0 ? context_agents : live_agents;

    if (message && pool.length > 0) {
        const active_cluster = (use_workspace_store.getState().clusters || []).find(c => c.is_active && c.alpha_id);
        const lead_agent =
            (active_cluster?.alpha_id ? pool.find(a => a.id === active_cluster.alpha_id) : undefined) ||
            pool.find(a => a.role?.toLowerCase().includes('ceo') || a.name?.toLowerCase() === 'ceo' || a.id === '1') ||
            pool[0];

        if (lead_agent) {
            const settings = get_settings();
            const { model_id, provider } = resolve_agent_model_config(lead_agent, settings.default_model);
            const swarm_dispatch = (ctx?.execution_mode || ctx?.recent_findings)
                ? agent_api_service.send_command({
                    agent_id: lead_agent.id,
                    message,
                    model_id,
                    provider,
                    cluster_id: active_cluster?.id,
                    department: active_cluster?.department,
                    safe_mode: !!ctx?.is_safe_mode,
                    execution_mode: ctx?.execution_mode,
                    recent_findings: ctx?.recent_findings,
                })
                : agent_api_service.send_command(
                    lead_agent.id,
                    message,
                    model_id,
                    provider,
                    active_cluster?.id,
                    active_cluster?.department,
                    undefined,
                    undefined,
                    !!ctx?.is_safe_mode
                );

            swarm_dispatch.catch(err => {
                const err_msg = err instanceof Error ? err.message : String(err);
                const is_engine_stopped = err_msg.toLowerCase().includes('engine dispatch is stopped');
                const text = is_engine_stopped
                    ? 'Swarm broadcast failed: Engine dispatch is stopped by emergency latch. Type "/resume engine" or open the Oversight Dashboard to resume.'
                    : `Swarm broadcast failed: ${err_msg}`;

                event_bus.emit_log({
                    source: 'System',
                    text,
                    severity: 'error'
                });
                use_sovereign_store.getState().add_message({
                    sender_id: 'system',
                    sender_name: 'Neural System',
                    text,
                    scope: 'swarm'
                });
            });
        }
    }

    return { should_clear_logs: false, handled: true };
}
