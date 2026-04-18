/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Advanced navigation router for the Dashboard Sub-Sectors. 
 * Manages transitions between Hive (Agents), Ledger (Oversight), and Terminal (Missions) with active-state layout indicators.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Route mismatch during direct-link hydration, or tab indicator "stretching" during navigation transitions.
 * - **Telemetry Link**: Search for `[Operation_Tab_Header]` or `set_active_tab` in UI tracing.
 */

import React from 'react';
import { motion } from 'framer-motion';
import { 
    Globe, 
    Layers, 
    Power, 
    ExternalLink,
    Activity
} from 'lucide-react';
import { i18n } from '../../i18n';
import type { Mission_Cluster } from '../../types';
import { Tooltip } from '../ui';

interface Operation_Tab_Header_Props {
    active_tab_id: string;
    clusters: Mission_Cluster[];
    on_tab_change: (id: string) => void;
    on_toggle_cluster: (cluster_id: string) => void;
    on_detach_tab: (id: string) => void;
    on_detach_grid?: () => void;
}

export const Operation_Tab_Header: React.FC<Operation_Tab_Header_Props> = ({
    active_tab_id,
    clusters,
    on_tab_change,
    on_toggle_cluster,
    on_detach_tab,
    on_detach_grid
}) => {
    const render_tab = (id: string, label: string, icon: React.ReactNode, is_cluster = false, is_active_cluster = false) => {
        const is_selected = active_tab_id === id;

        return (
            <div 
                key={id}
                className={`flex-shrink-0 flex items-center gap-2 px-4 h-12 relative cursor-pointer transition-all duration-300 group
                    ${is_selected ? 'text-blue-400' : 'text-zinc-500 hover:text-zinc-300'}
                `}
                onClick={() => on_tab_change(id)}
            >
                {/* Active Indicator Bar */}
                {is_selected && (
                    <motion.div 
                        layoutId="activeTab"
                        className="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.6)] z-10"
                    />
                )}

                <div className="flex items-center gap-3">
                    <div className="flex items-center gap-2">
                        {icon}
                        <span className="text-[10px] uppercase font-bold tracking-widest whitespace-nowrap">
                            {label}
                        </span>
                    </div>
                    
                    <div className="flex items-center gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity">
                         <Tooltip content={i18n.t('layout.pop_out_sector') || 'Pop Out Sector'}>
                            <button
                                onClick={(e) => {
                                    e.stopPropagation();
                                    on_detach_tab(id);
                                }}
                                className="p-1 text-zinc-600 hover:text-blue-400 hover:bg-blue-500/10 rounded transition-all"
                            >
                                <ExternalLink size={10} />
                            </button>
                        </Tooltip>

                        {is_cluster && (
                            <div className="flex items-center gap-2 ml-1 pl-1 border-l border-white/5">
                                <Tooltip content={is_active_cluster ? (i18n.t('layout.deactivate_mission') || 'Deactivate Mission') : (i18n.t('layout.activate_mission') || 'Activate Mission')}>
                                    <button
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            on_toggle_cluster(id);
                                        }}
                                        className={`p-1 rounded-md transition-all duration-300 
                                            ${is_active_cluster 
                                                ? 'text-emerald-500 bg-emerald-500/10 hover:bg-emerald-500/20 shadow-[0_0_10px_rgba(16,185,129,0.2)]' 
                                                : 'text-zinc-600 hover:text-zinc-400 hover:bg-white/5'
                                            }
                                        `}
                                    >
                                        <Power size={10} className={is_active_cluster ? 'animate-pulse' : ''} />
                                    </button>
                                </Tooltip>
                                
                                {/* Small Status Dot */}
                                <div className={`w-1 h-1 rounded-full ${is_active_cluster ? 'bg-emerald-500 animate-pulse shadow-[0_0_5px_rgba(16,185,129,0.5)]' : 'bg-zinc-700'}`} />
                            </div>
                        )}
                    </div>
                </div>
            </div>
        );
    };

    return (
        <div className="sticky top-0 bg-zinc-950/80 backdrop-blur-md z-30 border-b border-zinc-800/50 flex items-center justify-between px-2">
            <div className="flex-1 flex items-center overflow-x-auto no-scrollbar min-w-0">
                {/* Global View Tab */}
                {render_tab('global', i18n.t('dashboard.global_view') || 'Global View', <Globe size={12} className="group-hover:rotate-12 transition-transform" />)}
 
                <div className="h-4 w-px bg-white/5 mx-2" />

                {/* Dynamic Cluster Tabs */}
                {(clusters || []).map(cluster => render_tab(
                    cluster.id, 
                    cluster.name, 
                    <Layers size={12} />, 
                    true, 
                    cluster.is_active
                ))}
            </div>

            {/* Actions Sidebar in Header */}
            <div className="flex items-center gap-4 pr-4">
                {on_detach_grid && (
                    <Tooltip content={i18n.t('layout.detach_grid') || 'Detach Agent Grid'}>
                        <button
                            onClick={(e) => {
                                e.stopPropagation();
                                on_detach_grid();
                            }}
                            className="p-1.5 text-zinc-500 hover:text-blue-400 hover:bg-blue-500/10 rounded-lg transition-all border border-transparent hover:border-blue-500/20 shadow-lg active:scale-90"
                        >
                            <ExternalLink size={14} />
                        </button>
                    </Tooltip>
                )}

                 <div className="flex items-center gap-2 px-3 py-1 bg-blue-500/5 border border-blue-500/10 rounded-full">
                    <Activity size={10} className="text-blue-500 animate-pulse" />
                    <span className="text-[9px] font-mono text-blue-400/80 uppercase tracking-tighter">{i18n.t('layout.live_status')}</span>
                </div>
            </div>
        </div>
    );
};

