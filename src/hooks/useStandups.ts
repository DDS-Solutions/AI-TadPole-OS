/**
 * @docs ARCHITECTURE:UI-Hooks
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useStandups
 * - **Primary Entrypoints**: `useStandups`, `Transcript_Entry`, `UseStandupsHook`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 * - `[Structural]` Gemini Live sessions delegate microphone capture exclusively to Live_Voice_Hub.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[Sovereignty]`, `[useStandups]`
 */

import { useState, useEffect, useRef, useMemo, useCallback } from 'react';
import { event_bus } from '../services/event_bus';
import { voice_client, type Audio_Input_Device } from '../services/voice_client';
import { system_api_service } from '../services/system_api_service';
import { agent_api_service } from '../services/agent';
import { use_workspace_store } from '../stores/workspace_store';
import { use_agent_store } from '../stores/agent_store';
import { use_settings_store, resolve_execution_mode, type Execution_Mode } from '../stores/settings_store';
import { build_brainstorm_context } from '../utils/brainstorm_handoff';
import { i18n } from '../i18n';
import { type Agent } from '../types';
import { resolve_agent_model_config } from '../utils/model_utils';
import { type Mission_Cluster } from '../stores/workspace_store';

export interface Transcript_Entry {
    speaker: string;
    text: string;
    source: 'User' | 'Agent' | 'System';
}

export interface UseStandupsHook {
    is_live: boolean;
    is_muted: boolean;
    sync_error: string | null;
    can_start_sync: boolean;
    transcript_history: Transcript_Entry[];
    active_speaker: string | null;
    agents: Agent[];
    target_type: 'agent' | 'cluster';
    selected_target_id: string;
    live_seconds: number;
    active_agent: Agent | undefined;
    clusters: Mission_Cluster[];
    input_devices: Audio_Input_Device[];
    selected_device_id: string;
    execution_mode: Execution_Mode;
    can_launch_swarm: boolean;

    set_is_live: (live: boolean) => void;
    set_target_type: (type: 'agent' | 'cluster') => void;
    set_selected_target_id: (id: string) => void;
    set_live_seconds: (seconds: number) => void;
    set_selected_device_id: (device_id: string) => void;
    set_execution_mode: (mode: Execution_Mode) => void;
    launch_swarm_handoff: () => Promise<void>;
    toggle_live: () => void;
    toggle_mute: () => void;
}

function process_log_entry(
    entry: { source?: string; agent_id?: string; text: string },
    last_spoken_ref: React.MutableRefObject<string | null>,
    active_agent_ref: React.MutableRefObject<Agent | undefined>,
    set_transcript_history: React.Dispatch<React.SetStateAction<Transcript_Entry[]>>,
    set_active_speaker: (speaker: string | null) => void,
    timeout_ref: React.MutableRefObject<ReturnType<typeof setTimeout> | null>
) {
    if (entry.source !== 'User' && entry.source !== 'Agent') return;
    const speaker_name = entry.source === 'User' ? 'User' : (entry.agent_id || 'Agent');

    set_transcript_history(prev => [...prev, {
        speaker: speaker_name,
        text: entry.text,
        source: entry.source as 'User' | 'Agent'
    }]);
    set_active_speaker(speaker_name);

    if (timeout_ref.current) clearTimeout(timeout_ref.current);
    timeout_ref.current = setTimeout(() => {
        set_active_speaker(null);
        timeout_ref.current = null;
    }, 3000);

    if (entry.source === 'Agent' && entry.text !== last_spoken_ref.current) {
        const agent = active_agent_ref.current;
        const speak_promise = (agent?.voice_id || agent?.voice_engine)
            ? voice_client.speak(entry.text, agent.voice_id, agent.voice_engine || 'browser')
            : voice_client.speak(entry.text);
        speak_promise?.catch?.(error => {
            console.error('[useStandups] Speech playback failed:', error);
        });
        last_spoken_ref.current = entry.text;
    }
}

interface StandupTargetContext {
    text: string;
    target_id: string;
    target_type: 'agent' | 'cluster';
    agents: Agent[];
    clusters: Mission_Cluster[];
    append_msg: (msg: string) => void;
    execution_mode?: Execution_Mode;
    recent_findings?: string;
}

async function dispatch_standup_target_command(ctx: StandupTargetContext) {
    const { get_settings } = await import('../stores/settings_store');
    const default_model = get_settings().default_model;
    const effective_mode: Execution_Mode = ctx.execution_mode ?? 'swarm';
    const use_extended_payload = effective_mode !== 'swarm' || Boolean(ctx.recent_findings);

    if (ctx.target_type === 'agent') {
        const agent = ctx.agents.find(a => a.id === ctx.target_id);
        if (!agent) {
            ctx.append_msg(i18n.t('standups.msg_no_target'));
            return;
        }
        const { model_id, provider } = resolve_agent_model_config(agent, default_model);
        if (!use_extended_payload) {
            await agent_api_service.send_command(ctx.target_id, ctx.text, model_id, provider);
            return;
        }
        await agent_api_service.send_command({
            agent_id: ctx.target_id,
            message: ctx.text,
            model_id,
            provider,
            safe_mode: effective_mode !== 'swarm',
            execution_mode: effective_mode,
            recent_findings: ctx.recent_findings,
        });
        return;
    }

    const cluster = ctx.clusters.find(c => c.id === ctx.target_id);
    const alpha_id = cluster?.alpha_id;
    const alpha_agent = alpha_id ? ctx.agents.find(a => a.id === alpha_id) : undefined;
    if (!cluster || !alpha_id || !alpha_agent) {
        ctx.append_msg(i18n.t('standups.msg_no_target'));
        return;
    }

    const { model_id, provider } = resolve_agent_model_config(alpha_agent, default_model);
    if (!use_extended_payload) {
        await agent_api_service.send_command(
            alpha_id,
            `[CLUSTER COMMAND: ${cluster.name}] ${ctx.text}`,
            model_id,
            provider,
            cluster.id,
            cluster.department,
            cluster.budget_usd
        );
        return;
    }
    await agent_api_service.send_command({
        agent_id: alpha_id,
        message: `[CLUSTER COMMAND: ${cluster.name}] ${ctx.text}`,
        model_id,
        provider,
        cluster_id: cluster.id,
        department: cluster.department,
        budget_usd: cluster.budget_usd,
        safe_mode: effective_mode !== 'swarm',
        execution_mode: effective_mode,
        recent_findings: ctx.recent_findings,
    });
}

async function handle_finish_recording(
    target_id: string,
    target_type: 'agent' | 'cluster',
    agents: Agent[],
    clusters: Mission_Cluster[],
    append_msg: (msg: string) => void,
    set_sync_error: (err: string | null) => void,
    execution_mode: Execution_Mode,
    recent_findings?: string
) {
    console.debug(`🎙️ [Sovereignty] ${i18n.t('standups.debug_ending_sync')}`);
    const audio_blob = await voice_client.stop_recording();
    if (!audio_blob || !target_id) return;

    append_msg(i18n.t('standups.msg_transcribing'));

    try {
        const target_agent = target_type === 'agent'
            ? agents.find(a => a.id === target_id)
            : undefined;
        const stt_engine = target_agent?.stt_engine;
        const text = stt_engine
            ? await system_api_service.engine.transcribe(audio_blob, { engine: stt_engine })
            : await system_api_service.engine.transcribe(audio_blob);
        if (!text) {
            append_msg(i18n.t('standups.msg_failed'));
            return;
        }

        set_sync_error(null);
        const target_name = target_type === 'agent'
            ? (target_agent?.name || i18n.t('standups.label_target_agent'))
            : (clusters.find(c => c.id === target_id)?.name || i18n.t('standups.label_target_cluster'));

        event_bus.emit_log({
            source: 'User',
            text: `${text} (To: ${target_name})`,
            severity: 'info'
        });

        await dispatch_standup_target_command({
            text,
            target_id,
            target_type,
            agents,
            clusters,
            append_msg,
            execution_mode,
            recent_findings,
        });
    } catch (error) {
        console.error('[useStandups] Neural sync dispatch failed:', error);
        const detail = error instanceof Error && error.message.trim() ? error.message.trim() : '';
        const message = detail
            ? `${i18n.t('standups.msg_dispatch_failed')} (${detail})`
            : i18n.t('standups.msg_dispatch_failed');
        set_sync_error(message);
        append_msg(message);
    }
}

async function handle_start_recording(
    can_start_sync: boolean,
    set_sync_error: (err: string | null) => void,
    append_msg: (msg: string) => void,
    set_is_live: (live: boolean) => void,
    refresh_devices: () => Promise<void>
): Promise<boolean> {
    if (!can_start_sync) {
        const message = i18n.t('standups.msg_no_target');
        set_sync_error(message);
        append_msg(message);
        set_is_live(false);
        return false;
    }

    console.debug(`🎙️ [Sovereignty] ${i18n.t('standups.debug_start_recording')}`);
    const recording_started = await voice_client.start_recording();
    if (!recording_started) {
        const detailed_error = voice_client.get_last_mic_error?.()?.message;
        const message = detailed_error || i18n.t('standups.msg_mic_failed');
        set_sync_error(message);
        append_msg(message);
        set_is_live(false);
        return false;
    }

    set_sync_error(null);
    // Re-enumerate devices once permission is granted so hardware labels populate
    void refresh_devices();
    return true;
}

export function useStandups(): UseStandupsHook {
    const [is_live, set_is_live] = useState(false);
    const [is_muted, set_is_muted] = useState(false);
    const [transcript_history, set_transcript_history] = useState<Transcript_Entry[]>([
        { speaker: 'System', text: i18n.t('standups.msg_online'), source: 'System' }
    ]);
    const [active_speaker, set_active_speaker] = useState<string | null>(null);
    const agents = use_agent_store(state => state.agents);
    const clusters = use_workspace_store(state => state.clusters);
    const { settings, update_setting } = use_settings_store();
    const execution_mode = resolve_execution_mode(settings);
    const [target_type, set_target_type] = useState<'agent' | 'cluster'>('agent');
    const [selected_target_id, set_selected_target_id] = useState<string>('');
    const [live_seconds, set_live_seconds] = useState(0);
    const [sync_error, set_sync_error] = useState<string | null>(null);
    const [input_devices, set_input_devices] = useState<Audio_Input_Device[]>([]);
    const [selected_device_id, set_selected_device_id_state] = useState<string>(
        () => voice_client.get_input_device?.() || ''
    );

    const last_spoken_ref = useRef<string | null>(null);
    const active_speaker_timeout_ref = useRef<ReturnType<typeof setTimeout> | null>(null);
    const prev_is_live_ref = useRef(false);
    const is_gemini_live_session_ref = useRef(false);
    const transcript_history_ref = useRef<Transcript_Entry[]>(transcript_history);
    const execution_mode_ref = useRef<Execution_Mode>(execution_mode);

    useEffect(() => {
        transcript_history_ref.current = transcript_history;
    }, [transcript_history]);

    useEffect(() => {
        execution_mode_ref.current = execution_mode;
    }, [execution_mode]);

    const active_agent = useMemo(
        () => agents.find(a => a.id === selected_target_id),
        [agents, selected_target_id]
    );
    const active_agent_ref = useRef<Agent | undefined>(active_agent);
    useEffect(() => {
        active_agent_ref.current = active_agent;
    }, [active_agent]);

    const refresh_devices = useCallback(async () => {
        const list = await voice_client.list_input_devices?.();
        if (Array.isArray(list)) {
            set_input_devices(list);
        }
    }, []);

    useEffect(() => {
        let cancelled = false;
        const list_fn = voice_client.list_input_devices;
        if (typeof list_fn !== 'function') return;
        void list_fn.call(voice_client).then((list) => {
            if (!cancelled && Array.isArray(list)) {
                set_input_devices(list);
            }
        });
        return () => {
            cancelled = true;
        };
    }, []);

    useEffect(() => {
        void use_agent_store.getState().fetch_agents().then(() => {
            const current_agents = use_agent_store.getState().agents;
            if (current_agents.length > 0) {
                set_selected_target_id(prev => prev || current_agents[0].id);
            }
        });
    }, []);

    const set_selected_device_id = useCallback((device_id: string) => {
        set_selected_device_id_state(device_id);
        voice_client.set_input_device?.(device_id || null);
    }, []);

    const handle_set_target_type = useCallback((type: 'agent' | 'cluster') => {
        set_target_type(type);
        if (type === 'agent') {
            if (!agents.some(a => a.id === selected_target_id) && agents.length > 0) {
                set_selected_target_id(agents[0].id);
            }
        } else {
            if (!clusters.some(c => c.id === selected_target_id) && clusters.length > 0) {
                set_selected_target_id(clusters[0].id);
            }
        }
    }, [agents, clusters, selected_target_id]);

    useEffect(() => {
        let interval: ReturnType<typeof setInterval>;
        if (is_live) {
            interval = setInterval(() => {
                set_live_seconds(prev => prev + 1);
            }, 1000);
        }
        return () => {
            if (interval) clearInterval(interval);
        };
    }, [is_live]);

    const append_system_message = useCallback((text: string) => {
        set_transcript_history(prev => [...prev, {
            speaker: 'System',
            text,
            source: 'System'
        }]);
    }, []);

    const can_start_sync = useMemo(() => {
        if (target_type === 'agent') {
            return agents.some(agent => agent.id === selected_target_id);
        }
        const selected_cluster = clusters.find(cluster => cluster.id === selected_target_id);
        return Boolean(selected_cluster?.alpha_id && agents.some(agent => agent.id === selected_cluster.alpha_id));
    }, [agents, clusters, selected_target_id, target_type]);

    const uses_gemini_live = target_type === 'agent' && active_agent?.voice_engine === 'gemini-live';

    useEffect(() => {
        const timeout_ref = active_speaker_timeout_ref;
        const unsubscribe_bus = event_bus.subscribe_logs((entry) => {
            process_log_entry(
                entry,
                last_spoken_ref,
                active_agent_ref,
                set_transcript_history,
                set_active_speaker,
                timeout_ref
            );
        });

        return () => {
            unsubscribe_bus();
            if (timeout_ref.current) {
                clearTimeout(timeout_ref.current);
            }
        };
    }, []);

    useEffect(() => {
        const handle_neural_sync = async () => {
            if (is_live && !prev_is_live_ref.current) {
                if (uses_gemini_live) {
                    // Live_Voice_Hub owns microphone capture exclusively in gemini-live mode
                    is_gemini_live_session_ref.current = true;
                    prev_is_live_ref.current = true;
                    set_sync_error(null);
                    return;
                }
                is_gemini_live_session_ref.current = false;
                const started = await handle_start_recording(
                    can_start_sync,
                    set_sync_error,
                    append_system_message,
                    set_is_live,
                    refresh_devices
                );
                prev_is_live_ref.current = started;
            } else if (!is_live && prev_is_live_ref.current) {
                prev_is_live_ref.current = false;
                if (is_gemini_live_session_ref.current) {
                    is_gemini_live_session_ref.current = false;
                    return;
                }
                const recent_findings = build_brainstorm_context(
                    transcript_history_ref.current.map((t) => ({
                        speaker: t.speaker,
                        text: t.text,
                        is_system: t.source === 'System',
                    }))
                );
                await handle_finish_recording(
                    selected_target_id,
                    target_type,
                    agents,
                    clusters,
                    append_system_message,
                    set_sync_error,
                    execution_mode_ref.current,
                    recent_findings
                );
            }
        };

        handle_neural_sync().catch(error => {
            console.error('[useStandups] Recording transition failed:', error);
            const detailed_error = voice_client.get_last_mic_error?.()?.message;
            const message = detailed_error || i18n.t('standups.msg_mic_failed');
            set_sync_error(message);
            append_system_message(message);
            prev_is_live_ref.current = false;
            is_gemini_live_session_ref.current = false;
            set_is_live(false);
        });
    }, [is_live, uses_gemini_live, selected_target_id, agents, clusters, target_type, can_start_sync, append_system_message, refresh_devices]);

    useEffect(() => {
        return () => {
            voice_client.stop_recording().catch(error => {
                console.error('[useStandups] Recording cleanup failed:', error);
            });
        };
    }, []);

    const toggle_live = useCallback(() => {
        if (!is_live && !can_start_sync) {
            const message = i18n.t('standups.msg_no_target');
            set_sync_error(message);
            append_system_message(message);
            return;
        }

        if (!is_live) {
            set_live_seconds(0);
            set_sync_error(null);
        }
        set_is_live(!is_live);
    }, [append_system_message, can_start_sync, is_live]);

    const toggle_mute = useCallback(() => {
        const next_muted = typeof voice_client.toggle_mute === 'function'
            ? voice_client.toggle_mute()
            : !is_muted;
        set_is_muted(next_muted);
    }, [is_muted]);

    const set_execution_mode = useCallback((mode: Execution_Mode) => {
        update_setting('execution_mode', mode);
    }, [update_setting]);

    const can_launch_swarm = useMemo(() => {
        if (execution_mode === 'swarm' || !can_start_sync) return false;
        return transcript_history.some((t) => t.source !== 'System' && Boolean(t.text?.trim()));
    }, [execution_mode, can_start_sync, transcript_history]);

    const launch_swarm_handoff = useCallback(async () => {
        if (!selected_target_id || !can_start_sync) {
            const message = i18n.t('standups.msg_no_target');
            set_sync_error(message);
            append_system_message(message);
            return;
        }

        const recent_findings = build_brainstorm_context(
            transcript_history_ref.current.map((t) => ({
                speaker: t.speaker,
                text: t.text,
                is_system: t.source === 'System',
            }))
        );
        update_setting('execution_mode', 'swarm');
        const directive = 'Execute the agreed plan from our prior voice brainstorm session and delegate tasks to the swarm.';
        const target_name = target_type === 'agent'
            ? (agents.find(a => a.id === selected_target_id)?.name || i18n.t('standups.label_target_agent'))
            : (clusters.find(c => c.id === selected_target_id)?.name || i18n.t('standups.label_target_cluster'));

        set_sync_error(null);
        event_bus.emit_log({
            source: 'User',
            text: `${directive} (To: ${target_name})`,
            severity: 'info'
        });

        try {
            await dispatch_standup_target_command({
                text: directive,
                target_id: selected_target_id,
                target_type,
                agents,
                clusters,
                append_msg: append_system_message,
                execution_mode: 'swarm',
                recent_findings,
            });
        } catch (error) {
            console.error('[useStandups] Swarm handoff dispatch failed:', error);
            const detail = error instanceof Error && error.message.trim() ? error.message.trim() : '';
            const message = detail
                ? `${i18n.t('standups.msg_dispatch_failed')} (${detail})`
                : i18n.t('standups.msg_dispatch_failed');
            set_sync_error(message);
            append_system_message(message);
        }
    }, [selected_target_id, can_start_sync, update_setting, target_type, agents, clusters, append_system_message]);

    return {
        is_live,
        is_muted,
        sync_error,
        can_start_sync,
        transcript_history,
        active_speaker,
        agents,
        target_type,
        selected_target_id,
        live_seconds,
        active_agent,
        clusters,
        input_devices,
        selected_device_id,
        execution_mode,
        can_launch_swarm,
        set_is_live,
        set_target_type: handle_set_target_type,
        set_selected_target_id,
        set_live_seconds,
        set_selected_device_id,
        set_execution_mode,
        launch_swarm_handoff,
        toggle_live,
        toggle_mute
    };
}
