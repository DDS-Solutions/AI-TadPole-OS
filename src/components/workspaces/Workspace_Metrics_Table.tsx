/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Workspace_Metrics_Table]` in observability traces.
 */

import React from 'react';
import { Database } from 'lucide-react';
import { i18n } from '../../i18n';
import { Tooltip } from '../ui';

interface WorkspaceMetricsTableProps {
    clusters: { id: string; name: string; path: string; pending_tasks: unknown[] }[];
    sync_status: Record<string, { file_count?: number; status?: string }>;
}

export const Workspace_Metrics_Table: React.FC<WorkspaceMetricsTableProps> = ({
    clusters,
    sync_status
}) => {
    return (
        <section className="pt-12 border-t border-[color:var(--color-surface)] animate-in fade-in slide-in-from-bottom-4 duration-1000">
            <h2 className="text-xs font-bold text-zinc-600 uppercase tracking-[0.3em] mb-6 flex items-center gap-2">
                <Database size={14} className="text-green-500" />
                {i18n.t('workspaces.header_sync_metrics', { defaultValue: 'Sovereign Synchronization Metrics' })}
            </h2>
            
            <div className="overflow-hidden rounded-2xl border border-[color:var(--color-surface)] bg-[color:var(--color-surface)]/20 backdrop-blur-sm shadow-inner">
                <table className="w-full text-left text-xs font-mono">
                    <thead className="bg-[color:color-mix(in_srgb,var(--color-background)_50%,transparent)] text-zinc-500 uppercase tracking-widest text-[9px]">
                        <tr>
                            <th className="px-6 py-4 font-bold">
                                <Tooltip content={i18n.t('workspaces.tooltip_metric_cluster', { defaultValue: 'The identifier and name of the active swarm cluster' })} position="top">
                                    <span className="cursor-help">{i18n.t('workspaces.metric_cluster')}</span>
                                </Tooltip>
                            </th>
                            <th className="px-6 py-4 font-bold">
                                <Tooltip content={i18n.t('workspaces.tooltip_metric_depth', { defaultValue: 'Total count of scanned files tracked within the cluster workspace' })} position="top">
                                    <span className="cursor-help">{i18n.t('workspaces.metric_depth')}</span>
                                </Tooltip>
                            </th>
                            <th className="px-6 py-4 font-bold">
                                <Tooltip content={i18n.t('workspaces.tooltip_metric_branching', { defaultValue: 'Number of active task drafts and pending file change branches' })} position="top">
                                    <span className="cursor-help">{i18n.t('workspaces.metric_branching')}</span>
                                </Tooltip>
                            </th>
                            <th className="px-6 py-4 font-bold">
                                <Tooltip content={i18n.t('workspaces.tooltip_metric_health', { defaultValue: 'Real-time synchronization status and OKF playbook compliance level' })} position="top">
                                    <span className="cursor-help">{i18n.t('workspaces.metric_health')}</span>
                                </Tooltip>
                            </th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-zinc-900/50">
                        {clusters.map((cluster) => {
                            const status = sync_status ? sync_status[cluster.path] : null;
                            return (
                                <tr key={cluster.id} className="hover:bg-[color:var(--color-surface)]/40 transition-colors">
                                    <td className="px-6 py-4 text-zinc-300 font-bold">{cluster.name}</td>
                                    <td className="px-6 py-4 text-zinc-500">{status?.file_count ?? 0} Files</td>
                                    <td className="px-6 py-4 text-zinc-500">{cluster.pending_tasks.length} Active</td>
                                    <td className="px-6 py-4">
                                        <Tooltip content={i18n.t(`workspaces.tooltip_sync_status_${status?.status || 'idle'}`, { defaultValue: `Cluster is currently ${status?.status || 'idle'}` })} position="top">
                                            <span className={`flex items-center gap-2 cursor-help ${status?.status === 'error' ? 'text-red-400' : 'text-emerald-400'}`}>
                                                <div className={`w-1.5 h-1.5 rounded-full ${status?.status === 'syncing' ? 'bg-amber-500 animate-pulse' : status?.status === 'error' ? 'bg-red-500' : 'bg-emerald-500'}`} />
                                                {(status?.status || 'idle').toUpperCase()}
                                            </span>
                                        </Tooltip>
                                    </td>
                                </tr>
                            );
                        })}
                        {clusters.length === 0 && (
                            <tr>
                                <td colSpan={4} className="px-6 py-10 text-center text-zinc-700 italic">
                                    {i18n.t('workspaces.no_metrics_available', { defaultValue: 'No active synchronization clusters detected.' })}
                                </td>
                            </tr>
                        )}
                    </tbody>
                </table>
            </div>
        </section>
    );
};

// Metadata: [Workspace_Metrics_Table]
