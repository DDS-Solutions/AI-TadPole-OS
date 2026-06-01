/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:Logic:Commands**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[routing_handlers]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:Logic:Commands
 */

import { event_bus } from '../../services/event_bus';
import { agent_api_service } from '../../services/agent_api_service';
import { resolve_agent_model_config } from '../../utils/model_utils';
import { get_settings } from '../../stores/settings_store';
import { use_sovereign_store } from '../../stores/sovereign_store';
import { use_workspace_store } from '../../stores/workspace_store';
import type { Command_Context, Command_Result } from './types';
import { find_agent } from './utils';

/**
 * Special handler for @agent targeting.
 */
export async function handle_agent_routing(cmd: string, ctx: Command_Context): Promise<Command_Result> {
    const target_name = cmd.substring(1);
    const agent = find_agent(target_name, ctx.agents);
    if (agent) {
        const message = ctx.args.join(' ');
        const settings = get_settings();
        const { model_id, provider } = resolve_agent_model_config(agent, settings.default_model);

        event_bus.emit_log({ source: 'User', text: `→ @${agent.name}: ${message}`, severity: 'info' });

        setTimeout(() => {
            const reply = `Neural Link: Routing directive to ${agent.name}...`;
            event_bus.emit_log({
                source: 'System',
                text: reply,
                severity: 'info'
            });
            use_sovereign_store.getState().add_message({
                sender_id: 'system',
                sender_name: 'Neural System',
                agent_id: agent.id,
                text: reply,
                scope: 'agent'
            });
        }, 100);

        agent_api_service.send_command(agent.id, message, model_id, provider, undefined, undefined, undefined, undefined, !!ctx.is_safe_mode)
            .catch(err => {
                event_bus.emit_log({
                    source: 'System',
                    text: `Neural link failed: ${err.message || err}`,
                    severity: 'error'
                });
            });
    }
    return { should_clear_logs: false, handled: true };
}

/**
 * Special handler for #cluster targeting.
 */
export async function handle_cluster_routing(cmd: string, ctx: Command_Context): Promise<Command_Result> {
    const cluster_name = cmd.substring(1).toLowerCase();
    const workspace_store = use_workspace_store.getState();
    const cluster = workspace_store.clusters.find(c => (c.name?.toLowerCase() === cluster_name) || c.id === cluster_name);

    if (cluster && cluster.alpha_id) {
        const alpha_agent = ctx.agents.find(a => a.id === cluster.alpha_id);
        if (alpha_agent) {
            const message = ctx.args.join(' ');
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
                    scope: 'cluster'
                });
            }, 100);

            agent_api_service.send_command(alpha_agent.id, message, model_id, provider, cluster.id, cluster.department, undefined, undefined, ctx.is_safe_mode)
                .catch(err => {
                    event_bus.emit_log({
                        source: 'System',
                        text: `Cluster link failed: ${err.message || err}`,
                        severity: 'error'
                    });
                });
        }
    } else {
        event_bus.emit_log({
            source: 'System',
            text: `Cluster "${cluster_name}" not found or lacks an Alpha node.`,
            severity: 'error'
        });
    }
    return { should_clear_logs: false, handled: true };
}

/**
 * Special handler for swarm broadcast (no prefix).
 */
export async function handle_swarm_broadcast(parts: string[]): Promise<Command_Result> {
    const message = parts.join(' ');

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

    return { should_clear_logs: false, handled: true };
}

// Metadata: [routing_handlers]
