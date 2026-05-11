/**
 * @docs ARCHITECTURE:Components
 * 
 * ### AI Assist Note
 * **Core functional element for the Tadpole OS engine.**
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path: Runtime logic error or state corruption.**
 * - **Telemetry Link**: Search `[Hardware_Load.tsx]` in tracing logs.
 */

import { useState, useEffect } from 'react';
import { Cpu, HardDrive, Activity, Server } from 'lucide-react';
import { api_request } from '../../services/base_api_service';

interface ComputeProfile {
    cpu_usage: number;
    memory_used: number;
    memory_total: number;
    active_processes: number;
    gpu_usage: number | null;
}

export function Hardware_Load() {
    const [profile, setProfile] = useState<ComputeProfile | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const controller = new AbortController();
        let is_mounted = true;

        const fetch_profile = async () => {
            try {
                const data = await api_request<ComputeProfile>('/v1/system/compute-profile', {
                    signal: controller.signal
                });
                
                if (is_mounted) {
                    setProfile(data);
                    setError(null);
                }
            } catch (e: unknown) {
                if (is_mounted && !(e instanceof Error && e.name === 'AbortError')) {
                    setError(e instanceof Error ? e.message : 'An unknown error occurred');
                }
            }
        };

        fetch_profile();
        const interval = setInterval(fetch_profile, 5000);
        
        return () => {
            is_mounted = false;
            controller.abort();
            clearInterval(interval);
        };
    }, []);

    if (error) {
        return (
            <div className="p-4 border border-red-500/20 bg-red-500/5 rounded-2xl">
                <p className="text-red-400 text-sm font-mono">{error}</p>
            </div>
        );
    }

    if (!profile) {
        return (
            <div className="p-5 border border-zinc-800 rounded-2xl bg-zinc-900/50 backdrop-blur-xl animate-pulse">
                <div className="h-4 bg-zinc-800 rounded w-1/3 mb-4"></div>
                <div className="space-y-3">
                    <div className="h-8 bg-zinc-800 rounded w-full"></div>
                    <div className="h-8 bg-zinc-800 rounded w-full"></div>
                </div>
            </div>
        );
    }

    const memory_gb = profile.memory_used / (1024 * 1024 * 1024);
    const total_gb = profile.memory_total / (1024 * 1024 * 1024);
    const mem_percent = (profile.memory_used / profile.memory_total) * 100;

    return (
        <div className="p-6 border border-zinc-800 rounded-3xl bg-zinc-950 shadow-2xl relative overflow-hidden group">
            <div className="absolute inset-0 bg-gradient-to-br from-emerald-500/5 to-transparent pointer-events-none" />
            
            <div className="flex items-center justify-between mb-6 relative z-10">
                <div className="flex items-center gap-3">
                    <div className="p-2 bg-emerald-500/10 rounded-lg border border-emerald-500/20 text-emerald-400">
                        <Server size={18} />
                    </div>
                    <h3 className="text-sm font-bold text-zinc-100 uppercase tracking-widest">Hardware Load</h3>
                </div>
                <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
                    <span className="text-[10px] font-mono text-zinc-500 uppercase tracking-widest">Live</span>
                </div>
            </div>

            <div className="space-y-6 relative z-10">
                {/* CPU */}
                <div>
                    <div className="flex justify-between items-end mb-2">
                        <div className="flex items-center gap-2 text-zinc-400">
                            <Cpu size={14} />
                            <span className="text-[10px] font-bold uppercase tracking-widest">CPU Usage</span>
                        </div>
                        <span className="text-lg font-mono text-zinc-100">{profile.cpu_usage.toFixed(1)}%</span>
                    </div>
                    <div className="h-1.5 w-full bg-zinc-900 rounded-full overflow-hidden">
                        <div 
                            className={`h-full transition-all duration-500 ${profile.cpu_usage > 80 ? 'bg-red-500' : profile.cpu_usage > 50 ? 'bg-yellow-500' : 'bg-emerald-500'}`}
                            style={{ width: `${Math.min(100, Math.max(0, profile.cpu_usage))}%` }}
                        />
                    </div>
                </div>

                {/* Memory */}
                <div>
                    <div className="flex justify-between items-end mb-2">
                        <div className="flex items-center gap-2 text-zinc-400">
                            <HardDrive size={14} />
                            <span className="text-[10px] font-bold uppercase tracking-widest">RAM Allocation</span>
                        </div>
                        <div className="text-right">
                            <span className="text-lg font-mono text-zinc-100">{memory_gb.toFixed(1)}</span>
                            <span className="text-[10px] font-mono text-zinc-500 ml-1">/ {total_gb.toFixed(1)} GB</span>
                        </div>
                    </div>
                    <div className="h-1.5 w-full bg-zinc-900 rounded-full overflow-hidden">
                        <div 
                            className={`h-full transition-all duration-500 ${mem_percent > 85 ? 'bg-red-500' : mem_percent > 60 ? 'bg-yellow-500' : 'bg-emerald-500'}`}
                            style={{ width: `${Math.min(100, Math.max(0, mem_percent))}%` }}
                        />
                    </div>
                </div>

                {/* Processes */}
                <div className="pt-4 border-t border-zinc-800/50 flex justify-between items-center">
                    <div className="flex items-center gap-2 text-zinc-400">
                        <Activity size={14} />
                        <span className="text-[10px] font-bold uppercase tracking-widest">Active Processes</span>
                    </div>
                    <span className="text-sm font-mono text-zinc-300">{profile.active_processes}</span>
                </div>
            </div>
        </div>
    );
}

// Metadata: [Hardware_Load]

// Metadata: [Hardware_Load]
