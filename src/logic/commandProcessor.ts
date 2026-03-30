import { EventBus } from '../services/eventBus';
import { AgentApiService } from '../services/AgentApiService';
import { SystemApiService } from '../services/SystemApiService';
import { resolveAgentModelConfig } from '../utils/modelUtils';
import { useWorkspaceStore } from '../stores/workspaceStore';
import { useSovereignStore } from '../stores/sovereignStore';
import type { Agent } from '../types';

/** Return value from processCommand indicating if the log should be cleared. */
export interface CommandResult {
    /** If true, the Terminal should wipe its local log state. */
    shouldClearLogs: boolean;
}

/**
 * Processes a single slash-command string from the user.
 * Supports standard slash commands (/help, /clear), agent-specific targeting (@agent), 
 * and cluster-specific targeting (#cluster).
 * 
 * @param commandText - The raw input from the Terminal UI.
 * @param agents - Current active agent registry for lookup.
 * @param isSafeMode - If true, restricts executable actions (e.g., skip file mutations).
 * @returns A {@link CommandResult} indicating UI side-effects like log clearing.
 */
export async function processCommand(
    commandText: string,
    agents: Agent[],
    isSafeMode?: boolean
): Promise<CommandResult> {
    // 1. Lexical Analysis: Split by spaces but preserve quoted strings (e.g. "quoted msg")
    const parts: string[] = [];
    const regex = /[^\s"']+|"([^"]*)"|'([^']*)'/g;
    let match;
    while ((match = regex.exec(commandText)) !== null) {
        // match[1] or [2] contains the content inside quotes, match[0] is the fallback for unquoted words.
        parts.push(match[1] || match[2] || match[0]);
    }

    if (parts.length === 0) return { shouldClearLogs: false };
    const cmd = parts[0].toLowerCase();
    const args = parts.slice(1);

    // Build O(1) lookup indexes for agent resolution (name + id + partial match)
    const agentByName = new Map<string, Agent>();
    const agentById = new Map<string, Agent>();
    for (const a of agents) {
        agentByName.set(a.name.toLowerCase(), a);
        agentById.set(String(a.id), a);
    }
    /**
     * Resolves an agent by exact name, partial name, or ID.
     * Emits an error to the EventBus if unresolvable.
     */
    const findAgent = (nameOrId: string | undefined): Agent | null => {
        if (!nameOrId) {
            EventBus.emit({ source: 'System', text: 'Missing agent name. Usage: /<command> <agent-name>', severity: 'error' });
            return null;
        }
        const lower = nameOrId.toLowerCase();
        const found = agentByName.get(lower)
            || agentById.get(nameOrId)
            || agents.find(a => a.name.toLowerCase().includes(lower)); // partial match fallback
        if (!found) {
            EventBus.emit({ source: 'System', text: `Agent "${nameOrId}" not found. Available: ${agents.map(a => a.name).slice(0, 8).join(', ')}...`, severity: 'error' });
            return null;
        }
        return found;
    };

    switch (cmd) {
        // ────────────── HELP ──────────────
        case '/help': {
            EventBus.emit({
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
            return { shouldClearLogs: false };
        }

        // ────────────── CLEAR ──────────────
        case '/clear': {
            EventBus.clearHistory();
            return { shouldClearLogs: true };
        }

        // ────────────── STATUS ──────────────
        case '/status': {
            const active = agents.filter(a => a.status === 'active' || a.status === 'thinking' || a.status === 'coding').length;
            const idle = agents.filter(a => a.status === 'idle').length;
            const offline = agents.filter(a => a.status === 'offline').length;
            const totalTokens = agents.reduce((sum, a) => sum + a.tokensUsed, 0);

            EventBus.emit({
                source: 'System',
                text: `Swarm Status: ${active} active · ${idle} idle · ${offline} offline | Total tokens: ${(totalTokens / 1000).toFixed(1)}k`,
                severity: 'success'
            });
            return { shouldClearLogs: false };
        }

        // ────────────── DEPLOY (2-step confirmation) ──────────────
        // Requires a deliberate "confirm" suffix to prevent accidental production triggers.
        case '/deploy': {
            if (args[0]?.toLowerCase() !== 'confirm') {
                EventBus.emit({
                    source: 'System',
                    text: '⚠️ This will trigger a production deployment to Swarm Bunker. Type "/deploy confirm" to proceed.',
                    severity: 'warning'
                });
                return { shouldClearLogs: false };
            }

            EventBus.emit({
                source: 'System',
                text: '🚀 Triggering deployment to Swarm Bunker via /engine/deploy...',
                severity: 'warning'
            });

            try {
                const data = await SystemApiService.deployEngine();
                EventBus.emit({
                    source: 'System',
                    text: `✅ Deployment successful. Output: ${(data.output || '').slice(-300)}`,
                    severity: 'success'
                });
            } catch (e: unknown) {
                const errorMsg = e instanceof Error ? e.message : String(e);
                EventBus.emit({
                    source: 'System',
                    text: `❌ Deployment error: ${errorMsg}`,
                    severity: 'error'
                });
            }
            return { shouldClearLogs: false };
        }


        // ────────────── CONFIG ──────────────
        case '/config': {
            const agent = findAgent(args[0]);
            if (!agent) return { shouldClearLogs: false };

            EventBus.emit({
                source: 'System',
                text: [
                    `⚙️ Config for ${agent.name}:`,
                    `  Model: ${agent.model}`,
                    `  Temperature: ${agent.modelConfig?.temperature ?? 'default'}`,
                    `  Status: ${agent.status}`,
                    `  Prompt: ${agent.modelConfig?.systemPrompt ? agent.modelConfig.systemPrompt.substring(0, 80) + '...' : '(none)'}`,
                ].join('\n'),
                severity: 'info'
            });
            return { shouldClearLogs: false };
        }

        // ────────────── PAUSE ──────────────
        case '/pause': {
            const agent = findAgent(args[0]);
            if (!agent) return { shouldClearLogs: false };

            const success = await AgentApiService.pauseAgent(agent.id);
            EventBus.emit({
                source: 'System',
                text: success
                    ? `⏸️ Agent ${agent.name} paused via TadpoleOS.`
                    : `⏸️ Agent ${agent.name} paused locally (TadpoleOS offline).`,
                severity: 'warning'
            });
            return { shouldClearLogs: false };
        }

        // ────────────── RESUME ──────────────
        case '/resume': {
            const agent = findAgent(args[0]);
            if (!agent) return { shouldClearLogs: false };

            const success = await AgentApiService.resumeAgent(agent.id);
            EventBus.emit({
                source: 'System',
                text: success
                    ? `▶️ Agent ${agent.name} resumed via TadpoleOS.`
                    : `▶️ Agent ${agent.name} resumed locally (TadpoleOS offline).`,
                severity: 'success'
            });
            return { shouldClearLogs: false };
        }

        // ────────────── SEND (sanitized) ──────────────
        case '/send': {
            const agent = findAgent(args[0]);
            if (!agent) return { shouldClearLogs: false };

            const MAX_MSG_LENGTH = 500;
            let message = args.slice(1).join(' ');
            if (!message) {
                EventBus.emit({
                    source: 'System',
                    text: 'Usage: /send <agent-name> <message>',
                    severity: 'error'
                });
                return { shouldClearLogs: false };
            }

            // Sanitize: strip control characters and enforce length limit
            // eslint-disable-next-line no-control-regex
            message = message.replace(/[\x00-\x1F\x7F]/g, '').trim();
            if (message.length > MAX_MSG_LENGTH) {
                EventBus.emit({
                    source: 'System',
                    text: `Message exceeds ${MAX_MSG_LENGTH} character limit (${message.length} chars). Please shorten it.`,
                    severity: 'error'
                });
                return { shouldClearLogs: false };
            }

            const { modelId, provider } = resolveAgentModelConfig(agent);

            // 1. Immediate User Echo: Show outgoing directive in the central log.
            EventBus.emit({
                source: 'User',
                text: `→ ${agent.name}: ${message}`,
                severity: 'info'
            });

            // 2. Immediate System Acknowledgment: Confirm the routing attempt.
            setTimeout(() => {
                const reply = `Neural Link: Routing directive to ${agent.name}...`;
                EventBus.emit({
                    source: 'System',
                    text: reply,
                    severity: 'info'
                });
                // Update persistent sovereign store for the Agent-specific scope
                useSovereignStore.getState().addMessage({
                    senderId: 'system',
                    senderName: 'Neural System',
                    agentId: agent.id, // CRITICAL: Required for filter in Agent scope
                    text: reply,
                    scope: 'agent'
                });
            }, 100);

            // 3. Trigger API Call: Non-blocking fire-and-forget to the Rust engine.
            AgentApiService.sendCommand(agent.id, message, modelId, provider, undefined, undefined, undefined, undefined, isSafeMode)
                .catch(err => {
                    EventBus.emit({
                        source: 'System',
                        text: `Neural link failed: ${err.message || err}`,
                        severity: 'error'
                    });
                });

            return { shouldClearLogs: false };
        }

        // ────────────── SWARM ──────────────
        case '/swarm': {
            const workspaceStore = useWorkspaceStore.getState();
            const subCmd = args[0]?.toLowerCase();

            if (subCmd === 'status') {
                const clusterInfo = workspaceStore.clusters.map(c =>
                    `🔹 ${c.name} [${c.theme.toUpperCase()}]\n` +
                    `  Alpha: ${agents.find(a => a.id === c.alphaId)?.name || 'NONE'}\n` +
                    `  Objective: ${c.objective || 'No objective set'}\n` +
                    `  Collaborators: ${c.collaborators.length}`
                ).join('\n\n');

                EventBus.emit({
                    source: 'System',
                    text: `🌐 Mission Cluster Inventory:\n\n${clusterInfo}`,
                    severity: 'info'
                });
            } else if (subCmd === 'optimize') {
                EventBus.emit({
                    source: 'System',
                    text: '⚡ Initiating global swarm optimization...',
                    severity: 'warning'
                });

                workspaceStore.clusters.forEach(cluster => {
                    workspaceStore.generateProposal(cluster.id);
                    const proposal = useWorkspaceStore.getState().activeProposals[cluster.id];

                    if (proposal) {
                        setTimeout(() => {
                            EventBus.emit({
                                source: 'Agent',
                                agentId: agents.find(a => a.id === cluster.alphaId)?.name || 'Alpha Node',
                                text: proposal.reasoning,
                                severity: 'info'
                            });
                        }, 500 + Math.random() * 1000);
                    }
                });
            } else {
                EventBus.emit({
                    source: 'System',
                    text: 'Usage: /swarm <status|optimize>',
                    severity: 'error'
                });
            }
            return { shouldClearLogs: false };
        }

        // ────────────── SWITCH ──────────────
        case '/switch': {
            const agent = findAgent(args[0]);
            const slotStr = args[1];
            if (agent && slotStr) {
                const slot = parseInt(slotStr) as 1 | 2 | 3;
                if (slot >= 1 && slot <= 3) {
                    await AgentApiService.updateAgent(agent.id, { activeModelSlot: slot });
                    EventBus.emit({
                        source: 'System',
                        text: `Agent ${agent.name} switched to Neural Slot ${slot}.`,
                        severity: 'success'
                    });
                } else {
                    EventBus.emit({ source: 'System', text: 'Invalid slot. Use 1, 2, or 3.', severity: 'error' });
                }
            } else {
                EventBus.emit({ source: 'System', text: 'Usage: /switch <agent-name> <1|2|3>', severity: 'error' });
            }
            return { shouldClearLogs: false };
        }

        // ────────────── UNKNOWN / SPECIAL ──────────────
        default: {
            // Check for conversational targeting (@agent)
            if (cmd.startsWith('@')) {
                const targetName = cmd.substring(1);
                const agent = findAgent(targetName);
                if (agent) {
                    const message = args.join(' ');
                    const { modelId, provider } = resolveAgentModelConfig(agent);

                    // 1. Immediate User Echo to Log
                    EventBus.emit({ source: 'User', text: `→ @${agent.name}: ${message}`, severity: 'info' });

                    // 2. Immediate System Acknowledgment
                    setTimeout(() => {
                        const reply = `Neural Link: Routing directive to ${agent.name}...`;
                        EventBus.emit({
                            source: 'System',
                            text: reply,
                            severity: 'info'
                        });
                        useSovereignStore.getState().addMessage({
                            senderId: 'system',
                            senderName: 'Neural System',
                            agentId: agent.id, // CRITICAL: Required for filter in Agent scope
                            text: reply,
                            scope: 'agent'
                        });
                    }, 100);

                    // 3. Trigger API Call
                    AgentApiService.sendCommand(agent.id, message, modelId, provider, undefined, undefined, undefined, undefined, isSafeMode)
                        .catch(err => {
                            EventBus.emit({
                                source: 'System',
                                text: `Neural link failed: ${err.message || err}`,
                                severity: 'error'
                            });
                        });
                }
                return { shouldClearLogs: false };
            }

            // Check for cluster targeting (#cluster)
            if (cmd.startsWith('#')) {
                const clusterName = cmd.substring(1).toLowerCase();
                const workspaceStore = useWorkspaceStore.getState();
                const cluster = workspaceStore.clusters.find(c => (c.name?.toLowerCase() === clusterName) || c.id === clusterName);

                if (cluster && cluster.alphaId) {
                    const alphaAgent = agents.find(a => a.id === cluster.alphaId);
                    if (alphaAgent) {
                        const message = args.join(' ');
                        const { modelId, provider } = resolveAgentModelConfig(alphaAgent);

                        // 1. Immediate User Echo
                        EventBus.emit({ source: 'User', text: `→ #${cluster.name}: ${message}`, severity: 'info' });

                        // 2. Immediate System Acknowledgment
                        setTimeout(() => {
                            const reply = `Neural Link: Distributing directive to ${cluster.name}...`;
                            EventBus.emit({
                                source: 'System',
                                text: reply,
                                severity: 'info'
                            });
                            useSovereignStore.getState().addMessage({
                                senderId: 'system',
                                senderName: 'Neural System',
                                text: reply,
                                scope: 'cluster'
                            });
                        }, 100);

                        // 3. Trigger API Call
                        AgentApiService.sendCommand(alphaAgent.id, message, modelId, provider, cluster.id, cluster.department, undefined, undefined, isSafeMode)
                            .catch(err => {
                                EventBus.emit({
                                    source: 'System',
                                    text: `Cluster link failed: ${err.message || err}`,
                                    severity: 'error'
                                });
                            });
                    }
                } else {
                    EventBus.emit({
                        source: 'System',
                        text: `Cluster "${clusterName}" not found or lacks an Alpha node.`,
                        severity: 'error'
                    });
                }
                return { shouldClearLogs: false };
            }

            // General swarm directive (no prefix)
            if (!cmd.startsWith('/')) {
                const message = parts.join(' ');

                // 1. User Echo to Log
                EventBus.emit({ source: 'User', text: `Swarm Broadcast: ${message}`, severity: 'info' });

                // 2. System/Swarm Acknowledgment
                const reply = `Broadcasting to swarm: ${message.substring(0, 30)}...`;
                EventBus.emit({
                    source: 'System',
                    text: reply,
                    severity: 'info'
                });
                useSovereignStore.getState().addMessage({
                    senderId: 'system',
                    senderName: 'Neural System',
                    text: reply,
                    scope: 'swarm'
                });

                // Note: Broadcast API would be called here if available
                return { shouldClearLogs: false };
            }

            // Fallback for unknown slash commands
            EventBus.emit({
                source: 'System',
                text: `Unknown command: ${cmd}. Type /help for available commands.`,
                severity: 'error'
            });
            return { shouldClearLogs: false };
        }
    }
}
