/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / system_handlers
 * - **Primary Entrypoints**: `register_system_commands`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { event_bus } from '../../services/event_bus';
import { system_api_service } from '../../services/system_api_service';
import { command_registry } from './registry';
import type { Command_Context } from './types';

export function register_system_commands() {
    // ────────────── HELP ──────────────
    command_registry.register({
        command: '/help',
        description: 'Show available commands and routing syntax',
        handler: async () => {
            const definitions = command_registry.get_definitions();
            const help_text = [
                '📋 Available Slash Commands:',
                ...definitions.map(d => `  ${d.command.padEnd(18)} — ${d.description}`),
                '',
                '🎯 Targeting Syntax:',
                '  @<agent> <msg>     — Direct neural message to an agent (e.g. @CEO sync status)',
                '  #<cluster> <msg>   — Dispatch directive to cluster Alpha node (e.g. #Engineering deploy)',
                '  <message>          — Broadcast intent across current active scope'
            ].join('\n');

            event_bus.emit_log({
                source: 'System',
                text: help_text,
                severity: 'info'
            });
            return { should_clear_logs: false, handled: true };
        }
    });

    // ────────────── CLEAR ──────────────
    command_registry.register({
        command: '/clear',
        description: 'Clear terminal history',
        handler: async () => {
            event_bus.clear_history();
            return { should_clear_logs: true, handled: true };
        }
    });

    // ────────────── STATUS ──────────────
    command_registry.register({
        command: '/status',
        description: 'Show agent swarm summary',
        handler: async (ctx: Command_Context) => {
            const active = ctx.agents.filter(a => a.status === 'active' || a.status === 'thinking' || a.status === 'coding' || a.status === 'speaking').length;
            const idle = ctx.agents.filter(a => a.status === 'idle').length;
            const offline = ctx.agents.filter(a => a.status === 'offline').length;
            const degraded = ctx.agents.filter(a => a.status === 'suspended' || a.status === 'failed' || a.status === 'throttled').length;
            const total_tokens = ctx.agents.reduce((sum, a) => sum + (a.tokens_used || 0), 0);

            const degraded_text = degraded > 0 ? ` · ${degraded} degraded/suspended` : '';

            event_bus.emit_log({
                source: 'System',
                text: `Swarm Status: ${active} active · ${idle} idle · ${offline} offline${degraded_text} | Total tokens: ${(total_tokens / 1000).toFixed(1)}k`,
                severity: 'success'
            });
            return { should_clear_logs: false, handled: true };
        }
    });

    // ────────────── DEPLOY ──────────────
    command_registry.register({
        command: '/deploy',
        description: 'Trigger production deployment simulation',
        timeout_ms: 60000,
        handler: async (ctx: Command_Context) => {
            if (ctx.args[0]?.toLowerCase() !== 'confirm') {
                event_bus.emit_log({
                    source: 'System',
                    text: '⚠️ This will trigger a production deployment to Swarm Bunker. Type "/deploy confirm" to proceed.',
                    severity: 'warning'
                });
                return { should_clear_logs: false, handled: true };
            }

            event_bus.emit_log({
                source: 'System',
                text: '🚀 Triggering deployment to Swarm Bunker via /engine/deploy...',
                severity: 'warning'
            });

            try {
                const data = await system_api_service.engine.deploy_engine();
                event_bus.emit_log({
                    source: 'System',
                    text: `✅ Deployment successful. Output: ${(data.output || '').slice(-300)}`,
                    severity: 'success'
                });
            } catch (e: unknown) {
                const error_msg = e instanceof Error ? e.message : String(e);
                event_bus.emit_log({
                    source: 'System',
                    text: `❌ Deployment error: ${error_msg}`,
                    severity: 'error'
                });
            }
            return { should_clear_logs: false, handled: true };
        }
    });
}
