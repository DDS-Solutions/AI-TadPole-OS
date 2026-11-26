/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Voice / Standup_Meeting_Area
 * - **Primary Entrypoints**: `Standup_Meeting_Area`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { Users, Play, Pause, Mic, MicOff } from 'lucide-react';
import { Tooltip, Execution_Mode_Selector } from '../ui';
import { i18n } from '../../i18n';
import { Audio_Visualizer } from './Audio_Visualizer';
import { Standup_Target_Selector } from './Standup_Target_Selector';
import type { Agent } from '../../types';
import type { Execution_Mode } from '../../contracts/agent';
import type { Mission_Cluster } from '../../stores/workspace_store';
import type { Audio_Input_Device } from '../../services/voice_client';

interface StandupMeetingAreaProps {
    is_live: boolean;
    is_muted?: boolean;
    sync_error: string | null;
    can_start_sync: boolean;
    live_seconds: number;
    target_type: 'agent' | 'cluster';
    selected_target_id: string;
    agents: Agent[];
    clusters: Mission_Cluster[];
    input_devices?: Audio_Input_Device[];
    selected_device_id?: string;
    execution_mode?: Execution_Mode;
    can_launch_swarm?: boolean;
    set_target_type: (type: 'agent' | 'cluster') => void;
    set_selected_target_id: (id: string) => void;
    set_selected_device_id?: (device_id: string) => void;
    set_execution_mode?: (mode: Execution_Mode) => void;
    launch_swarm_handoff?: () => void;
    toggle_live: () => void;
    toggle_mute?: () => void;
}

export function Standup_Meeting_Area({
    is_live,
    is_muted = false,
    sync_error,
    can_start_sync,
    live_seconds,
    target_type,
    selected_target_id,
    agents,
    clusters,
    input_devices = [],
    selected_device_id = '',
    execution_mode = 'swarm',
    can_launch_swarm = false,
    set_target_type,
    set_selected_target_id,
    set_selected_device_id,
    set_execution_mode,
    launch_swarm_handoff,
    toggle_live,
    toggle_mute
}: StandupMeetingAreaProps) {
    const format_time = (total_seconds: number) => {
        const mins = Math.floor(total_seconds / 60);
        const secs = total_seconds % 60;
        return `00:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    };

    return (
        <div className="lg:col-span-2 sovereign-panel p-8 flex flex-col items-center justify-center relative overflow-hidden">
            <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-blue-900/10 via-transparent to-transparent"></div>

            <div className="z-10 text-center space-y-6 flex flex-col items-center">
                <div className="w-32 h-32 rounded-full bg-[color:var(--color-surface)] border-4 border-[color:var(--color-border)] flex items-center justify-center shadow-2xl relative">
                    {is_live && !is_muted ? <div className="absolute inset-0 rounded-full animate-ping bg-green-500/10"></div> : null}
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

                {set_execution_mode && (
                    <Execution_Mode_Selector
                        mode={execution_mode}
                        on_change={set_execution_mode}
                        disabled={is_live}
                        can_launch_swarm={can_launch_swarm}
                        on_launch_swarm={launch_swarm_handoff}
                    />
                )}

                {input_devices.length > 0 && set_selected_device_id && (
                    <div className="w-full max-w-sm">
                        <label htmlFor="mic-device-select" className="sr-only">
                            Microphone input device
                        </label>
                        <select
                            id="mic-device-select"
                            aria-label="Microphone input device"
                            value={selected_device_id}
                            onChange={(e) => set_selected_device_id(e.target.value)}
                            disabled={is_live}
                            className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg px-3 py-1.5 text-[11px] text-zinc-300 font-mono focus:outline-none focus:border-zinc-700 disabled:opacity-50"
                        >
                            <option value="">Default System Microphone</option>
                            {input_devices.map((device) => (
                                <option key={device.device_id} value={device.device_id}>
                                    {device.label}
                                </option>
                            ))}
                        </select>
                    </div>
                )}

                <Audio_Visualizer is_active={is_live && !is_muted} />

                {sync_error && (
                    <div role="status" className="max-w-sm rounded-lg border border-red-500/30 bg-red-950/30 px-3 py-2 text-xs font-medium text-red-300">
                        {sync_error}
                    </div>
                )}

                <div className="flex gap-4">
                    <Tooltip content={is_live ? i18n.t('standups.tooltip_end') : can_start_sync ? i18n.t('standups.tooltip_start') : i18n.t('standups.tooltip_no_target')} position="top">
                        <button
                            onClick={toggle_live}
                            disabled={!is_live && !can_start_sync}
                            className={`px-6 py-2 rounded-full font-bold flex items-center gap-2 transition-all disabled:cursor-not-allowed disabled:bg-zinc-800 disabled:text-zinc-500 ${is_live ? 'bg-red-500/10 text-red-500 hover:bg-red-500/20' : 'bg-emerald-500 text-zinc-950 hover:bg-emerald-400'}`}>
                            {is_live ? <><Pause size={18} /> {i18n.t('standups.btn_end')}</> : <><Play size={18} /> {i18n.t('standups.btn_start')}</>}
                        </button>
                    </Tooltip>
                    <Tooltip content={is_muted ? 'Unmute local microphone capture.' : i18n.t('standups.tooltip_mic')} position="top">
                        <button
                            type="button"
                            onClick={toggle_mute}
                            aria-label={i18n.t('standups.aria_mic')}
                            aria-pressed={is_muted}
                            className={`p-2 rounded-full transition-colors ${
                                is_muted
                                    ? 'bg-amber-900/40 text-amber-400 hover:bg-amber-900/60'
                                    : is_live
                                        ? 'bg-red-900/40 text-red-400 hover:bg-red-900/60'
                                        : 'bg-zinc-800 text-zinc-400 hover:bg-zinc-700'
                            }`}>
                            {is_muted ? <MicOff size={20} /> : <Mic size={20} />}
                        </button>
                    </Tooltip>
                </div>
            </div>
        </div>
    );
}
