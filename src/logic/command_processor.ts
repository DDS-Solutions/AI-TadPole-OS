/**
 * @docs ARCHITECTURE:Logic
 * @docs OPERATIONS_MANUAL:Commands
 * 
 * ### AI Assist Note
 * **NLP Orchestrator**: Manages the translation of user intent (slash commands, @mentions, #clusters) into actionable system directives. 
 * Implements lexical analysis with quote preservation and multi-tier agent resolution.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Lexical parsing errors (unclosed quotes), agent resolution ambiguity (multiple matches), or API timeout during `/deploy` or `/send`.
 * - **Telemetry Link**: Search for `[CommandProcessor]` in `event_bus` logs or `process_command` trace spans.
 */

import { event_bus } from '../services/event_bus';
import type { Agent } from '../types';

import type { Command_Context } from './commands/types';
import { command_registry } from './commands/registry';
import { register_system_commands } from './commands/system_handlers';
import { register_agent_commands } from './commands/agent_handlers';
import { register_swarm_commands } from './commands/swarm_handlers';
import { handle_agent_routing, handle_cluster_routing, handle_swarm_broadcast } from './commands/routing_handlers';

// Initialize the registry with domain-specific handlers
register_system_commands();
register_agent_commands();
register_swarm_commands();

/**
 * process_command
 * Processes a single slash-command string from the user.
 * Supports standard slash commands (/help, /clear), agent-specific targeting (@agent), 
 * and cluster-specific targeting (#cluster).
 * 
 * REFACTORED: Now uses a Strategy Pattern / Plug-in Architecture.
 */
export async function process_command(
    command_text: string,
    agents: Agent[],
    is_safe_mode: boolean = false,
    active_scope: 'agent' | 'cluster' | 'swarm' = 'swarm',
    target_node?: string
): Promise<{ should_clear_logs: boolean }> {
    const telemetry_source = '[CommandProcessor]';
    
    // 1. Lexical Analysis: Split by spaces but preserve quoted strings (e.g. "quoted msg")
    const parts: string[] = [];
    const regex = /[^\s"']+|"([^"]*)"|'([^']*)'/g;
    let match;
    while ((match = regex.exec(command_text)) !== null) {
        parts.push(match[1] || match[2] || match[0]);
    }

    if (parts.length === 0) return { should_clear_logs: false };
    const cmd = parts[0].toLowerCase();
    const args = parts.slice(1);

    const ctx: Command_Context = {
        parts,
        args,
        agents,
        is_safe_mode,
        active_scope,
        target_node,
        telemetry_source
    };

    // 2. Dispatch to Registry (Slash Commands)
    if (cmd.startsWith('/')) {
        const result = await command_registry.execute(cmd, ctx);
        if (result.handled) return { should_clear_logs: result.should_clear_logs };
        
        // Fallback for unknown slash commands
        event_bus.emit_log({
            source: 'System',
            text: `Unknown command: ${cmd}. Type /help for available commands.`,
            severity: 'error'
        });
        return { should_clear_logs: false };
    }

    // 3. Handle Targeted Routing (@agent, #cluster)
    if (cmd.startsWith('@')) {
        const result = await handle_agent_routing(cmd, ctx);
        return { should_clear_logs: result.should_clear_logs };
    }

    if (cmd.startsWith('#')) {
        const result = await handle_cluster_routing(cmd, ctx);
        return { should_clear_logs: result.should_clear_logs };
    }

    // 4. Auto-Routing based on active scope (if no prefix is used)
    if (!cmd.startsWith('/') && !cmd.startsWith('@') && !cmd.startsWith('#') && active_scope !== 'swarm' && target_node) {
        console.debug(`${telemetry_source} Auto-routing intent to ${active_scope}:${target_node}`);
        const prefix = active_scope === 'cluster' ? '#' : '@';
        return process_command(`${prefix}${target_node} ${command_text}`, agents, is_safe_mode, active_scope, target_node);
    }

    // 5. Default Swarm Broadcast
    const result = await handle_swarm_broadcast(parts);
    return { should_clear_logs: result.should_clear_logs };
}

// Metadata: [command_processor]
