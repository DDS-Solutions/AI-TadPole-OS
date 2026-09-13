/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / swarm_handlers
 * - **Primary Entrypoints**: `register_swarm_commands`
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
import { use_workspace_store } from '../../stores/workspace_store';
import { command_registry } from './registry';
import type { Command_Context } from './types';

export function register_swarm_commands() {
    // ────────────── SWARM ──────────────
    command_registry.register({
        command: '/swarm',
        description: 'Global swarm management <status|optimize>',
        timeout_ms: 15000,
        handler: async (ctx: Command_Context) => {
            const workspace_store = use_workspace_store.getState();
            const sub_cmd = ctx.args[0]?.toLowerCase();

            if (sub_cmd === 'status') {
                const cluster_info = workspace_store.clusters.map(c => {
                    const alpha_agent = ctx.agents.find(a => a.id === c.alpha_id);
                    const theme = (c.theme || 'zinc').toUpperCase();
                    const collabs = (c.collaborators || []).length;
                    return (
                        `🔹 ${c.name} [${theme}]\n` +
                        `  Alpha: ${alpha_agent?.name || 'NONE'}\n` +
                        `  Objective: ${c.objective || 'No objective set'}\n` +
                        `  Collaborators: ${collabs}`
                    );
                }).join('\n\n');

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

                for (const cluster of workspace_store.clusters) {
                    try {
                        const proposal = await workspace_store.generate_proposal(cluster.id, true);
                        if (proposal) {
                            const alpha_agent = ctx.agents.find(a => a.id === cluster.alpha_id);
                            event_bus.emit_log({
                                source: 'Agent',
                                agent_id: cluster.alpha_id || 'alpha-node',
                                agent_name: alpha_agent?.name || 'Alpha Node',
                                text: `[Optimization Proposal for ${cluster.name}]: ${proposal.reasoning}`,
                                severity: 'info'
                            });
                        }
                    } catch (err) {
                        event_bus.emit_log({
                            source: 'System',
                            text: `Failed to generate optimization proposal for cluster ${cluster.name}: ${err instanceof Error ? err.message : String(err)}`,
                            severity: 'error'
                        });
                    }
                }
            } else {
                event_bus.emit_log({
                    source: 'System',
                    text: 'Usage: /swarm <status|optimize>',
                    severity: 'error'
                });
            }
            return { should_clear_logs: false, handled: true };
        }
    });
}
