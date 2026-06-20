/**
 * @docs ARCHITECTURE:Logic
 * @docs OPERATIONS_MANUAL:Commands
 * 
 * ### AI Assist Note
 * **NLP Orchestrator**: Manages the translation of user intent (slash commands, @mentions, #clusters) into actionable system directives. 
 * Implements lexical analysis with quote preservation and multi-tier agent resolution.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Lexical parsing errors (unclosed quotes), agent resolution ambiguity (multiple matches), or API timeout during /deploy or /send.
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

let is_initialized = false;

/**
 * initialize_command_processor
 * Registers all domain-specific handlers with the global registry.
 * Safe to call multiple times (idempotent).
 */
export function initialize_command_processor(): void {
    if (is_initialized) return;
    register_system_commands();
    register_agent_commands();
    register_swarm_commands();
    is_initialized = true;
}

/**
 * reset_command_processor
 * Resets the initialization flag and clears the command registry map.
 * Typically used to ensure test isolation.
 */
export function reset_command_processor(): void {
    is_initialized = false;
    command_registry.clear();
}

/**
 * with_timeout
 * Wraps a promise in a timeout wrapper to prevent hanging operations.
 */
function with_timeout<T>(promise: Promise<T>, timeout_ms: number, error_message: string): Promise<T> {
    let timeout_id: ReturnType<typeof setTimeout> | undefined;
    const timeout_promise = new Promise<never>((_, reject) => {
        timeout_id = setTimeout(() => {
            reject(new Error(error_message));
        }, timeout_ms);
    });
    return Promise.race([promise, timeout_promise]).finally(() => {
        if (timeout_id) clearTimeout(timeout_id);
    });
}

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
    const MAX_CMD_LENGTH = 4096;
    const DISPATCH_TIMEOUT_MS = 5000;

    // Guard: null/undefined checks
    if (command_text === null || command_text === undefined) {
        event_bus.emit_log({
            source: 'System',
            text: 'System Error: Received null or undefined command text.',
            severity: 'error'
        });
        return { should_clear_logs: false };
    }

    // Sanitize: strip C0 and C1 control characters
    // eslint-disable-next-line no-control-regex
    const sanitized_text = command_text.replace(/[\u0000-\u001F\u007F-\u009F]/g, '').trim();

    // Guard: Length boundary check
    if (sanitized_text.length > MAX_CMD_LENGTH) {
        event_bus.emit_log({
            source: 'System',
            text: `System Error: Command text exceeds maximum limit of ${MAX_CMD_LENGTH} characters.`,
            severity: 'error'
        });
        return { should_clear_logs: false };
    }

    // 1. Lexical Analysis: Split by spaces but preserve quoted strings (e.g. "quoted msg")
    const parts: string[] = [];
    const regex = /[^\s"']+|"([^"]*)"|'([^']*)'/g;
    let match;
    while ((match = regex.exec(sanitized_text)) !== null) {
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

    try {
        // 2. Dispatch to Registry (Slash Commands)
        if (cmd.startsWith('/')) {
            const result = await with_timeout(
                command_registry.execute(cmd, ctx),
                DISPATCH_TIMEOUT_MS,
                'Command execution timed out'
            );
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
            const result = await with_timeout(
                handle_agent_routing(cmd, ctx),
                DISPATCH_TIMEOUT_MS,
                'Command execution timed out'
            );
            return { should_clear_logs: result.should_clear_logs };
        }

        if (cmd.startsWith('#')) {
            const result = await with_timeout(
                handle_cluster_routing(cmd, ctx),
                DISPATCH_TIMEOUT_MS,
                'Command execution timed out'
            );
            return { should_clear_logs: result.should_clear_logs };
        }

        // 4. Auto-Routing based on active scope (if no prefix is used)
        if (!cmd.startsWith('/') && !cmd.startsWith('@') && !cmd.startsWith('#') && active_scope !== 'swarm' && target_node) {
            console.debug(`${telemetry_source} Auto-routing intent to ${active_scope}:${target_node}`);
            const prefix = active_scope === 'cluster' ? '#' : '@';
            return process_command(`${prefix}${target_node} ${sanitized_text}`, agents, is_safe_mode, active_scope, target_node);
        }

        // 5. Default Swarm Broadcast
        const result = await with_timeout(
            handle_swarm_broadcast(parts),
            DISPATCH_TIMEOUT_MS,
            'Command execution timed out'
        );
        return { should_clear_logs: result.should_clear_logs };
    } catch (error) {
        const error_msg = error instanceof Error ? error.message : 'Unknown command execution failure';
        event_bus.emit_log({
            source: 'System',
            text: `Command execution failed: ${error_msg}`,
            severity: 'error'
        });
        return { should_clear_logs: false };
    }
}

// Metadata: [command_processor]
