/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Isolated development and execution environment manager. 
 * Orchestrates the visualization and management of sandboxed workspaces for agent swarms via `useWorkspacesManager`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Workspace mount failure (filesystem permissions), or sandboxed process leak.
 * - **Telemetry Link**: Search for `[Workspaces_View]` or `SANDBOX_SYNC` in service logs.
 */

import { Folder } from 'lucide-react';
import { useWorkspacesManager } from '../hooks/useWorkspacesManager';
import { Tooltip } from '../components/ui';
import { i18n } from '../i18n';

// Modular Components
import { Workspace_Cluster } from '../components/workspaces/Workspace_Cluster';
import { Workspace_Metrics_Table } from '../components/workspaces/Workspace_Metrics_Table';

export default function Workspaces() {
    const {
        clusters,
        agents,
        sync_status,
        approve_branch,
        reject_branch,
        format_bytes
    } = useWorkspacesManager();

    return (
        <div className="h-full flex flex-col bg-[color:var(--color-background)]">
            {/* Header */}
            <div className="py-2 px-6 border-b border-[color:var(--color-surface)] bg-[color:color-mix(in_srgb,var(--color-background)_50%,transparent)] backdrop-blur sticky top-0 z-40 flex items-center justify-between">
                <div>
                    <h1 className="text-xl font-bold text-zinc-100 flex items-center gap-2">
                        <Tooltip content={i18n.t('workspaces.tooltip_fs')} position="right">
                            <Folder className="text-green-500 cursor-help" />
                        </Tooltip>
                        {i18n.t('workspaces.title')}</h1>
                    <p className="text-xs text-zinc-500 font-mono mt-0.5 tracking-wide uppercase">
                        {i18n.t('workspaces.label_sync_status', { count: clusters.length })}
                    </p>
                </div>
                <script type="application/ld+json">
                {JSON.stringify({
                  "@context": "https://schema.org",
                  "@type": "Dataset",
                  "name": "Tadpole OS Workspaces",
                  "description": "Orchestration environment for agent swarm clusters and sandboxed execution units.",
                  "creator": {
                    "@type": "Person",
                    "name": "Agent of Nine"
                  }
                })}
                </script>
            </div>

            <div className="space-y-12 animate-in fade-in slide-in-from-bottom-4 duration-700 pb-20 max-w-7xl mx-auto h-full overflow-y-auto custom-scrollbar px-6 pt-6">
                {/* Mission Clusters */}
                {clusters.map((cluster) => (
                    <Workspace_Cluster 
                        key={cluster.id}
                        cluster={cluster}
                        agents={agents}
                        sync_status={sync_status}
                        format_bytes={format_bytes}
                        on_approve={approve_branch}
                        on_reject={reject_branch}
                    />
                ))}

                {/* Synchronization Metrics */}
                <Workspace_Metrics_Table 
                    clusters={clusters}
                    sync_status={sync_status}
                />

                {/* Legacy Silos */}
                <section className="pt-8 border-t border-[color:var(--color-surface)]">
                    <h2 className="text-xs font-bold text-zinc-600 uppercase tracking-[0.3em] mb-4">{i18n.t('workspaces.header_legacy_silos')}</h2>
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-4 opacity-40 grayscale group hover:grayscale-0 hover:opacity-100 transition-all">
                        {agents.filter(a => !clusters.some(c => c.collaborators.includes(a.id))).map(agent => (
                            <div key={agent.id} className="p-3 bg-[color:var(--color-surface)]/30 border border-[color:var(--color-border)] rounded-xl flex items-center gap-3">
                                <Folder size={16} className="text-zinc-600" />
                                <span className="text-xs font-mono text-zinc-400 truncate">{agent.name}</span>
                            </div>
                        ))}
                    </div>
                </section>
            </div>
        </div>
    );
}

// Metadata: [Workspaces]

// Metadata: [Workspaces]
