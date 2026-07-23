/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[useStandups]` in observability traces.
 */

import { useState, useEffect, useRef, useMemo, useCallback } from 'react';
import { event_bus } from '../services/event_bus';
import { voice_client } from '../services/voice_client';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { use_workspace_store } from '../stores/workspace_store';
import { load_agents } from '../services/agent_service';
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

    // Initial setup fetch
    useEffect(() => {
        void load_agents().then(() => {
            const current_agents = use_agent_store.getState().agents;
            if (current_agents.length > 0) {
                set_selected_target_id(prev => prev || current_agents[0].id);
            }
        });
    }, []);

    // Timer Logic
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

    // Voice & Transcript Subscriptions
    useEffect(() => {
        const unsubscribe_bus = event_bus.subscribe_logs((entry) => {
            if (entry.source === 'User' || entry.source === 'Agent') {
                const speaker_name = entry.source === 'User' ? 'User' : (entry.agent_id || 'Agent');

                set_transcript_history(prev => [...prev, {
                    speaker: speaker_name,
                    text: entry.text,
                    source: entry.source as 'User' | 'Agent'
                }]);
                set_active_speaker(speaker_name);
                
                if (active_speaker_timeout_ref.current) {
                    clearTimeout(active_speaker_timeout_ref.current);
                }
                active_speaker_timeout_ref.current = setTimeout(() => {
                    set_active_speaker(null);
                    active_speaker_timeout_ref.current = null;
                }, 3000);

                if (entry.source === 'Agent' && entry.text !== last_spoken_ref.current) {
                    voice_client.speak(entry.text);
                    last_spoken_ref.current = entry.text;
                }
            }
        });

        return () => {
            unsubscribe_bus();
            if (active_speaker_timeout_ref.current) {
                clearTimeout(active_speaker_timeout_ref.current);
            }
        };
    }, []);

    useEffect(() => {
        const handle_neural_sync = async () => {
            if (is_live && !prev_is_live_ref.current) {
                if (!can_start_sync) {
                    const message = i18n.t('standups.msg_no_target');
                    set_sync_error(message);
                    append_system_message(message);
                    set_is_live(false);
                    return;
                }

                console.debug(`🎙️ [Sovereignty] ${i18n.t('standups.debug_start_recording')}`);
                const recording_started = await voice_client.start_recording();
                if (!recording_started) {
                    const message = i18n.t('standups.msg_mic_failed');
                    set_sync_error(message);
                    append_system_message(message);
                    prev_is_live_ref.current = false;
                    set_is_live(false);
                    return;
                }

                set_sync_error(null);
                prev_is_live_ref.current = true;
            } else if (!is_live && prev_is_live_ref.current) {
                console.debug(`🎙️ [Sovereignty] ${i18n.t('standups.debug_ending_sync')}`);
                const audio_blob = await voice_client.stop_recording();
                prev_is_live_ref.current = false;

                if (!audio_blob || !selected_target_id) {
                    return;
                }

                append_system_message(i18n.t('standups.msg_transcribing'));

                try {
                    const text = await tadpole_os_service.transcribe(audio_blob);
                    if (!text) {
                        append_system_message(i18n.t('standups.msg_failed'));
                        return;
                    }

                    const target_name = target_type === 'agent'
                        ? (agents.find(a => a.id === selected_target_id)?.name || i18n.t('standups.label_target_agent'))
                        : (clusters.find(c => c.id === selected_target_id)?.name || i18n.t('standups.label_target_cluster'));

                    event_bus.emit_log({
                        source: 'User',
                        text: `${text} (To: ${target_name})`,
                        severity: 'info'
                    });

                    const { get_settings } = await import('../stores/settings_store');
                    if (target_type === 'agent') {
                        const agent = agents.find(a => a.id === selected_target_id);
                        if (!agent) {
                            append_system_message(i18n.t('standups.msg_no_target'));
                            return;
                        }

                        const { model_id, provider } = resolve_agent_model_config(agent, get_settings().default_model);
                        await tadpole_os_service.send_command(selected_target_id, text, model_id, provider);
                    } else {
                        const cluster = clusters.find(c => c.id === selected_target_id);
                        const alpha_id = cluster?.alpha_id;
                        const alpha_agent = alpha_id ? agents.find(a => a.id === alpha_id) : undefined;
                        if (!cluster || !alpha_id || !alpha_agent) {
                            append_system_message(i18n.t('standups.msg_no_target'));
                            return;
                        }

                        const { model_id, provider } = resolve_agent_model_config(alpha_agent, get_settings().default_model);
                        await tadpole_os_service.send_command(
                            alpha_id,
                            `[CLUSTER COMMAND: ${cluster.name}] ${text}`,
                            model_id,
                            provider,
                            cluster.id,
                            cluster.department,
                            cluster.budget_usd
                        );
                    }
                } catch (error) {
                    console.error('[useStandups] Neural sync dispatch failed:', error);
                    append_system_message(i18n.t('standups.msg_dispatch_failed'));
                }
            }
        };

        handle_neural_sync();
    }, [is_live, selected_target_id, agents, clusters, target_type, can_start_sync, append_system_message]);

    useEffect(() => {
        return () => {
            voice_client.stop_recording();
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
        set_target_type,
        set_selected_target_id,
        set_live_seconds,
        toggle_live
    };
}

// Metadata: [useStandups]
