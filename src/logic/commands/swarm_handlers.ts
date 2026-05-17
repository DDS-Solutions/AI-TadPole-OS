/**
 * @docs ARCHITECTURE:Logic:Commands
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
        handler: async (ctx: Command_Context) => {
            const workspace_store = use_workspace_store.getState();
            const sub_cmd = ctx.args[0]?.toLowerCase();

            if (sub_cmd === 'status') {
                const cluster_info = workspace_store.clusters.map(c =>
                    `🔹 ${c.name} [${c.theme.toUpperCase()}]\n` +
                    `  Alpha: ${ctx.agents.find(a => a.id === c.alpha_id)?.name || 'NONE'}\n` +
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
                                agent_id: ctx.agents.find(a => a.id === cluster.alpha_id)?.name || 'Alpha Node',
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
            return { should_clear_logs: false, handled: true };
        }
    });
}
