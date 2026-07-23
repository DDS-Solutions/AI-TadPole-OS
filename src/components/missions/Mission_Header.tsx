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
import { Zap, Pause, Play, RefreshCw, OctagonX, ShieldCheck } from 'lucide-react';
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
    on_toggle_analysis,
    is_launching = false,
    has_halted_agents = false
}) => {
    const theme = get_theme_colors(active_cluster.theme);
    const dept_icon_cmp = get_department_icon(active_cluster.department);

    const is_active = active_cluster.is_active;
    const is_running_normally = is_active && !has_halted_agents;

    return (
        <div className="p-6 border-b border-[color:var(--color-border)] bg-[color:var(--color-surface)]/50 backdrop-blur flex justify-between items-center relative overflow-hidden">
            <div className="absolute top-0 left-0 w-1 h-full" style={{ backgroundColor: theme.hex }} />
            <div>
                <h1 className="text-lg font-bold text-zinc-100 uppercase tracking-tight">{active_cluster.name}</h1>
                <p className="text-xs text-zinc-500 mt-1">
                    {i18n.t('missions.label_root_path')} <code className={`${theme.text}/80`}>{active_cluster.path}</code>
                    <span className="mx-2 text-zinc-600">|</span>
                    <span className="font-mono text-[10px] text-zinc-500 uppercase tracking-wider">Mission ID: {active_cluster.id}</span>
                </p>
            </div>
            <div className="flex items-center gap-2">
                {/* 3-Button Mission Control Cluster */}
                <div className="flex items-center gap-1.5 p-1 rounded-xl bg-[color:var(--color-surface)] border border-[color:var(--color-border)] shadow-inner">
                    {/* 1. RUN MISSION */}
                    <Tooltip content={i18n.t('missions.tooltip_run')} position="top">
                        <button
                            disabled={agents_loading || !has_agents || is_launching}
                            onClick={on_run_mission}
                            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg border ${theme.border} bg-[color:var(--color-surface)] ${theme.text} hover:scale-105 active:scale-95 transition-all text-xs font-bold uppercase tracking-tighter shadow-md ${theme.glow} disabled:opacity-40 disabled:pointer-events-none`}
                        >
                            <Zap size={14} fill="currentColor" className={is_launching ? 'animate-spin' : ''} />
                            {is_launching ? i18n.t('missions.label_launching') : i18n.t('missions.btn_run')}
                        </button>
                    </Tooltip>

                    {/* 2. PAUSE / RESUME / RECOVER */}
                    <Tooltip 
                        content={
                            has_halted_agents 
                                ? i18n.t('missions.tooltip_recover_resume') 
                                : is_running_normally 
                                    ? i18n.t('missions.tooltip_pause') 
                                    : i18n.t('missions.tooltip_resume')
                        } 
                        position="top"
                    >
                        <button
                            disabled={agents_loading || !has_agents || is_launching}
                            onClick={on_pause_resume_mission}
                            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg border transition-all text-xs font-bold uppercase tracking-tighter shadow-md disabled:opacity-40 disabled:pointer-events-none hover:scale-105 active:scale-95 ${
                                has_halted_agents
                                    ? 'border-amber-500/80 bg-amber-500/10 text-amber-300 shadow-amber-950/40 animate-pulse'
                                    : is_running_normally
                                        ? 'border-yellow-500/60 bg-yellow-500/10 text-yellow-300'
                                        : 'border-emerald-500/60 bg-emerald-500/10 text-emerald-300'
                            }`}
                        >
                            {has_halted_agents ? (
                                <>
                                    <RefreshCw size={14} className="animate-spin" />
                                    {i18n.t('missions.btn_recover_resume')}
                                </>
                            ) : is_running_normally ? (
                                <>
                                    <Pause size={14} />
                                    {i18n.t('missions.btn_pause')}
                                </>
                            ) : (
                                <>
                                    <Play size={14} fill="currentColor" />
                                    {i18n.t('missions.btn_resume')}
                                </>
                            )}
                        </button>
                    </Tooltip>

                    {/* 3. CANCEL MISSION */}
                    <Tooltip content={i18n.t('missions.tooltip_cancel')} position="top">
                        <button
                            disabled={agents_loading || !has_agents || is_launching}
                            onClick={on_cancel_mission}
                            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-red-500/40 bg-red-500/10 text-red-400 hover:border-red-500 hover:bg-red-500/20 hover:scale-105 active:scale-95 transition-all text-xs font-bold uppercase tracking-tighter shadow-md disabled:opacity-40 disabled:pointer-events-none"
                        >
                            <OctagonX size={14} />
                            {i18n.t('missions.btn_cancel')}
                        </button>
                    </Tooltip>
                </div>

                {/* Security Analysis Toggle */}
                <Tooltip content={i18n.t('missions.tooltip_analysis')} position="top">
                    <div className="flex flex-col items-center gap-1">
                        <button
                            onClick={() => on_toggle_analysis(active_cluster.id)}
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

                {/* Department Icon */}
                <div className={`p-2.5 bg-[color:var(--color-surface)] rounded-xl border border-[color:var(--color-border)] ${theme.text}`}>
                    {React.createElement(dept_icon_cmp, { size: 22 })}
                </div>
            </div>
        </div>
    );
};


// Metadata: [Mission_Header]
