/**
 * @docs ARCHITECTURE:UI-Hooks
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useStandups
 * - **Primary Entrypoints**: `useStandups`, `Transcript_Entry`, `UseStandupsHook`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[Sovereignty]`, `[useStandups]`
 */

import { useState, useEffect, useRef, useMemo, useCallback } from 'react';
import { event_bus } from '../services/event_bus';
import { voice_client } from '../services/voice_client';
import { system_api_service } from '../services/system_api_service';
import { agent_api_service } from '../services/agent';
import { use_workspace_store } from '../stores/workspace_store';
import { use_agent_store } from '../stores/agent_store';
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
    
    set_is_live: (live: boolean) => void;
    set_target_type: (type: 'agent' | 'cluster') => void;
    set_selected_target_id: (id: string) => void;
    set_live_seconds: (seconds: number) => void;
    toggle_live: () => void;
}

function process_log_entry(
    entry: { source?: string; agent_id?: string; text: string },
    last_spoken_ref: React.MutableRefObject<string | null>,
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
        voice_client.speak(entry.text)?.catch?.(error => {
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
}

async function dispatch_standup_target_command(ctx: StandupTargetContext) {
    const { get_settings } = await import('../stores/settings_store');
    const default_model = get_settings().default_model;

    if (ctx.target_type === 'agent') {
        const agent = ctx.agents.find(a => a.id === ctx.target_id);
        if (!agent) {
            ctx.append_msg(i18n.t('standups.msg_no_target'));
            return;
        }
        const { model_id, provider } = resolve_agent_model_config(agent, default_model);
        await agent_api_service.send_command(ctx.target_id, ctx.text, model_id, provider);
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
    await agent_api_service.send_command(
        alpha_id,
        `[CLUSTER COMMAND: ${cluster.name}] ${ctx.text}`,
        model_id,
        provider,
        cluster.id,
        cluster.department,
        cluster.budget_usd
    );
}

async function handle_finish_recording(
    target_id: string,
    target_type: 'agent' | 'cluster',
    agents: Agent[],
    clusters: Mission_Cluster[],
    append_msg: (msg: string) => void
) {
    console.debug(`🎙️ [Sovereignty] ${i18n.t('standups.debug_ending_sync')}`);
    const audio_blob = await voice_client.stop_recording();
    if (!audio_blob || !target_id) return;

    append_msg(i18n.t('standups.msg_transcribing'));

    try {
        const text = await system_api_service.engine.transcribe(audio_blob);
        if (!text) {
            append_msg(i18n.t('standups.msg_failed'));
            return;
        }

        const target_name = target_type === 'agent'
            ? (agents.find(a => a.id === target_id)?.name || i18n.t('standups.label_target_agent'))
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
            append_msg
        });
    } catch (error) {
        console.error('[useStandups] Neural sync dispatch failed:', error);
        append_msg(i18n.t('standups.msg_dispatch_failed'));
    }
}

async function handle_start_recording(
    can_start_sync: boolean,
    set_sync_error: (err: string | null) => void,
    append_msg: (msg: string) => void,
    set_is_live: (live: boolean) => void
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
        const message = i18n.t('standups.msg_mic_failed');
        set_sync_error(message);
        append_msg(message);
        set_is_live(false);
        return false;
    }

    set_sync_error(null);
    return true;
}

export function useStandups(): UseStandupsHook {
    const [is_live, set_is_live] = useState(false);
    const [transcript_history, set_transcript_history] = useState<Transcript_Entry[]>([
        { speaker: 'System', text: i18n.t('standups.msg_online'), source: 'System' }
    ]);
    const [active_speaker, set_active_speaker] = useState<string | null>(null);
    const agents = use_agent_store(state => state.agents);
    const clusters = use_workspace_store(state => state.clusters);
    const [target_type, set_target_type] = useState<'agent' | 'cluster'>('agent');
    const [selected_target_id, set_selected_target_id] = useState<string>('');
    const [live_seconds, set_live_seconds] = useState(0);
    const [sync_error, set_sync_error] = useState<string | null>(null);
    
    const last_spoken_ref = useRef<string | null>(null);
    const active_speaker_timeout_ref = useRef<ReturnType<typeof setTimeout> | null>(null);
    const prev_is_live_ref = useRef(false);

    useEffect(() => {
        void use_agent_store.getState().fetch_agents().then(() => {
            const current_agents = use_agent_store.getState().agents;
            if (current_agents.length > 0) {
                set_selected_target_id(prev => prev || current_agents[0].id);
            }
        });
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

    useEffect(() => {
        const timeout_ref = active_speaker_timeout_ref;
        const unsubscribe_bus = event_bus.subscribe_logs((entry) => {
            process_log_entry(entry, last_spoken_ref, set_transcript_history, set_active_speaker, timeout_ref);
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
                const started = await handle_start_recording(can_start_sync, set_sync_error, append_system_message, set_is_live);
                prev_is_live_ref.current = started;
            } else if (!is_live && prev_is_live_ref.current) {
                prev_is_live_ref.current = false;
                await handle_finish_recording(selected_target_id, target_type, agents, clusters, append_system_message);
            }
        };

        handle_neural_sync().catch(error => {
            console.error('[useStandups] Recording transition failed:', error);
            const message = i18n.t('standups.msg_mic_failed');
            set_sync_error(message);
            append_system_message(message);
            prev_is_live_ref.current = false;
            set_is_live(false);
        });
    }, [is_live, selected_target_id, agents, clusters, target_type, can_start_sync, append_system_message]);

    useEffect(() => {
        return () => {
            voice_client.stop_recording().catch(error => {
                console.error('[useStandups] Recording cleanup failed:', error);
            });
        };
    }, []);

    const active_agent = useMemo(() => 
        agents.find(a => a.id === selected_target_id),
    [agents, selected_target_id]);

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

    return {
        is_live,
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
        set_is_live,
        set_target_type: handle_set_target_type,
        set_selected_target_id,
        set_live_seconds,
        toggle_live
    };
}
