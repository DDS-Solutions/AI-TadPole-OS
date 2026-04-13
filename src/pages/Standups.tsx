/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Swarm coordination and synchronization hub. 
 * Orchestrates rapid status updates from active agents and facilitates swarm-wide alignment.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Standup response timeout, or duplicate report generation from a single agent.
 * - **Telemetry Link**: Search for `[Standups_View]` or `SYNC_UP` in service logs.
 */

import { useState, useEffect, useRef, useMemo } from 'react';
import { Mic, Users, Play, Pause, BarChart3, Target, ChevronDown } from 'lucide-react';
import { Tooltip } from '../components/ui';
import { event_bus } from '../services/event_bus';
import { voice_client } from '../services/voice_client';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { use_workspace_store } from '../stores/workspace_store';
import { load_agents } from '../services/agent_service';
import { i18n } from '../i18n';
import { type Agent } from '../types';
import { resolve_agent_model_config } from '../utils/model_utils';
import { Live_Voice_Hub } from '../components/voice/Live_Voice_Hub';

/**
 * Audio_Visualizer
 * Visual representation of the active voice stream.
 * Refactored for strict snake_case compliance for backend parity.
 */
const Audio_Visualizer = ({ is_active }: { is_active: boolean }) => {
    return (
        <div className="flex items-end justify-center gap-1 h-12 w-32">
            {[...Array(8)].map((_, i) => (
                <div
                    key={i}
                    className={`w-2 bg-emerald-500 rounded-t transition-all duration-150 ${is_active ? 'animate-pulse' : 'h-1 bg-zinc-800'}`}
                    style={{ height: is_active ? `${20 + (Math.sin(i * 1.5) * 30 + 30)}%` : '4px' }}
                ></div>
            ))}
        </div>
    );
};

/**
 * Standups
 * Voice communication and activity hub for real-time mission sequences.
 * Integrates with Voice_Client for speech-to-text and text-to-speech.
 * Refactored for strict snake_case compliance for backend parity.
 */
export default function Standups() {
    const [is_live, set_is_live] = useState(false);
    const [transcript_history, set_transcript_history] = useState<string[]>([i18n.t('standups.msg_online')]);
    const [active_speaker, set_active_speaker] = useState<string | null>(null);
    const [agents, set_agents] = useState<Agent[]>([]);
    const clusters = use_workspace_store(state => state.clusters);
    const [target_type, set_target_type] = useState<'agent' | 'cluster'>('agent');
    const [selected_target_id, set_selected_target_id] = useState<string>('');
    const last_spoken_ref = useRef<string | null>(null);

    // Initial setup fetch
    useEffect(() => {
        load_agents().then(data => {
            set_agents(data);
            if (data.length > 0) set_selected_target_id(data[0].id);
        });
    }, []);

    // ── Voice & Transcript Subscriptions ──────────────────────────────────
    useEffect(() => {
        // 1. Subscribe to event_bus for visual transcript
        const unsubscribe_bus = event_bus.subscribe_logs((entry) => {
            if (entry.source === 'User' || entry.source === 'Agent') {
                const speaker_name = entry.source === 'User' ? 'User' : (entry.agent_id || 'Agent');
                const line = `${speaker_name}: ${entry.text}`;

                set_transcript_history(prev => [...prev, line]);
                set_active_speaker(speaker_name);
                setTimeout(() => set_active_speaker(null), 3000);

                // 2. Speak Agent responses (if they change and it's not the same as last spoken)
                if (entry.source === 'Agent' && entry.text !== last_spoken_ref.current) {
                    voice_client.speak(entry.text);
                    last_spoken_ref.current = entry.text;
                }
            }
        });

        // 3. Neural Handoff: High-Fidelity Backend Transcription Orchestration
        const handle_neural_sync = async () => {
            if (is_live) {
                console.debug(`🎙️ [Sovereignty] ${i18n.t('standups.debug_start_recording')}`);
                await voice_client.start_recording();
            } else {
                console.debug(`🎙️ [Sovereignty] ${i18n.t('standups.debug_ending_sync')}`);
                const audio_blob = await voice_client.stop_recording();
                if (audio_blob && selected_target_id) {
                    // Show a temporary "processing" message
                    set_transcript_history(prev => [...prev, i18n.t('standups.msg_transcribing')]);

                    const text = await tadpole_os_service.transcribe(audio_blob);
                    if (text) {
                        const target_name = target_type === 'agent'
                            ? (agents.find(a => a.id === selected_target_id)?.name || i18n.t('standups.label_target_agent'))
                            : (clusters.find(c => c.id === selected_target_id)?.name || i18n.t('standups.label_target_cluster'));

                        event_bus.emit_log({
                            source: 'User',
                            text: `${text} (To: ${target_name})`,
                            severity: 'info'
                        });

                        // Dispatch to backend
                        if (target_type === 'agent') {
                            const agent = agents.find(a => a.id === selected_target_id);
                            if (agent) {
                                const { model_id, provider } = resolve_agent_model_config(agent);
                                tadpole_os_service.send_command(selected_target_id, text, model_id, provider);
                            }
                        } else {
                            const cluster = clusters.find(c => c.id === selected_target_id);
                            if (cluster?.alpha_id) {
                                const alpha_agent = agents.find(a => a.id === cluster.alpha_id);
                                if (alpha_agent) {
                                    const { model_id, provider } = resolve_agent_model_config(alpha_agent);
                                    tadpole_os_service.send_command(
                                        cluster.alpha_id,
                                        `[CLUSTER COMMAND: ${cluster.name}] ${text}`,
                                        model_id,
                                        provider,
                                        cluster.id,
                                        cluster.department,
                                        cluster.budget_usd
                                    );
                                }
                            }
                        }
                    } else {
                        set_transcript_history(prev => [...prev, i18n.t('standups.msg_failed')]);
                    }
                }
            }
        };

        handle_neural_sync();

        return () => {
            unsubscribe_bus();
            voice_client.stop_recording();
        };
    }, [is_live, selected_target_id, agents, clusters, target_type]);

    const active_agent = useMemo(() => 
        agents.find(a => a.id === selected_target_id),
    [agents, selected_target_id]);

    return (
        <div className="h-full grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Active Meeting Area */}
            <div className="lg:col-span-2 bg-zinc-950 border border-zinc-800 rounded-xl p-8 flex flex-col items-center justify-center relative overflow-hidden shadow-sm">
                <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-blue-900/10 via-transparent to-transparent"></div>

                <div className="z-10 text-center space-y-6">
                    <div className="w-32 h-32 rounded-full bg-zinc-900 border-4 border-zinc-800 flex items-center justify-center shadow-2xl relative">
                        {is_live ? <div className="absolute inset-0 rounded-full animate-ping bg-blue-500/10"></div> : null}
                        <Users size={48} className="text-zinc-600" />
                    </div>

                    <Tooltip content={i18n.t('standups.tooltip_interface')} position="bottom">
                        <div>
                            <h2 className="text-2xl font-bold text-zinc-100 cursor-help">{i18n.t('standups.title')}</h2>
                            <p className="text-zinc-500 mt-2 font-mono text-[10px] uppercase tracking-widest">{is_live ? i18n.t('standups.status_live', { time: '00:14:23' }) : i18n.t('standups.status_ready')}</p>
                        </div>
                    </Tooltip>

                    {/* Target Selector */}
                    <div className="w-full max-w-sm flex flex-col gap-3 p-4 bg-zinc-900/50 border border-zinc-800 rounded-2xl backdrop-blur-sm">
                        <div className="flex bg-zinc-950 p-1 rounded-lg border border-zinc-800">
                            <Tooltip content={i18n.t('standups.tooltip_agent')} position="top">
                                <button
                                    onClick={() => { set_target_type('agent'); set_selected_target_id(agents[0]?.id || ''); }}
                                    className={`flex-1 flex items-center justify-center gap-2 py-1.5 text-[10px] font-bold uppercase tracking-widest transition-all rounded-md ${target_type === 'agent' ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-500 hover:text-zinc-300'}`}
                                >
                                    <Users size={12} /> {i18n.t('standups.label_agent_node')}
                                </button>
                            </Tooltip>
                            <Tooltip content={i18n.t('standups.tooltip_cluster')} position="top">
                                <button
                                    onClick={() => { set_target_type('cluster'); set_selected_target_id(clusters[0]?.id || ''); }}
                                    className={`flex-1 flex items-center justify-center gap-2 py-1.5 text-[10px] font-bold uppercase tracking-widest transition-all rounded-md ${target_type === 'cluster' ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-500 hover:text-zinc-300'}`}
                                >
                                    <Target size={12} /> {i18n.t('standups.label_mission_cluster')}
                                </button>
                            </Tooltip>
                        </div>

                        <div className="relative">
                            <label htmlFor="target-select" className="sr-only">{i18n.t('standups.placeholder_select_target')}</label>
                            <select
                                id="target-select"
                                value={selected_target_id}
                                onChange={(e) => set_selected_target_id(e.target.value)}
                                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-4 py-2.5 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 transition-all font-mono appearance-none cursor-pointer"
                            >
                                {target_type === 'agent' ? (
                                    agents.map(a => <option key={a.id} value={a.id}>{a.name.toUpperCase()}</option>)
                                ) : (
                                    clusters.map(c => <option key={c.id} value={c.id}>{c.name.toUpperCase()}</option>)
                                )}
                            </select>
                            <ChevronDown size={14} className="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-600 pointer-events-none" />
                        </div>
                    </div>

                    <Audio_Visualizer is_active={is_live} />

                    <div className="flex gap-4">
                        <Tooltip content={is_live ? i18n.t('standups.tooltip_end') : i18n.t('standups.tooltip_start')} position="top">
                            <button
                                onClick={() => set_is_live(!is_live)}
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

            {/* Live Transcript / Activity */}
            <div className="bg-zinc-950 border border-zinc-800 rounded-xl flex flex-col overflow-hidden">
                <Tooltip content={i18n.t('standups.tooltip_transcript')} position="left">
                    <div className="p-4 border-b border-zinc-800 bg-zinc-900 flex items-center justify-between cursor-help">
                        <h3 className="font-bold text-zinc-400 text-sm flex items-center gap-2">
                            <BarChart3 size={16} /> {i18n.t('standups.header_transcript')}
                        </h3>
                        <div className="flex items-center gap-2">
                            {is_live && <span className="w-2 h-2 bg-red-500 rounded-full animate-pulse"></span>}
                            <span className="text-xs text-zinc-500 font-mono">{is_live ? i18n.t('standups.status_rec') : i18n.t('standups.status_idle')}</span>
                        </div>
                    </div>
                </Tooltip>

                <div className="flex-1 overflow-y-auto p-4 space-y-4">
                    {transcript_history.length === 0 ? (
                        <div className="h-full flex flex-col items-center justify-center text-zinc-600 italic text-sm">
                            {i18n.t('standups.empty_transcript')}
                        </div>
                    ) : (
                        transcript_history.map((line, i) => {
                            const [speaker, text] = line.split(': ');
                            return (
                                <div key={i} className="flex gap-3">
                                    <div className={`w-8 h-8 rounded shrink-0 flex items-center justify-center text-xs font-bold ${speaker === 'User' ? 'bg-emerald-900/50 text-emerald-400' :
                                        speaker === 'Agent' ? 'bg-blue-900/50 text-blue-400' :
                                            speaker === 'Dev-1' ? 'bg-zinc-900/50 text-zinc-400' :
                                                speaker === 'User' ? 'bg-emerald-900/50 text-emerald-400' : 'bg-zinc-800 text-zinc-400'
                                        }`}>
                                        {speaker.substring(0, 1)}
                                    </div>
                                    <div>
                                        <div className="text-xs font-bold text-zinc-400 mb-0.5">{speaker}</div>
                                        <p className="text-sm text-zinc-200">{text}</p>
                                    </div>
                                </div>
                            )
                        })
                    )}
                    {active_speaker && (
                        <div className="flex gap-2 items-center text-zinc-500 text-xs pl-11 animate-pulse">
                            {i18n.t('standups.label_speaking', { name: active_speaker })}
                        </div>
                    )}
                </div>
            </div>

            {/* Gemini Live HUD Overlay */}
            {is_live && target_type === 'agent' && active_agent?.voice_engine === 'gemini-live' && (
                <Live_Voice_Hub 
                    agent_id={selected_target_id}
                    theme_color={active_agent?.theme_color || '#10b981'}
                    on_close={() => set_is_live(false)}
                />
            )}
        </div>
    )
}

