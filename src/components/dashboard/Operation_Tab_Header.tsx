/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Advanced navigation router for the Dashboard Sub-Sectors. 
 * Manages transitions between Hive (Agents), Ledger (Oversight), and Terminal (Missions) with active-state layout indicators.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Route mismatch during direct-link hydration, or tab indicator "stretching" during navigation transitions.
 * - **Telemetry Link**: Search for `[Operation_Tab_Header]` or set_active_tab in UI tracing.
 */

import React, { useRef, useState, useCallback } from 'react';
import { motion } from 'framer-motion';
import { 
    Globe, 
    Layers, 
    Power, 
    ExternalLink,
    Activity,
    ArrowUpDown
} from 'lucide-react';
import { i18n } from '../../i18n';
import type { Mission_Cluster } from '../../types';
import { Tooltip } from '../ui';

export type OperationTabId = 'global' | 'ledger' | 'terminal' | 'hive' | string;
export type SortMode = 'alpha' | 'active_first' | 'inactive_first';

interface Operation_Tab_Header_Props {
    active_tab_id: OperationTabId;
    clusters: Mission_Cluster[];
    on_tab_change: (id: OperationTabId) => void;
    on_toggle_cluster: (cluster_id: string) => void;
    on_detach_tab: (id: OperationTabId) => void;
    on_detach_grid?: () => void;
    sort_mode?: SortMode;
    on_cycle_sort_mode?: () => void;
}

export const Operation_Tab_Header: React.FC<Operation_Tab_Header_Props> = ({
    active_tab_id,
    clusters,
    on_tab_change,
    on_toggle_cluster,
    on_detach_tab,
    on_detach_grid,
    sort_mode = 'alpha',
    on_cycle_sort_mode
}) => {
    const scrollRef = useRef<HTMLDivElement>(null);
    const [isDown, setIsDown] = useState(false);
    const [startX, setStartX] = useState(0);
    const [scrollLeftState, setScrollLeftState] = useState(0);
    const hasMovedRef = useRef(false);

    const handleMouseDown = (e: React.MouseEvent) => {
        if (!scrollRef.current) return;
        setIsDown(true);
        hasMovedRef.current = false;
        setStartX(e.pageX - scrollRef.current.offsetLeft);
        setScrollLeftState(scrollRef.current.scrollLeft);
    };

    const handleMouseLeave = () => {
        setIsDown(false);
    };

    const handleMouseUp = () => {
        setIsDown(false);
    };

    const handleMouseMove = (e: React.MouseEvent) => {
        if (!isDown || !scrollRef.current) return;
        const x = e.pageX - scrollRef.current.offsetLeft;
        const walk = (x - startX) * 2; // scroll speed multiplier
        if (Math.abs(walk) > 3) {
            hasMovedRef.current = true;
        }
        scrollRef.current.scrollLeft = scrollLeftState - walk;
    };

    const get_sort_tooltip = useCallback((mode: SortMode) => {
        const mode_label = mode === 'active_first' 
            ? i18n.t('dashboard.sort_active_first') 
            : mode === 'inactive_first' 
                ? i18n.t('dashboard.sort_inactive_first') 
                : i18n.t('dashboard.sort_alpha');
        return i18n.t('dashboard.sort_tooltip', { mode: mode_label });
    }, []);

    const render_tab = (id: string, label: string, icon: React.ReactNode, is_cluster = false, is_active_cluster = false) => {
        const is_selected = active_tab_id === id;

        return (
            <div 
                key={id}
                className={`flex-shrink-0 flex items-center gap-2 px-4 h-12 relative cursor-pointer transition-all duration-300 group
                    ${is_selected ? 'text-green-400' : 'text-zinc-500 hover:text-zinc-300'}
                `}
                onClick={() => {
                    if (!hasMovedRef.current) {
                        on_tab_change(id);
                    }
                }}
            >
                {/* Active Indicator Bar */}
                {is_selected && (
                    <motion.div 
                        layoutId="activeTab"
                        className="absolute bottom-0 left-0 right-0 h-0.5 bg-green-500 shadow-[0_0_8px_rgba(59,130,246,0.6)] z-10"
                    />
                )}

                <div className="flex items-center gap-3">
                    <div className="flex items-center gap-2">
                        {icon}
                        <span className="text-[10px] uppercase font-bold tracking-widest whitespace-nowrap">
                            {label}
                        </span>
                    </div>
                    
                    <div className={`flex items-center gap-1.5 transition-opacity ${is_selected ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'}`}>
                         <Tooltip content={i18n.t('layout.pop_out_sector')}>
                            <button
                                onClick={(e) => {
                                    e.stopPropagation();
                                    on_detach_tab(id);
                                }}
                                className="p-1 text-zinc-600 hover:text-green-400 hover:bg-green-500/10 rounded transition-all"
                                aria-label={i18n.t('layout.pop_out_sector')}
                            >
                                <ExternalLink size={10} />
                            </button>
                        </Tooltip>

                        {on_cycle_sort_mode && (
                            <Tooltip content={get_sort_tooltip(sort_mode)}>
                                <button
                                    type="button"
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        if (!is_selected) {
                                            on_tab_change(id);
                                        }
                                        on_cycle_sort_mode();
                                    }}
                                    aria-label={get_sort_tooltip(sort_mode)}
                                    className={`p-1 rounded transition-all flex items-center gap-0.5 ${
                                        is_selected && sort_mode !== 'alpha'
                                            ? 'text-emerald-400 bg-emerald-500/15 border border-emerald-500/30'
                                            : 'text-zinc-600 hover:text-green-400 hover:bg-green-500/10'
                                    }`}
                                >
                                    <ArrowUpDown size={10} />
                                    {is_selected && sort_mode === 'active_first' && (
                                        <span className="text-[7px] font-mono font-extrabold uppercase text-emerald-400 tracking-tighter">ACT</span>
                                    )}
                                    {is_selected && sort_mode === 'inactive_first' && (
                                        <span className="text-[7px] font-mono font-extrabold uppercase text-amber-400 tracking-tighter">INA</span>
                                    )}
                                </button>
                            </Tooltip>
                        )}

                        {is_cluster && (
                            <div className="flex items-center gap-2 ml-1 pl-1 border-l border-white/5">
                                <Tooltip content={is_active_cluster ? i18n.t('layout.deactivate_mission') : i18n.t('layout.activate_mission')}>
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
        <div className="sticky top-0 bg-[color:color-mix(in_srgb,var(--color-background)_80%,transparent)] backdrop-blur-md z-30 border-b border-[color:var(--color-border)]/50 flex items-center justify-between px-2 select-none">
            <div className="flex-1 flex items-center min-w-0">
                {/* Global View Tab */}
                {render_tab('global', i18n.t('dashboard.global_view'), <Globe size={12} className="group-hover:rotate-12 transition-transform" />)}
 
                <div className="h-4 w-px bg-white/5 mx-2 shrink-0" />

                {/* Scrollable Container for Dynamic Cluster Tabs */}
                <div 
                    ref={scrollRef}
                    onMouseDown={handleMouseDown}
                    onMouseLeave={handleMouseLeave}
                    onMouseUp={handleMouseUp}
                    onMouseMove={handleMouseMove}
                    className="flex-1 flex items-center overflow-x-auto no-scrollbar cursor-grab active:cursor-grabbing select-none"
                >
                    {(clusters || []).map(cluster => render_tab(
                        cluster.id, 
                        cluster.name, 
                        <Layers size={12} />, 
                        true, 
                        cluster.is_active
                    ))}
                </div>
            </div>

            {/* Actions Sidebar in Header */}
            <div className="flex items-center gap-2.5 pr-4">
                {on_cycle_sort_mode && (
                    <Tooltip content={get_sort_tooltip(sort_mode)}>
                        <button
                            type="button"
                            onClick={(e) => {
                                e.stopPropagation();
                                on_cycle_sort_mode();
                            }}
                            aria-label={get_sort_tooltip(sort_mode)}
                            className={`p-1.5 rounded-lg transition-all border flex items-center gap-1 shadow-lg active:scale-95 ${
                                sort_mode !== 'alpha'
                                    ? 'text-emerald-400 bg-emerald-500/15 border-emerald-500/30'
                                    : 'text-zinc-400 hover:text-green-400 hover:bg-green-500/10 border-zinc-800 hover:border-green-500/20'
                            }`}
                        >
                            <ArrowUpDown size={14} />
                            {sort_mode === 'active_first' && (
                                <span className="text-[9px] font-mono font-bold text-emerald-400">ACT</span>
                            )}
                            {sort_mode === 'inactive_first' && (
                                <span className="text-[9px] font-mono font-bold text-amber-400">INA</span>
                            )}
                        </button>
                    </Tooltip>
                )}

                {on_detach_grid && (
                    <Tooltip content={i18n.t('layout.detach_grid')}>
                        <button
                            onClick={(e) => {
                                e.stopPropagation();
                                on_detach_grid();
                            }}
                            className="p-1.5 text-zinc-500 hover:text-green-400 hover:bg-green-500/10 rounded-lg transition-all border border-transparent hover:border-green-500/20 shadow-lg active:scale-90"
                        >
                            <ExternalLink size={14} />
                        </button>
                    </Tooltip>
                )}

                 <div className="flex items-center gap-2 px-3 py-1 bg-green-500/5 border border-green-500/10 rounded-full">
                    <Activity size={10} className="text-green-500 animate-pulse" />
                    <span className="text-[9px] font-mono text-green-400/80 uppercase tracking-tighter">{i18n.t('layout.live_status')}</span>
                </div>
            </div>
        </div>
    );
};


// Metadata: [Operation_Tab_Header]
