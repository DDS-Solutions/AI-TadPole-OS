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
import { agent_api_service } from '../services/agent_api_service';
import { system_api_service } from '../services/system_api_service';
import { resolve_agent_model_config } from '../utils/model_utils';
import { use_workspace_store } from '../stores/workspace_store';
import { use_sovereign_store } from '../stores/sovereign_store';
import type { Agent } from '../types';

/** Return value from process_command indicating if the log should be cleared. */
export interface Command_Result {
    /** If true, the Terminal should wipe its local log state. */
    should_clear_logs: boolean;
}

/**
 * process_command
 * Processes a single slash-command string from the user.
 * Supports standard slash commands (/help, /clear), agent-specific targeting (@agent), 
 * and cluster-specific targeting (#cluster).
 * 
 * Refactored for strict snake_case compliance and backend parity.
 */
export async function process_command(
    command_text: string,
    agents: Agent[],
    is_safe_mode?: boolean
): Promise<Command_Result> {
    // 1. Lexical Analysis: Split by spaces but preserve quoted strings (e.g. "quoted msg")
    const parts: string[] = [];
    const regex = /[^\s"']+|"([^"]*)"|'([^']*)'/g;
    let match;
    while ((match = regex.exec(command_text)) !== null) {
        // match[1] or [2] contains the content inside quotes, match[0] is the fallback for unquoted words.
        parts.push(match[1] || match[2] || match[0]);
    }

    if (parts.length === 0) return { should_clear_logs: false };
    const cmd = parts[0].toLowerCase();
    const args = parts.slice(1);

    // Build O(1) lookup indexes for agent resolution (name + id + partial match)
    const agent_by_name = new Map<string, Agent>();
    const agent_by_id = new Map<string, Agent>();
    for (const a of agents) {
        agent_by_name.set(a.name.toLowerCase(), a);
        agent_by_id.set(String(a.id), a);
    }
    
    /**
     * Resolves an agent by exact name, partial name, or ID.
     * Emits an error to the event_bus if unresolvable.
     */
    const find_agent = (name_or_id: string | undefined): Agent | null => {
        if (!name_or_id) {
            event_bus.emit_log({ source: 'System', text: 'Missing agent name. Usage: /<command> <agent-name>', severity: 'error' });
            return null;
        }
        const lower = name_or_id.toLowerCase();
        const found = agent_by_name.get(lower)
            || agent_by_id.get(name_or_id)
            || agents.find(a => a.name.toLowerCase().includes(lower)); // partial match fallback
        if (!found) {
            event_bus.emit_log({ source: 'System', text: `Agent "${name_or_id}" not found. Available: ${agents.map(a => a.name).slice(0, 8).join(', ')}...`, severity: 'error' });
            return null;
        }
        return found;
    };

    switch (cmd) {
        // ────────────── HELP ──────────────
        case '/help': {
            event_bus.emit_log({
                source: 'System',
                text: [
                    '📋 Available Commands:',
                    '  /help              — Show this list',
                    '  /clear             — Clear terminal',
                    '  /status            — Agent swarm summary',
                    '  /deploy            — Trigger deploy simulation',
                    '  /config <name>     — View agent config',
                    '  /switch <name> [1-3] — Switch active model slot',
                    '  /pause <name>      — Pause an agent',
                    '  /resume <name>     — Resume an agent',
                    '  /send <name> <msg> — Inject message to agent',
                    '  /swarm status      — Show mission clusters',
                    '  /swarm optimize    — Trigger reconfiguration',
                ].join('\n'),
                severity: 'info'
            });
            return { should_clear_logs: false };
        }

        // ────────────── CLEAR ──────────────
        case '/clear': {
            event_bus.clear_history();
            return { should_clear_logs: true };
        }

        // ────────────── STATUS ──────────────
        case '/status': {
            const active = agents.filter(a => a.status === 'active' || a.status === 'thinking' || a.status === 'coding').length;
            const idle = agents.filter(a => a.status === 'idle').length;
            const offline = agents.filter(a => a.status === 'offline').length;
            const total_tokens = agents.reduce((sum, a) => sum + a.tokens_used, 0);

            event_bus.emit_log({
                source: 'System',
                text: `Swarm Status: ${active} active · ${idle} idle · ${offline} offline | Total tokens: ${(total_tokens / 1000).toFixed(1)}k`,
                severity: 'success'
            });
            return { should_clear_logs: false };
        }

        // ────────────── DEPLOY (2-step confirmation) ──────────────
        case '/deploy': {
            if (args[0]?.toLowerCase() !== 'confirm') {
                event_bus.emit_log({
                    source: 'System',
                    text: '⚠️ This will trigger a production deployment to Swarm Bunker. Type "/deploy confirm" to proceed.',
                    severity: 'warning'
                });
                return { should_clear_logs: false };
            }

            event_bus.emit_log({
                source: 'System',
                text: '🚀 Triggering deployment to Swarm Bunker via /engine/deploy...',
                severity: 'warning'
            });

            try {
                const data = await system_api_service.deploy_engine();
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
            return { should_clear_logs: false };
        }

        // ────────────── CONFIG ──────────────
        case '/config': {
            const agent = find_agent(args[0]);
            if (!agent) return { should_clear_logs: false };

            event_bus.emit_log({
                source: 'System',
                text: [
                    `⚙️ Config for ${agent.name}:`,
                    `  Model: ${agent.model}`,
                    `  Temperature: ${agent.model_config?.temperature ?? 'default'}`,
                    `  Status: ${agent.status}`,
                    `  Prompt: ${agent.model_config?.system_prompt ? agent.model_config.system_prompt.substring(0, 80) + '...' : '(none)'}`,
                ].join('\n'),
                severity: 'info'
            });
            return { should_clear_logs: false };
        }

        // ────────────── PAUSE ──────────────
        case '/pause': {
            const agent = find_agent(args[0]);
            if (!agent) return { should_clear_logs: false };

            const success = await agent_api_service.pause_agent(agent.id);
            event_bus.emit_log({
                source: 'System',
                text: success
                    ? `⏸️ Agent ${agent.name} paused via TadpoleOS.`
                    : `⏸️ Agent ${agent.name} paused locally (TadpoleOS offline).`,
                severity: 'warning'
            });
            return { should_clear_logs: false };
        }

        // ────────────── RESUME ──────────────
        case '/resume': {
            const agent = find_agent(args[0]);
            if (!agent) return { should_clear_logs: false };

            const success = await agent_api_service.resume_agent(agent.id);
            event_bus.emit_log({
                source: 'System',
                text: success
                    ? `▶️ Agent ${agent.name} resumed via TadpoleOS.`
                    : `▶️ Agent ${agent.name} resumed locally (TadpoleOS offline).`,
                severity: 'success'
            });
            return { should_clear_logs: false };
        }

        // ────────────── SEND (sanitized) ──────────────
        case '/send': {
            const agent = find_agent(args[0]);
            if (!agent) return { should_clear_logs: false };

            const MAX_MSG_LENGTH = 500;
            let message = args.slice(1).join(' ');
            if (!message) {
                event_bus.emit_log({
                    source: 'System',
                    text: 'Usage: /send <agent-name> <message>',
                    severity: 'error'
                });
                return { should_clear_logs: false };
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
                return { should_clear_logs: false };
            }

            const { model_id, provider } = resolve_agent_model_config(agent);

            // 1. Immediate User Echo: Show outgoing directive in the central log.
            event_bus.emit_log({
                source: 'User',
                text: `→ ${agent.name}: ${message}`,
                severity: 'info'
            });

            // 2. Immediate System Acknowledgment: Confirm the routing attempt.
            setTimeout(() => {
                const reply = `Neural Link: Routing directive to ${agent.name}...`;
                event_bus.emit_log({
                    source: 'System',
                    text: reply,
                    severity: 'info'
                });
                // Update persistent sovereign store for the Agent-specific scope
                use_sovereign_store.getState().add_message({
                    sender_id: 'system',
                    sender_name: 'Neural System',
                    agent_id: agent.id, 
                    text: reply,
                    scope: 'agent'
                });
            }, 100);

            // 3. Trigger API Call
            agent_api_service.send_command(agent.id, message, model_id, provider, undefined, undefined, undefined, undefined, is_safe_mode)
                .catch(err => {
                    event_bus.emit_log({
                        source: 'System',
                        text: `Neural link failed: ${err.message || err}`,
                        severity: 'error'
                    });
                });

            return { should_clear_logs: false };
        }

        // ────────────── SWARM ──────────────
        case '/swarm': {
            const workspace_store = use_workspace_store.getState();
            const sub_cmd = args[0]?.toLowerCase();

            if (sub_cmd === 'status') {
                const cluster_info = workspace_store.clusters.map(c =>
                    `🔹 ${c.name} [${c.theme.toUpperCase()}]\n` +
                    `  Alpha: ${agents.find(a => a.id === c.alpha_id)?.name || 'NONE'}\n` +
                    `  Objective: ${c.objective || 'No objective set'}\n` +
                    `  Collaborators: ${c.collaborators.length}`
                ).join('\n\n');

                event_bus.emit_log({
                    source: 'System',
                    text: `🌐 Mission Cluster Inventory:\n\n${cluster_info}`,
                    severity: 'info'
                });
            } else if (sub_cmd === 'optimize') {
                event_bus.emit_log({
                    source: 'System',
                    text: '⚡ Initiating global swarm optimization...',
                    severity: 'warning'
                });

                workspace_store.clusters.forEach(cluster => {
                    workspace_store.generate_proposal(cluster.id);
                    const proposal = use_workspace_store.getState().active_proposals[cluster.id];

                    if (proposal) {
                        setTimeout(() => {
                            event_bus.emit_log({
                                source: 'Agent',
                                agent_id: agents.find(a => a.id === cluster.alpha_id)?.name || 'Alpha Node',
                                text: proposal.reasoning,
                                severity: 'info'
                            });
                        }, 500 + Math.random() * 1000);
                    }
                });
            } else {
                event_bus.emit_log({
                    source: 'System',
                    text: 'Usage: /swarm <status|optimize>',
                    severity: 'error'
                });
            }
            return { should_clear_logs: false };
        }

        // ────────────── SWITCH ──────────────
        case '/switch': {
            const agent = find_agent(args[0]);
            const slot_str = args[1];
            if (agent && slot_str) {
                const slot = parseInt(slot_str) as 1 | 2 | 3;
                if (slot >= 1 && slot <= 3) {
                    await agent_api_service.update_agent(agent.id, { active_model_slot: slot });
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
            return { should_clear_logs: false };
        }

        // ────────────── UNKNOWN / SPECIAL ──────────────
        default: {
            // Check for conversational targeting (@agent)
            if (cmd.startsWith('@')) {
                const target_name = cmd.substring(1);
                const agent = find_agent(target_name);
                if (agent) {
                    const message = args.join(' ');
                    const { model_id, provider } = resolve_agent_model_config(agent);

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

                    agent_api_service.send_command(agent.id, message, model_id, provider, undefined, undefined, undefined, undefined, is_safe_mode)
                        .catch(err => {
                            event_bus.emit_log({
                                source: 'System',
                                text: `Neural link failed: ${err.message || err}`,
                                severity: 'error'
                            });
                        });
                }
                return { should_clear_logs: false };
            }

            // Check for cluster targeting (#cluster)
            if (cmd.startsWith('#')) {
                const cluster_name = cmd.substring(1).toLowerCase();
                const workspace_store = use_workspace_store.getState();
                const cluster = workspace_store.clusters.find(c => (c.name?.toLowerCase() === cluster_name) || c.id === cluster_name);

                if (cluster && cluster.alpha_id) {
                    const alpha_agent = agents.find(a => a.id === cluster.alpha_id);
                    if (alpha_agent) {
                        const message = args.join(' ');
                        const { model_id, provider } = resolve_agent_model_config(alpha_agent);

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

                        agent_api_service.send_command(alpha_agent.id, message, model_id, provider, cluster.id, cluster.department, undefined, undefined, is_safe_mode)
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
                return { should_clear_logs: false };
            }

            // General swarm directive (no prefix)
            if (!cmd.startsWith('/')) {
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

                return { should_clear_logs: false };
            }

            // Fallback for unknown slash commands
            event_bus.emit_log({
                source: 'System',
                text: `Unknown command: ${cmd}. Type /help for available commands.`,
                severity: 'error'
            });
            return { should_clear_logs: false };
        }
    }
}

