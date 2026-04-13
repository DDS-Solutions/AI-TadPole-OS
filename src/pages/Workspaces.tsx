/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Isolated development and execution environment manager. 
 * Orchestrates the visualization and management of sandboxed workspaces for agent swarms.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Workspace mount failure (filesystem permissions), or sandboxed process leak.
 * - **Telemetry Link**: Search for `[Workspaces_View]` or `SANDBOX_SYNC` in service logs.
 */

import { useState, useEffect } from 'react';
import { Folder, Database, Globe, Code2, Server, Users, ArrowUpRight, CheckCircle2, XCircle, Clock } from 'lucide-react';
import { use_workspace_store } from '../stores/workspace_store';
import { Tooltip } from '../components/ui';
import { load_agents } from '../services/agent_service';
import { i18n } from '../i18n';
import type { Agent } from '../types';

export default function Workspaces() {
    const { clusters, approve_branch, reject_branch } = use_workspace_store();
    const [agents, set_agents] = useState<Agent[]>([]);

    useEffect(() => {
        load_agents().then(set_agents);
    }, []);

    return (
        <div className="h-full flex flex-col bg-zinc-950">
            {/* Header */}
            <div className="py-2 px-6 border-b border-zinc-900 bg-zinc-950/50 backdrop-blur sticky top-0 z-40 flex items-center justify-between">
                <div>
                    <h2 className="text-xl font-bold text-zinc-100 flex items-center gap-2">
                        <Tooltip content={i18n.t('workspaces.tooltip_fs')} position="right">
                            <Folder className="text-blue-500 cursor-help" />
                        </Tooltip>
                        {i18n.t('workspaces.title')}</h2>
                    <p className="text-xs text-zinc-500 font-mono mt-0.5 tracking-wide uppercase">
                        {i18n.t('workspaces.label_sync_status', { count: clusters.length })}
                    </p>
                </div>
            </div>

            <div className="space-y-12 animate-in fade-in slide-in-from-bottom-4 duration-700 pb-20 max-w-7xl mx-auto h-full overflow-y-auto custom-scrollbar px-6 pt-6">

                {/* Mission Clusters Grouped by Department */}
                {(clusters || []).map((cluster) => (
                    <section key={cluster.id} className="space-y-6">
                        <div className="flex items-center justify-between border-b border-zinc-900 pb-2">
                            <div className="flex items-center gap-3">
                                <Tooltip content={i18n.t('workspaces.tooltip_dept')} position="right">
                                    <div className={`p-1.5 rounded-lg border bg-zinc-900 cursor-help ${cluster.department === 'Executive' ? 'border-amber-500/30 text-amber-400' :
                                        cluster.department === 'Engineering' ? 'border-blue-500/30 text-blue-400' : 'border-emerald-500/30 text-emerald-400'
                                        }`}>
                                        <Users size={16} />
                                    </div>
                                </Tooltip>
                                <div>
                                    <h2 className="text-lg font-bold text-zinc-100 tracking-tight">{cluster.name.toUpperCase()}</h2>
                                    <p className="text-xs text-zinc-500 font-mono tracking-widest mt-0.5">{i18n.t('workspaces.label_cluster_info', { dept: cluster.department, path: cluster.path })}</p>
                                </div>
                            </div>
                            <div className="flex -space-x-2 p-1">
                                {(cluster.collaborators || []).map(id => {
                                    const agent = agents.find(a => a.id === id);
                                    const is_alpha = cluster.alpha_id === id;
                                    const avatar_color = agent?.theme_color || (is_alpha ? '#f59e0b' : undefined);
                                    return (
                                        <Tooltip key={id} content={`${agent?.name || 'Unknown Agent'} ${is_alpha ? i18n.t('workspaces.tooltip_alpha') : ''}`}>
                                            <div
                                                className={`w-7 h-7 rounded-full border-2 border-zinc-950 flex items-center justify-center transition-colors relative`}
                                                style={{
                                                    backgroundColor: avatar_color ? `${avatar_color}20` : '#18181b',
                                                    borderColor: avatar_color || '#27272a'
                                                }}
                                            >
                                                <span className="text-[10px] font-bold" style={{ color: avatar_color || '#71717a' }}>
                                                    {agent?.name?.[0].toUpperCase() || '?'}
                                                </span>
                                                {is_alpha && (
                                                    <div className="absolute -top-0.5 -right-0.5 w-2.5 h-2.5 rounded-full bg-amber-400 border border-zinc-950 shadow-[0_0_8px_rgba(245,158,11,0.6)]" />
                                                )}
                                            </div>
                                        </Tooltip>
                                    );
                                })}
                            </div>
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
                            {/* Workspace Details Card */}
                            <div className="bg-zinc-900 border border-zinc-800 p-5 rounded-2xl group hover:border-zinc-700 transition-all flex flex-col gap-4 relative overflow-hidden shadow-2xl">
                                <div className="absolute top-0 right-0 p-3 opacity-10 group-hover:opacity-20 transition-opacity">
                                    <Folder size={48} />
                                </div>

                                <div className="flex items-center gap-3">
                                    <Tooltip content={i18n.t('workspaces.tooltip_root')} position="top">
                                        <div className="p-2.5 bg-zinc-900 border border-zinc-800 rounded-xl cursor-help">
                                            <Database size={20} className="text-zinc-500" />
                                        </div>
                                    </Tooltip>
                                    <div>
                                        <h3 className="font-bold text-zinc-200 text-sm">{i18n.t('workspaces.label_root_title')}</h3>
                                        <p className="text-[10px] text-zinc-500 font-mono mt-0.5">{i18n.t('workspaces.label_root_info')}</p>
                                    </div>
                                </div>

                                <div className="space-y-2 bg-zinc-950 p-3 rounded-xl border border-zinc-900">
                                    <div className="text-[8px] font-bold text-zinc-600 uppercase tracking-[0.2em] mb-1">{i18n.t('workspaces.header_environments')}</div>
                                    <div className="flex flex-wrap gap-2">
                                        <span className="flex items-center gap-1.5 px-2 py-1 rounded bg-blue-500/5 text-blue-400 border border-blue-500/10 text-[10px] font-mono"><Code2 size={10} /> VS_CODE</span>
                                        <span className="flex items-center gap-1.5 px-2 py-1 rounded bg-emerald-500/5 text-emerald-400 border border-emerald-500/10 text-[10px] font-mono"><Server size={10} /> K8S_NODE</span>
                                        <span className="flex items-center gap-1.5 px-2 py-1 rounded bg-amber-500/5 text-amber-400 border border-amber-500/10 text-[10px] font-mono"><Globe size={10} /> HEADLESS</span>
                                    </div>
                                </div>
                            </div>

                            {/* Pending Approvals (Option C: Task Branches) */}
                            <div className="bg-zinc-900 border border-zinc-800 p-5 rounded-2xl md:col-span-1 xl:col-span-2 flex flex-col gap-4 relative shadow-2xl">
                                <div className="flex items-center justify-between">
                                    <h3 className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                                        <Clock size={12} className="text-amber-500" />
                                        {i18n.t('workspaces.header_branches', { count: cluster.pending_tasks.filter(t => t.status === 'pending').length })}
                                    </h3>
                                </div>

                                <div className="flex-1 overflow-y-auto max-h-48 custom-scrollbar space-y-2">
                                    {(cluster.pending_tasks || []).length === 0 ? (
                                        <div className="h-full flex items-center justify-center text-zinc-700 text-[10px] uppercase font-bold tracking-widest italic animate-in fade-in">
                                            {i18n.t('workspaces.empty_branches')}
                                        </div>
                                    ) : (
                                        (cluster.pending_tasks || []).map((task) => (
                                            <div key={task.id} className={`flex items-center justify-between p-3 rounded-xl border transition-all ${task.status === 'pending' ? 'bg-zinc-900/50 border-zinc-800 group hover:border-zinc-700' :
                                                task.status === 'completed' ? 'bg-emerald-500/5 border-emerald-500/20 opacity-50' : 'bg-red-500/5 border-red-500/20 opacity-50'
                                                }`}>
                                                <div className="flex items-center gap-3">
                                                    <div className="p-2 bg-zinc-900 border border-zinc-800 rounded-lg">
                                                        <ArrowUpRight size={14} className={task.status === 'pending' ? 'text-amber-500' : 'text-zinc-600'} />
                                                    </div>
                                                    <div>
                                                        <p className="text-xs text-zinc-200 font-medium">{task.description}</p>
                                                        <div className="flex items-center gap-2 mt-1">
                                                            <span className="text-[9px] font-mono text-zinc-500 uppercase">{i18n.t('workspaces.label_from_agent', { id: task.agent_id })}</span>
                                                            <span className="text-zinc-800">•</span>
                                                            <span className="text-[9px] font-mono text-zinc-500 uppercase">{new Date(task.timestamp).toLocaleTimeString()}</span>
                                                        </div>
                                                    </div>
                                                </div>
                                                {task.status === 'pending' && (
                                                    <div className="flex items-center gap-2">
                                                        <Tooltip content={i18n.t('workspaces.tooltip_merge')} position="top">
                                                            <button onClick={() => approve_branch(cluster.id, task.id)} className="p-2 hover:bg-emerald-500/10 text-zinc-600 hover:text-emerald-500 transition-all rounded-lg">
                                                                <CheckCircle2 size={16} />
                                                            </button>
                                                        </Tooltip>
                                                        <Tooltip content={i18n.t('workspaces.tooltip_reject')} position="top">
                                                            <button onClick={() => reject_branch(cluster.id, task.id)} className="p-2 hover:bg-red-500/10 text-zinc-600 hover:text-red-500 transition-all rounded-lg">
                                                                <XCircle size={16} />
                                                            </button>
                                                        </Tooltip>
                                                    </div>
                                                )}
                                            </div>
                                        ))
                                    )}
                                </div>
                            </div>
                        </div>
                    </section>
                ))}

                {/* Empty State / Individual Silos */}
                <section className="pt-8 border-t border-zinc-900">
                    <h2 className="text-xs font-bold text-zinc-600 uppercase tracking-[0.3em] mb-4">{i18n.t('workspaces.header_legacy_silos')}</h2>
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-4 opacity-40 grayscale group hover:grayscale-0 hover:opacity-100 transition-all">
                        {agents.filter(a => !clusters.some(c => c.collaborators.includes(a.id))).map(agent => (
                            <div key={agent.id} className="p-3 bg-zinc-900/30 border border-zinc-800 rounded-xl flex items-center gap-3">
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

