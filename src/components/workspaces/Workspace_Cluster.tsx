/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Workspace_Cluster]` in observability traces.
 */

import React from 'react';
import { 
    Users, 
    Database, 
    Code2, 
    Server, 
    Globe, 
    Clock, 
    ArrowUpRight, 
    CheckCircle2, 
    XCircle 
} from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Agent } from '../../types';

interface WorkspacePendingTask {
    id: string;
    status: string;
    description: string;
    agent_id: string;
    timestamp: number | string;
}

interface WorkspaceClusterProps {
    cluster: {
        id: string;
        name: string;
        department: string;
        path: string;
        collaborators?: string[];
        alpha_id?: string;
        pending_tasks?: WorkspacePendingTask[];
    };
    agents: Agent[];
    sync_status: Record<string, {
        total_bytes: number;
        status: string;
        detected_environments?: string[];
        mounted_okf_nodes?: Array<{ id: string; title: string; concept_type: string; security_tier?: string; parent_id?: string }>;
        okf_validation?: {
            status: 'nominal' | 'warning' | 'critical';
            message?: string;
        };
    }>;
    format_bytes: (bytes: number) => string;
    on_approve: (cluster_id: string, task_id: string) => void;
    on_reject: (cluster_id: string, task_id: string) => void;
}

const ENV_META: Record<string, { label: string; bg: string; text: string; border: string }> = {
    vs_code: { label: 'VS Code', bg: 'bg-zinc-800/40', text: 'text-zinc-300', border: 'border-zinc-700/30' },
    k8s_node: { label: 'Kubernetes', bg: 'bg-zinc-800/40', text: 'text-zinc-300', border: 'border-zinc-700/30' },
    headless: { label: 'Headless', bg: 'bg-zinc-800/40', text: 'text-zinc-300', border: 'border-zinc-700/30' },
    docker: { label: 'Docker Sandbox', bg: 'bg-zinc-800/40', text: 'text-zinc-300', border: 'border-zinc-700/30' },
    wasm_sandbox: { label: 'WASM Isolation', bg: 'bg-zinc-800/40', text: 'text-zinc-300', border: 'border-zinc-700/30' },
    tauri_shell: { label: 'Tauri Shell', bg: 'bg-zinc-800/40', text: 'text-zinc-300', border: 'border-zinc-700/30' },
    jupyter_lab: { label: 'Jupyter Lab', bg: 'bg-zinc-800/40', text: 'text-zinc-300', border: 'border-zinc-700/30' }
};

const ENV_TOOLTIPS: Record<string, string> = {
    vs_code: 'workspaces.tooltip_env_vs_code',
    k8s_node: 'workspaces.tooltip_env_k8s_node',
    headless: 'workspaces.tooltip_env_headless',
    docker: 'workspaces.tooltip_env_docker',
    wasm_sandbox: 'workspaces.tooltip_env_wasm_sandbox',
    tauri_shell: 'workspaces.tooltip_env_tauri_shell',
    jupyter_lab: 'workspaces.tooltip_env_jupyter_lab'
};

const ENV_DEFAULT_TOOLTIPS: Record<string, string> = {
    vs_code: 'VS Code development environment detected',
    k8s_node: 'Kubernetes cluster deployment node detected',
    headless: 'Headless / CLI-only execution mode active',
    docker: 'Docker container virtualization sandbox running',
    wasm_sandbox: 'WebAssembly runner runtime isolation layer active',
    tauri_shell: 'Tauri native desktop shell wrapper active',
    jupyter_lab: 'Jupyter Lab interactive notebook workspace running'
};

export const Workspace_Cluster: React.FC<WorkspaceClusterProps> = ({
    cluster,
    agents,
    sync_status,
    format_bytes,
    on_approve,
    on_reject
}) => {
    const status = sync_status ? sync_status[cluster.path] : null;
    const detected_envs = status?.detected_environments || [];
    const okf_nodes = status?.mounted_okf_nodes || [];
    const okf_val = status?.okf_validation;

    return (
        <section className="space-y-6">
            <div className="flex items-center justify-between border-b border-[color:var(--color-surface)] pb-2">
                <div className="flex items-center gap-3">
                    <Tooltip content={i18n.t('workspaces.tooltip_dept')} position="right">
                        <div className={`p-1.5 rounded-lg border bg-[color:var(--color-surface)] cursor-help ${cluster.department === 'Executive' ? 'border-cyber-amber/30 text-cyber-amber' :
                            cluster.department === 'Engineering' ? 'border-cyber-green/30 text-cyber-green' : 'border-cyber-green/30 text-cyber-green'
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
                    {(cluster.collaborators || []).map((id: string) => {
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
                                        <div className="absolute -top-0.5 -right-0.5 w-2.5 h-2.5 rounded-full bg-cyber-amber border border-zinc-950 shadow-[0_0_8px_rgba(245,158,11,0.6)]" />
                                    )}
                                </div>
                            </Tooltip>
                        );
                    })}
                </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
                {/* Workspace Details Card */}
                <div className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] p-5 rounded-2xl group hover:border-zinc-700 transition-all flex flex-col gap-4 relative overflow-hidden shadow-2xl">
                    <div className="absolute top-0 right-0 p-3 opacity-10 group-hover:opacity-20 transition-opacity">
                        <Database size={48} />
                    </div>

                    <div className="flex items-center gap-3">
                        <Tooltip content={i18n.t('workspaces.tooltip_root')} position="top">
                            <div className="p-2.5 bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded-xl cursor-help">
                                <Database size={20} className="text-zinc-500" />
                            </div>
                        </Tooltip>
                        <div>
                            <h3 className="font-bold text-zinc-200 text-sm">{i18n.t('workspaces.label_root_title')}</h3>
                            <p className="text-[10px] text-zinc-500 font-mono mt-0.5">
                                {status 
                                    ? `${format_bytes(status.total_bytes)} ACTIVE • ${status.status.toUpperCase()}` 
                                    : i18n.t('workspaces.label_root_info')}
                            </p>
                        </div>
                    </div>

                    <div className="space-y-2 bg-[color:var(--color-background)] p-3 rounded-xl border border-[color:var(--color-surface)]">
                        <div className="text-[8px] font-bold text-zinc-600 uppercase tracking-[0.2em] mb-1">{i18n.t('workspaces.header_environments')}</div>
                        <div className="flex flex-wrap gap-2">
                            {detected_envs.length === 0 ? (
                                <span className="text-[10px] text-zinc-600 italic">{i18n.t('workspaces.no_environments_detected', { defaultValue: 'NONE DETECTED' })}</span>
                            ) : (
                                detected_envs.map(env => {
                                    const meta = ENV_META[env] || { label: env.toUpperCase(), bg: 'bg-zinc-800/40', text: 'text-zinc-300', border: 'border-zinc-700/30' };
                                    return (
                                        <Tooltip key={env} content={i18n.t(ENV_TOOLTIPS[env] || `workspaces.tooltip_env_${env}`, { defaultValue: ENV_DEFAULT_TOOLTIPS[env] || `${env.toUpperCase()} environment detected` })} position="top">
                                            <span className={`flex items-center gap-1.5 px-2 py-1 rounded ${meta.bg} ${meta.text} border ${meta.border} text-[10px] font-mono cursor-help`}>
                                                <Code2 size={10} />
                                                {meta.label}
                                            </span>
                                        </Tooltip>
                                    );
                                })
                            )}
                        </div>
                    </div>
                </div>

                {/* OKF Governance Card */}
                <div className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] p-5 rounded-2xl group hover:border-zinc-700 transition-all flex flex-col gap-4 relative overflow-hidden shadow-2xl">
                    <div className="absolute top-0 right-0 p-3 opacity-10 group-hover:opacity-20 transition-opacity">
                        <Server size={48} />
                    </div>

                    <div className="flex items-center gap-3">
                        <Tooltip content={i18n.t('workspaces.tooltip_okf', { defaultValue: 'Open Knowledge Format Safety constraints' })} position="top">
                            <div className={`p-2.5 rounded-xl border bg-[color:var(--color-surface)] cursor-help ${
                                okf_val?.status === 'critical' ? 'border-cyber-red/30 text-cyber-red bg-cyber-red/5' :
                                okf_val?.status === 'warning' ? 'border-cyber-amber/30 text-cyber-amber bg-cyber-amber/5' :
                                'border-cyber-green/30 text-cyber-green bg-cyber-green/5'
                            }`}>
                                <Globe size={20} />
                            </div>
                        </Tooltip>
                        <div>
                            <h3 className="font-bold text-zinc-200 text-sm">{i18n.t('workspaces.label_okf_title', { defaultValue: 'OKF GOVERNANCE' })}</h3>
                            <p className="text-[10px] font-mono mt-0.5 uppercase tracking-wider">
                                {okf_val?.status === 'critical' ? (
                                    <span className="text-cyber-red font-bold">CRITICAL SEC VIOLATION</span>
                                ) : okf_val?.status === 'warning' ? (
                                    <span className="text-cyber-amber font-bold">WARNING CONSTRAINTS</span>
                                ) : (
                                    <span className="text-cyber-green">NOMINAL STATE</span>
                                )}
                            </p>
                        </div>
                    </div>

                    {/* Safety Alert Banner */}
                    {okf_val?.message && (
                        <div className={`p-3 rounded-xl border text-[11px] font-medium leading-relaxed ${
                            okf_val.status === 'critical' ? 'bg-cyber-red/5 border-cyber-red/20 text-cyber-red font-semibold' :
                            'bg-cyber-amber/5 border-cyber-amber/20 text-cyber-amber'
                        }`}>
                            {okf_val.message}
                        </div>
                    )}

                    {/* Mounted Playbooks Badges */}
                    <div className="space-y-2 bg-[color:var(--color-background)] p-3 rounded-xl border border-[color:var(--color-surface)] flex-1 flex flex-col justify-between">
                        <div>
                            <div className="text-[8px] font-bold text-zinc-600 uppercase tracking-[0.2em] mb-2">{i18n.t('workspaces.header_mounted_playbooks', { defaultValue: 'MOUNTED PLAYBOOKS' })}</div>
                            <div className="flex flex-wrap gap-1.5 max-h-[110px] overflow-y-auto custom-scrollbar">
                                {okf_nodes.length === 0 ? (
                                    <span className="text-[10px] text-zinc-500 italic py-1">{i18n.t('workspaces.no_playbooks_mounted', { defaultValue: 'No playbooks mounted' })}</span>
                                ) : (
                                     okf_nodes.map(node => (
                                        <Tooltip key={node.id} content={i18n.t('workspaces.tooltip_playbook_link', { title: node.title, defaultValue: `Inspect ${node.title} relationships in the Semantic OKF Graph` })} position="top">
                                            <a
                                                href={`/knowledge?node=${node.id}`}
                                                className="px-2 py-0.5 rounded-full bg-zinc-850 hover:bg-zinc-750 text-zinc-300 hover:text-zinc-100 border border-zinc-700/50 hover:border-zinc-500 text-[9px] font-mono flex items-center gap-1 transition-all cursor-pointer"
                                            >
                                                <Code2 size={8} className="text-zinc-500" />
                                                {node.title}
                                                {node.security_tier && (
                                                    <span className={`px-1 py-0.2 rounded text-[7px] font-bold uppercase ${
                                                        node.security_tier.includes('GOLD') ? 'bg-amber-500/20 text-amber-300 border border-amber-500/30' :
                                                        node.security_tier.includes('SILVER') ? 'bg-zinc-400/20 text-zinc-200 border border-zinc-400/30' :
                                                        'bg-zinc-700/20 text-zinc-400'
                                                    }`}>
                                                        {node.security_tier.includes('GOLD') ? 'GOLD' : node.security_tier.includes('SILVER') ? 'SILVER' : 'BRONZE'}
                                                    </span>
                                                )}
                                            </a>
                                        </Tooltip>
                                    ))
                                )}
                            </div>
                        </div>
                    </div>
                </div>

                {/* Pending Approvals */}
                <div className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] p-5 rounded-2xl md:col-span-1 xl:col-span-1 flex flex-col gap-4 relative shadow-2xl">
                    <div className="flex items-center justify-between">
                        <h3 className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                            <Clock size={12} className="text-amber-500" />
                            {i18n.t('workspaces.header_branches', { count: (cluster.pending_tasks || []).filter((t: WorkspacePendingTask) => t.status === 'pending').length })}
                        </h3>
                    </div>

                    <div className="flex-1 overflow-y-auto max-h-48 custom-scrollbar space-y-2">
                        {(cluster.pending_tasks || []).length === 0 ? (
                            <div className="h-full flex items-center justify-center text-zinc-700 text-[10px] uppercase font-bold tracking-widest italic animate-in fade-in">
                                {i18n.t('workspaces.empty_branches')}
                            </div>
                        ) : (
                            (cluster.pending_tasks || []).map((task: WorkspacePendingTask) => (
                                <div key={task.id} className={`flex items-center justify-between p-3 rounded-xl border transition-all ${task.status === 'pending' ? 'bg-[color:var(--color-surface)]/50 border-[color:var(--color-border)] group hover:border-zinc-700' :
                                    task.status === 'completed' ? 'bg-emerald-500/5 border-emerald-500/20 opacity-50' : 'bg-red-500/5 border-red-500/20 opacity-50'
                                    }`}>
                                    <div className="flex items-center gap-3">
                                        <div className="p-2 bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded-lg">
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
                                                <button onClick={() => on_approve(cluster.id, task.id)} className="p-2 hover:bg-emerald-500/10 text-zinc-600 hover:text-emerald-500 transition-all rounded-lg">
                                                    <CheckCircle2 size={16} />
                                                </button>
                                            </Tooltip>
                                            <Tooltip content={i18n.t('workspaces.tooltip_reject')} position="top">
                                                <button onClick={() => on_reject(cluster.id, task.id)} className="p-2 hover:bg-red-500/10 text-zinc-600 hover:text-red-500 transition-all rounded-lg">
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
    );
};

// Metadata: [Workspace_Cluster]
