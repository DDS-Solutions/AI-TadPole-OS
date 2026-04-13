/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Low-level system orchestration dashboard. 
 * Orchestrates the visualization of internal engine states, service health, and middleware performance metrics.
 * 
 * ### 🔍 Debugging & Observability
 */

import React from 'react';
import { Activity, ShieldCheck, Zap, HardDrive, Cpu as CpuIcon, ExternalLink } from 'lucide-react';
import { useEngineStatus } from '../hooks/use_engine_status';
import { SectorBoundary } from '../components/Error_Boundary';
import { Swarm_Telemetry } from '../components/Swarm_Telemetry';
import { Swarm_Visualizer } from '../components/Swarm_Visualizer';
import { use_tab_store } from '../stores/tab_store';
import { Portal_Window } from '../components/ui';
import { use_settings_store } from '../stores/settings_store';
import { Tooltip } from '../components/ui';
import { i18n } from '../i18n';
import { Telemetry_Graph } from '../components/Telemetry_Graph';

/**
 * Real-time monitoring center for the Tadpole Engine.
 * Visualizes high-frequency system telemetry via Neural Pulse event streams.
 */
export default function Engine_Dashboard() {
    const { 
        is_online, 
        cpu, 
        memory, 
        latency, 
        connection_state, 
        active_agents, 
        max_depth, 
        tpm, 
        recruit_count 
    } = useEngineStatus();
    const { settings } = use_settings_store();
    const { is_swarm_pulse_detached, toggle_swarm_pulse_detachment } = use_tab_store();

    const stats = [
        { 
            label: i18n.t('engine_dashboard.label_cpu'), 
            value: `${cpu.toFixed(1)}%`, 
            icon: CpuIcon, 
            color: cpu > 80 ? 'text-red-400' : 'text-emerald-400' 
        },
        { 
            label: i18n.t('engine_dashboard.label_memory'), 
            value: `${memory.toFixed(1)}GB`, 
            icon: HardDrive, 
            color: 'text-emerald-400' 
        },
        { 
            label: i18n.t('engine_dashboard.label_latency'), 
            value: `${latency.toFixed(0)}ms`, 
            icon: Zap, 
            color: latency > 150 ? 'text-yellow-400' : 'text-emerald-400' 
        },
    ];

    return (
        <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-700">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                    <Tooltip content={i18n.t('engine_dashboard.tooltip_main')} position="right">
                        <div className="p-2 bg-zinc-900 border border-zinc-800 rounded-lg shadow-xl ring-1 ring-zinc-800 cursor-help">
                            <Activity className="w-5 h-5 text-emerald-500 animate-pulse" />
                        </div>
                    </Tooltip>
                    <div>
                        <h2 className="text-2xl font-bold tracking-tight text-zinc-100 italic">{i18n.t('engine_dashboard.title')}</h2>
                        <p className="text-zinc-500 text-[10px] uppercase tracking-[0.2em] font-bold font-mono">{i18n.t('engine_dashboard.subtitle')}</p>
                    </div>
                </div>
                <Tooltip content={i18n.t('engine_dashboard.tooltip_conn', { status: connection_state })} position="left">
                    <div className="flex items-center gap-2 px-3 py-1 bg-zinc-950 border border-zinc-800 rounded-full cursor-help">
                        <div className={`w-2 h-2 rounded-full ${is_online ? 'bg-emerald-500 animate-pulse' : 'bg-red-500'}`} />
                        <span className="text-[10px] font-bold font-mono text-zinc-400 uppercase tracking-widest">{connection_state}</span>
                    </div>
                </Tooltip>
            </div>

            <Swarm_Telemetry
                active_agents={active_agents}
                max_depth={max_depth}
                tpm={tpm}
                recruit_count={recruit_count}
                max_density={settings.max_agents}
            />


            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {stats.map((stat) => (
                    <Tooltip key={stat.label} content={
                        stat.label === i18n.t('engine_dashboard.label_cpu') ? i18n.t('engine_dashboard.tooltip_cpu') :
                            stat.label === i18n.t('engine_dashboard.label_memory') ? i18n.t('engine_dashboard.tooltip_memory') :
                                i18n.t('engine_dashboard.tooltip_latency')
                    } position="top">
                        <div className="p-5 border border-zinc-800 rounded-2xl bg-zinc-900/50 backdrop-blur-xl group hover:border-zinc-700 transition-all shadow-lg cursor-help">
                            <div className="flex justify-between items-start mb-3">
                                <div className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest">{stat.label}</div>
                                <stat.icon size={14} className="text-zinc-700 group-hover:text-zinc-500 transition-colors" />
                            </div>
                            <div className={`text-3xl font-mono ${stat.color} tracking-tighter`}>{stat.value}</div>
                            <div className="mt-3 flex items-center gap-1.5">
                                <div className={`h-1 w-full rounded-full bg-zinc-800 overflow-hidden`}>
                                    <div
                                        className={`h-full bg-current transition-all duration-100 ${stat.color}`}
                                        style={{ width: stat.label === i18n.t('engine_dashboard.label_cpu') ? `${cpu}%` : stat.label === i18n.t('engine_dashboard.label_memory') ? `${(memory / 16) * 100}%` : '50%' }}
                                    />
                                </div>
                            </div>
                        </div>
                    </Tooltip>
                ))}
            </div>

            <div className="border border-zinc-800 rounded-3xl bg-zinc-950 relative overflow-hidden group shadow-2xl h-[500px]">
                <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_120%,rgba(16,185,129,0.05),transparent)] pointer-events-none" />
                <div className="relative z-10 h-full flex flex-col items-center p-4">
                    <div className="w-full flex justify-between items-center mb-4">
                        <div className="flex items-center gap-2 text-zinc-600 font-mono text-[10px] uppercase tracking-[0.3em] font-bold">
                            <ShieldCheck size={12} className="text-emerald-500/50" /> {i18n.t('engine_dashboard.label_websocket')}
                        </div>
                        <div className="text-center">
                            <p className="text-zinc-100 text-[10px] font-bold tracking-tighter flex items-center justify-center gap-2 uppercase">
                                <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-ping" />
                                {i18n.t('engine_dashboard.label_status')} {is_online ? 'Active' : 'Offline'}
                            </p>
                        </div>
                    </div>
                    
                    <div className="w-full flex-grow rounded-2xl overflow-hidden border border-zinc-900 shadow-inner">
                        <SectorBoundary name="Swarm Execution Graph">
                            <Telemetry_Graph />
                        </SectorBoundary>
                    </div>
                    
                    <div className="mt-4 text-zinc-600 text-[9px] font-mono uppercase tracking-[0.2em]">
                        {i18n.t('engine_dashboard.label_node', { stream: is_online ? i18n.t('engine_dashboard.stream_active') : i18n.t('engine_dashboard.stream_offline') })}
                    </div>
                </div>
            </div>

            <div className="h-[400px] w-full relative">
                {is_swarm_pulse_detached ? (
                    <React.Fragment key="swarm-pulse-detached-state">
                        <div className="absolute inset-0 flex items-center justify-center bg-zinc-900/20 backdrop-blur-sm border border-zinc-800 rounded-[2.5rem] overflow-hidden group">
                            <div className="text-center space-y-4">
                                <div className="relative inline-block">
                                    <ExternalLink size={40} className="text-zinc-800 animate-pulse" />
                                    <div className="absolute inset-0 bg-cyan-500/10 blur-xl rounded-full" />
                                </div>
                                <div className="space-y-1">
                                    <h3 className="text-sm font-bold tracking-tight text-zinc-300 uppercase tracking-[0.2em]">{i18n.t('layout.sector_detached')}</h3>
                                    <p className="text-[10px] text-zinc-500 font-mono uppercase tracking-widest">{i18n.t('layout.link_established')} :: SWARM_PULSE_DETACHED</p>
                                </div>
                                <button 
                                    onClick={toggle_swarm_pulse_detachment}
                                    className="px-6 py-2.5 bg-zinc-100 text-black text-[10px] font-black uppercase tracking-[0.2em] rounded-xl hover:bg-white transition-all shadow-xl active:scale-95"
                                >
                                    Recall Swarm Pulse
                                </button>
                            </div>
                        </div>

                        <Portal_Window 
                            id="swarm-pulse-detached" 
                            title="Swarm Pulse" 
                            on_close={toggle_swarm_pulse_detachment}
                        >
                            <div className="h-screen bg-zinc-950 p-6 flex flex-col overflow-hidden">
                                <Swarm_Visualizer is_detached={true} on_detach={toggle_swarm_pulse_detachment} />
                            </div>
                        </Portal_Window>
                    </React.Fragment>
                ) : (
                    <Swarm_Visualizer 
                        key="swarm-pulse-inline-state" 
                        on_detach={toggle_swarm_pulse_detachment} 
                    />
                )}
            </div>
        </div>
    );
}
