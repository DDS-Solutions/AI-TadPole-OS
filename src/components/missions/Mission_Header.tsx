/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Action-oriented mission command strip. 
 * Orchestrates "Power-On" (Mission Execution) signaling, AI Security analysis toggles, and department-themed header aesthetics.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: `on_run_mission` trigger while `agents_loading` is true (should be disabled), theme color desync between header and body, or SVG icon render fault.
 * - **Telemetry Link**: Search for `[Mission_Header]` or `on_run_mission` in browser logs.
 */

import React from 'react';
import { Zap, Pause, Play, RefreshCw, OctagonX, ShieldCheck, Copy } from 'lucide-react';
import { Tooltip } from '../ui';
import { get_department_icon, get_theme_colors } from '../../utils/agent_uiutils';
import type { Mission_Cluster } from '../../stores/workspace_store';
import { i18n } from '../../i18n';

interface MissionHeaderProps {
    active_cluster: Mission_Cluster;
    agents_loading: boolean;
    has_agents: boolean;
    on_run_mission: () => void;
    on_pause_resume_mission: () => void;
    on_cancel_mission: () => void;
    on_clone_mission?: () => void;
    on_toggle_analysis: (id: string) => void;
    is_launching?: boolean;
    has_halted_agents?: boolean;
}

export const Mission_Header: React.FC<MissionHeaderProps> = ({
    active_cluster,
    agents_loading,
    has_agents,
    on_run_mission,
    on_pause_resume_mission,
    on_cancel_mission,
    on_clone_mission,
    on_toggle_analysis,
    is_launching = false,
    has_halted_agents = false
}) => {
    const theme = get_theme_colors(active_cluster.theme);
    const dept_icon_cmp = get_department_icon(active_cluster.department);

    const is_active = active_cluster.is_active;
    const is_running_normally = is_active && !has_halted_agents;

    const resumePauseState = React.useMemo(() => {
        if (has_halted_agents) {
            return {
                label: i18n.t('missions.btn_recover_resume'),
                tooltip: i18n.t('missions.tooltip_recover_resume'),
                style: 'border-amber-500/80 bg-amber-500/10 text-amber-300 shadow-amber-950/40 animate-pulse',
                icon: <RefreshCw size={14} className="animate-spin" />,
            };
        }
        if (is_running_normally) {
            return {
                label: i18n.t('missions.btn_pause'),
                tooltip: i18n.t('missions.tooltip_pause'),
                style: 'border-yellow-500/60 bg-yellow-500/10 text-yellow-300',
                icon: <Pause size={14} />,
            };
        }
        return {
            label: i18n.t('missions.btn_resume'),
            tooltip: i18n.t('missions.tooltip_resume'),
            style: 'border-emerald-500/60 bg-emerald-500/10 text-emerald-300',
            icon: <Play size={14} fill="currentColor" />,
        };
    }, [has_halted_agents, is_running_normally]);

    return (
        <div className="p-6 border-b border-[color:var(--color-border)] bg-[color:var(--color-surface)]/50 backdrop-blur flex flex-col gap-3 relative overflow-hidden">
            <div className="absolute top-0 left-0 w-1 h-full" style={{ backgroundColor: theme.hex }} />
            
            {/* Top Row: Mission Title + Controls */}
            <div className="flex justify-between items-center w-full">
                <div className="flex items-center gap-3">
                    <h1 className="text-lg font-bold text-zinc-100 uppercase tracking-tight">{active_cluster.name}</h1>
                    <Tooltip content={active_cluster.privacy_mode ? 'Air-Gap Active (100% Local Only)' : `Department: ${active_cluster.department}`} position="top">
                        <div
                            aria-label={active_cluster.privacy_mode ? 'Air-Gap Active (100% Local Only)' : `Department: ${active_cluster.department}`}
                            className={`p-1.5 rounded-lg border transition-all ${active_cluster.privacy_mode
                                ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/50 shadow-lg shadow-emerald-500/20 animate-pulse'
                                : `bg-[color:var(--color-surface)] border-[color:var(--color-border)] ${theme.text}`
                                }`}
                        >
                            {React.createElement(dept_icon_cmp, {
                                size: 16,
                                fill: active_cluster.privacy_mode ? "currentColor" : "none",
                                className: active_cluster.privacy_mode ? "animate-pulse" : ""
                            })}
                        </div>
                    </Tooltip>
                </div>

                <div className="flex items-center gap-2">
                    {/* Control Buttons Cluster */}
                    <div className="flex items-center gap-1.5 p-1 rounded-xl bg-[color:var(--color-surface)] border border-[color:var(--color-border)] shadow-inner">
                        {/* 1. RUN MISSION */}
                        <Tooltip content={i18n.t('missions.tooltip_run')} position="top">
                            <button
                                disabled={agents_loading || !has_agents || is_launching}
                                onClick={on_run_mission}
                                aria-label={is_launching ? i18n.t('missions.label_launching') : i18n.t('missions.btn_run')}
                                className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg border ${theme.border} bg-[color:var(--color-surface)] ${theme.text} hover:scale-105 active:scale-95 transition-all text-xs font-bold uppercase tracking-tighter shadow-md ${theme.glow} disabled:opacity-40 disabled:pointer-events-none`}
                            >
                                <Zap size={14} fill="currentColor" className={is_launching ? 'animate-spin' : ''} />
                                {is_launching ? i18n.t('missions.label_launching') : i18n.t('missions.btn_run')}
                            </button>
                        </Tooltip>

                        {/* 2. PAUSE / RESUME / RECOVER */}
                        <Tooltip content={resumePauseState.tooltip} position="top">
                            <button
                                disabled={agents_loading || !has_agents || is_launching}
                                onClick={on_pause_resume_mission}
                                aria-label={resumePauseState.label}
                                className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg border transition-all text-xs font-bold uppercase tracking-tighter shadow-md disabled:opacity-40 disabled:pointer-events-none hover:scale-105 active:scale-95 ${resumePauseState.style}`}
                            >
                                {resumePauseState.icon}
                                {resumePauseState.label}
                            </button>
                        </Tooltip>

                        {/* 3. CANCEL MISSION */}
                        <Tooltip content={i18n.t('missions.tooltip_cancel')} position="top">
                            <button
                                disabled={agents_loading || !has_agents || is_launching}
                                onClick={on_cancel_mission}
                                aria-label={i18n.t('missions.btn_cancel')}
                                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-red-500/40 bg-red-500/10 text-red-400 hover:border-red-500 hover:bg-red-500/20 hover:scale-105 active:scale-95 transition-all text-xs font-bold uppercase tracking-tighter shadow-md disabled:opacity-40 disabled:pointer-events-none"
                            >
                                <OctagonX size={14} />
                                {i18n.t('missions.btn_cancel')}
                            </button>
                        </Tooltip>

                        {/* 4. CLONE MISSION */}
                        {on_clone_mission && (
                            <Tooltip content="Duplicate mission config into a fresh unique ID" position="top">
                                <button
                                    disabled={agents_loading || is_launching}
                                    onClick={on_clone_mission}
                                    aria-label="Clone Mission"
                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-cyan-500/40 bg-cyan-500/10 text-cyan-300 hover:border-cyan-500 hover:bg-cyan-500/20 hover:scale-105 active:scale-95 transition-all text-xs font-bold uppercase tracking-tighter shadow-md disabled:opacity-40 disabled:pointer-events-none"
                                >
                                    <Copy size={14} />
                                    Clone
                                </button>
                            </Tooltip>
                        )}
                    </div>

                    {/* Security Analysis Toggle */}
                    <Tooltip content={i18n.t('missions.tooltip_analysis')} position="top">
                        <div className="flex flex-col items-center gap-1">
                            <button
                                onClick={() => on_toggle_analysis(active_cluster.id)}
                                aria-label={active_cluster.analysis_enabled ? i18n.t('missions.label_analysis_on') : i18n.t('missions.label_analysis_off')}
                                className={`flex items-center gap-2 px-3 py-2 rounded-xl border transition-all text-[10px] font-bold uppercase tracking-wider ${active_cluster.analysis_enabled
                                    ? `${theme.text} ${theme.border} bg-[color:var(--color-surface)] shadow-lg`
                                    : 'text-zinc-500 border-[color:var(--color-border)] bg-[color:var(--color-background)] grayscale'
                                    }`}
                            >
                                <ShieldCheck size={14} className={active_cluster.analysis_enabled ? 'animate-pulse' : ''} />
                                {active_cluster.analysis_enabled ? i18n.t('missions.label_analysis_on') : i18n.t('missions.label_analysis_off')}
                            </button>
                        </div>
                    </Tooltip>
                </div>
            </div>

            {/* Bottom Row: Metadata strip running width-wise under buttons */}
            <div className="flex items-center justify-between w-full pt-2.5 border-t border-[color:var(--color-border)]/50 text-xs text-zinc-400">
                <div className="flex items-center gap-2">
                    <span className="text-zinc-500 font-medium">{i18n.t('missions.label_root_path')}</span>
                    <code className={`${theme.text}/90 font-mono text-xs px-2 py-0.5 rounded bg-[color:var(--color-background)] border border-[color:var(--color-border)]`}>
                        {active_cluster.path}
                    </code>
                </div>

                <div className="flex items-center gap-2 bg-[color:var(--color-surface)]/80 px-3 py-1 rounded-lg border border-[color:var(--color-border)] shadow-sm">
                    <span className="text-zinc-400 font-bold uppercase tracking-wider text-[11px]">Mission ID:</span>
                    <span className="font-mono text-sm font-extrabold text-zinc-100 tracking-wide select-all">
                        {active_cluster.id}
                    </span>
                </div>
            </div>
        </div>
    );
};


// Metadata: [Mission_Header]
