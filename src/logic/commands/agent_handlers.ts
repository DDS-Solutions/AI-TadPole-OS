/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:Logic:Commands**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[agent_handlers]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:Logic:Commands
 */

import { event_bus } from '../../services/event_bus';
import { agent_service } from '../../services/agent_service';
import { agent_api_service } from '../../services/agent_api_service';
import { resolve_agent_model_config } from '../../utils/model_utils';
import { get_settings } from '../../stores/settings_store';
import { use_sovereign_store } from '../../stores/sovereign_store';
import { command_registry } from './registry';
import type { Command_Context } from './types';
import { find_agent } from './utils';

export function register_agent_commands() {
    // ────────────── CONFIG ──────────────
    command_registry.register({
        command: '/config',
        description: 'View agent configuration',
        handler: async (ctx: Command_Context) => {
            const agent = find_agent(ctx.args[0], ctx.agents);
            if (!agent) return { should_clear_logs: false, handled: true };

            event_bus.emit_log({
                source: 'System',
                text: [
                    `⚙️ Config for ${agent.name}:`,
                    `  Model: ${agent.model}`,
                    `  Temperature: ${agent.model_config?.temperature ?? 'default'}`,
                    `  Status: ${agent.status}`,
                    `  Prompt: ${agent.model_config?.systemPrompt ? agent.model_config.systemPrompt.substring(0, 80) + '...' : '(none)'}`,
                ].join('\n'),
                severity: 'info'
            });
            return { should_clear_logs: false, handled: true };
        }
    });

    // ────────────── PAUSE ──────────────
    command_registry.register({
        command: '/pause',
        description: 'Pause an autonomous agent',
        handler: async (ctx: Command_Context) => {
            const agent = find_agent(ctx.args[0], ctx.agents);
            if (!agent) return { should_clear_logs: false, handled: true };

            const success = await agent_service.pause_agent(agent.id);
            event_bus.emit_log({
                source: 'System',
                text: success
                    ? `⏸️ Agent ${agent.name} paused via TadpoleOS.`
                    : `⏸️ Agent ${agent.name} paused locally (TadpoleOS offline).`,
                severity: 'warning'
            });
            return { should_clear_logs: false, handled: true };
        }
    });

    // ────────────── RESUME ──────────────
    command_registry.register({
        command: '/resume',
        description: 'Resume an autonomous agent',
        handler: async (ctx: Command_Context) => {
            const agent = find_agent(ctx.args[0], ctx.agents);
            if (!agent) return { should_clear_logs: false, handled: true };

            const success = await agent_service.resume_agent(agent.id);
            event_bus.emit_log({
                source: 'System',
                text: success
                    ? `▶️ Agent ${agent.name} resumed via TadpoleOS.`
                    : `▶️ Agent ${agent.name} resumed locally (TadpoleOS offline).`,
                severity: 'success'
            });
            return { should_clear_logs: false, handled: true };
        }
    });

    // ────────────── SWITCH ──────────────
    command_registry.register({
        command: '/switch',
        description: 'Switch active model slot [1-3]',
        handler: async (ctx: Command_Context) => {
            const agent = find_agent(ctx.args[0], ctx.agents);
            const slot_str = ctx.args[1];
            if (agent && slot_str) {
                const slot = parseInt(slot_str) as 1 | 2 | 3;
                if (slot >= 1 && slot <= 3) {
                    await agent_service.update_agent(agent.id, { active_model_slot: slot });
                    event_bus.emit_log({
                        source: 'System',
                        text: `Agent ${agent.name} switched to Neural Slot ${slot}.`,
                        severity: 'success'
                    });
                } else {
                    event_bus.emit_log({ source: 'System', text: 'Invalid slot. Use 1, 2, or 3.', severity: 'error' });
                }
            } else {
                event_bus.emit_log({ source: 'System', text: 'Usage: /switch <agent-name> <1|2|3>', severity: 'error' });
            }
            return { should_clear_logs: false, handled: true };
        }
    });

    // ────────────── SEND ──────────────
    command_registry.register({
        command: '/send',
        description: 'Inject message to agent directive pipeline',
        handler: async (ctx: Command_Context) => {
            const agent = find_agent(ctx.args[0], ctx.agents);
            if (!agent) return { should_clear_logs: false, handled: true };

            const MAX_MSG_LENGTH = 500;
            let message = ctx.args.slice(1).join(' ');
            if (!message) {
                event_bus.emit_log({
                    source: 'System',
                    text: 'Usage: /send <agent-name> <message>',
                    severity: 'error'
                });
                return { should_clear_logs: false, handled: true };
            }

            // Sanitize: strip control characters and enforce length limit
            // eslint-disable-next-line no-control-regex
            message = message.replace(/[\x00-\x1F\x7F]/g, '').trim();
            if (message.length > MAX_MSG_LENGTH) {
                event_bus.emit_log({
                    source: 'System',
                    text: `Message exceeds ${MAX_MSG_LENGTH} character limit (${message.length} chars). Please shorten it.`,
                    severity: 'error'
                });
                return { should_clear_logs: false, handled: true };
            }

            const settings = get_settings();
            const { model_id, provider } = resolve_agent_model_config(agent, settings.default_model);

            event_bus.emit_log({
                source: 'User',
                text: `→ ${agent.name}: ${message}`,
                severity: 'info'
            });

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

            return { should_clear_logs: false, handled: true };
        }
    });
}

// Metadata: [agent_handlers]
