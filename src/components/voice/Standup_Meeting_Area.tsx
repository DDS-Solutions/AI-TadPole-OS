/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Standup_Meeting_Area]` in observability traces.
 */

import { Users, Play, Pause, Mic } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import { Audio_Visualizer } from './Audio_Visualizer';
import { Standup_Target_Selector } from './Standup_Target_Selector';
import type { Agent } from '../../types';
import type { Mission_Cluster } from '../../stores/workspace_store';

interface StandupMeetingAreaProps {
    is_live: boolean;
    live_seconds: number;
    target_type: 'agent' | 'cluster';
    selected_target_id: string;
    agents: Agent[];
    clusters: Mission_Cluster[];
    set_target_type: (type: 'agent' | 'cluster') => void;
    set_selected_target_id: (id: string) => void;
    toggle_live: () => void;
}

export function Standup_Meeting_Area({
    is_live,
    live_seconds,
    target_type,
    selected_target_id,
    agents,
    clusters,
    set_target_type,
    set_selected_target_id,
    toggle_live
}: StandupMeetingAreaProps) {
    const format_time = (total_seconds: number) => {
        const mins = Math.floor(total_seconds / 60);
        const secs = total_seconds % 60;
        return `00:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    };

    return (
        <div className="lg:col-span-2 bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-xl p-8 flex flex-col items-center justify-center relative overflow-hidden shadow-sm">
            <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-blue-900/10 via-transparent to-transparent"></div>

            <div className="z-10 text-center space-y-6">
                <div className="w-32 h-32 rounded-full bg-[color:var(--color-surface)] border-4 border-[color:var(--color-border)] flex items-center justify-center shadow-2xl relative">
                    {is_live ? <div className="absolute inset-0 rounded-full animate-ping bg-green-500/10"></div> : null}
                    <Users size={48} className="text-zinc-600" />
                </div>

                <Tooltip content={i18n.t('standups.tooltip_interface')} position="bottom">
                    <div>
                        <h2 className="text-2xl font-bold text-zinc-100 cursor-help">{i18n.t('standups.title')}</h2>
                        <p className="text-zinc-500 mt-2 font-mono text-[10px] uppercase tracking-widest">
                            {is_live ? i18n.t('standups.status_live', { time: format_time(live_seconds) }) : i18n.t('standups.status_ready')}
                        </p>
                    </div>
                </Tooltip>

                <Standup_Target_Selector 
                    target_type={target_type}
                    selected_target_id={selected_target_id}
                    agents={agents}
                    clusters={clusters}
                    set_target_type={set_target_type}
                    set_selected_target_id={set_selected_target_id}
                />

                <Audio_Visualizer is_active={is_live} />

                <div className="flex gap-4">
                    <Tooltip content={is_live ? i18n.t('standups.tooltip_end') : i18n.t('standups.tooltip_start')} position="top">
                        <button
                            onClick={toggle_live}
                            className={`px-6 py-2 rounded-full font-bold flex items-center gap-2 transition-all ${is_live ? 'bg-red-500/10 text-red-500 hover:bg-red-500/20' : 'bg-emerald-500 text-black hover:bg-emerald-400'}`}>
                            {is_live ? <><Pause size={18} /> {i18n.t('standups.btn_end')}</> : <><Play size={18} /> {i18n.t('standups.btn_start')}</>}
                        </button>
                    </Tooltip>
                    <Tooltip content={i18n.t('standups.tooltip_mic')} position="top">
                        <button className={`p-2 rounded-full text-zinc-400 hover:bg-zinc-700 transition-colors ${is_live ? 'bg-red-900/40 text-red-400' : 'bg-zinc-800'}`}>
                            <Mic size={20} />
                        </button>
                    </Tooltip>
                </div>
            </div>
        </div>
    );
}

// Metadata: [Standup_Meeting_Area]
