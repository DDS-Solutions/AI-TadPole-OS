/**
 * @docs ARCHITECTURE:Logic
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / use_chat_voice
 * - **Primary Entrypoints**: `useChatVoice`, `ChatVoiceState`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 * - `[Structural]` Falls back to MediaRecorder + /v1/engine/transcribe when browser Web Speech API is unavailable.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[useChatVoice]`
 */

import { useState, useRef, useEffect, useCallback } from 'react';
import { voice_client, type Voice_Status } from '../services/voice_client';
import { system_api_service } from '../services/system_api_service';
import type { Chat_Message } from '../stores/sovereign_store';
import type { Agent } from '../types';

const CHAT_RECORDER_OWNER = 'chat-recorder';

export interface ChatVoiceState {
    voice_status: Voice_Status;
    voice_error: string | null;
    clear_voice_error: () => void;
    is_speech_enabled: boolean;
    is_speaking: boolean;
    toggle_voice: () => void;
    toggle_speech: () => void;
}

/**
 * useChatVoice
 * Encapsulates all voice synthesis/recognition state and side-effects for Neural Chat Command:
 * - Subscribes to `voice_client.on_status_change` and `voice_client.on_error`
 * - Uses Web Speech API when available, automatically falling back to MediaRecorder + engine STT
 * - Auto-speaks finalized non-error agent responses when speech output is enabled
 */
export function useChatVoice(
    messages: Chat_Message[],
    selected_agent_id: string | null,
    agents: Agent[],
    on_speech_input?: (text: string) => void
): ChatVoiceState {
    const is_mounted_ref = useRef(true);
    const on_speech_input_ref = useRef(on_speech_input);
    const [voice_status, set_voice_status] = useState<Voice_Status>('idle');
    const [voice_error, set_voice_error] = useState<string | null>(null);
    const [is_speech_enabled, set_is_speech_enabled] = useState(false);
    const [is_speaking, set_is_speaking] = useState(false);

    const is_fallback_recording_ref = useRef(false);
    const is_tts_active_ref = useRef(false);
    const tts_echo_cooldown_until_ref = useRef(0);
    const last_spoken_id_ref = useRef<string | null>(null);
    const speak_start_timeout_ref = useRef<ReturnType<typeof setTimeout> | null>(null);
    const speak_end_timeout_ref = useRef<ReturnType<typeof setTimeout> | null>(null);
    const selected_agent_ref = useRef<Agent | undefined>(undefined);

    useEffect(() => {
        selected_agent_ref.current = agents.find((a) => a.id === selected_agent_id);
    }, [agents, selected_agent_id]);

    // Track mount state for async safety and release fallback recording on unmount
    useEffect(() => {
        is_mounted_ref.current = true;
        return () => {
            is_mounted_ref.current = false;
            if (is_fallback_recording_ref.current) {
                is_fallback_recording_ref.current = false;
                void voice_client.stop_recording?.(CHAT_RECORDER_OWNER);
            }
        };
    }, []);

    // Subscribe to voice status and error updates for UI feedback
    useEffect(() => {
        const unsub_status = voice_client.on_status_change((status) => {
            if (is_mounted_ref.current && !is_fallback_recording_ref.current) {
                console.debug(`[useChatVoice] Voice status transition: ${status}`);
                set_voice_status(status);
                if (status === 'active') {
                    set_voice_error(null);
                }
            }
        });
        const unsub_error = voice_client.on_error?.((err) => {
            if (is_mounted_ref.current) {
                set_voice_error(err.message);
            }
        });
        return () => {
            unsub_status();
            unsub_error?.();
        };
    }, []);

    // AUTO-SPEAK LOGIC: speak finalized non-error agent messages
    useEffect(() => {
        if (speak_start_timeout_ref.current) {
            clearTimeout(speak_start_timeout_ref.current);
            speak_start_timeout_ref.current = null;
        }
        if (speak_end_timeout_ref.current) {
            clearTimeout(speak_end_timeout_ref.current);
            speak_end_timeout_ref.current = null;
        }

        const last_message = messages[messages.length - 1];
        if (!last_message || last_message.id === last_spoken_id_ref.current) return;

        // Skip pure thought traces so internal reasoning isn't read aloud
        const is_thought_only = Boolean(
            last_message.parts &&
            last_message.parts.length > 0 &&
            last_message.parts.every((p) => p.type === 'thought')
        );
        if (is_thought_only) return;

        const is_agent_sender = last_message.sender_id !== '0' && last_message.sender_id !== 'system';
        const matches_active_or_broadcast =
            last_message.sender_id === selected_agent_id ||
            last_message.scope === 'cluster' ||
            last_message.scope === 'swarm';

        if (is_speech_enabled && is_agent_sender && matches_active_or_broadcast) {
            if (
                last_message.text.startsWith('❌') ||
                last_message.text.startsWith('🛡️') ||
                last_message.text.includes('Error:')
            ) {
                return;
            }

            const agent =
                agents.find((a) => a.id === last_message.sender_id || a.id === last_message.agent_id) ||
                agents.find((a) => a.id === selected_agent_id);
            if (agent) {
                last_spoken_id_ref.current = last_message.id;
                is_tts_active_ref.current = true;
                speak_start_timeout_ref.current = setTimeout(() => {
                    if (is_mounted_ref.current) {
                        set_is_speaking(true);
                    }
                }, 0);
                void voice_client
                    .speak(last_message.text, agent.voice_id, agent.voice_engine || 'browser')
                    .catch((error) => {
                        console.error(`[useChatVoice] Speech playback failed:`, error);
                    })
                    .finally(() => {
                        is_tts_active_ref.current = false;
                        tts_echo_cooldown_until_ref.current = Date.now() + 450;
                        if (is_mounted_ref.current) {
                            speak_end_timeout_ref.current = setTimeout(() => {
                                if (is_mounted_ref.current) {
                                    set_is_speaking(false);
                                }
                            }, Math.min(10000, last_message.text.length * 60));
                        }
                    });
            }
        }

        return () => {
            if (speak_start_timeout_ref.current) {
                clearTimeout(speak_start_timeout_ref.current);
                speak_start_timeout_ref.current = null;
            }
            if (speak_end_timeout_ref.current) {
                clearTimeout(speak_end_timeout_ref.current);
                speak_end_timeout_ref.current = null;
            }
        };
    }, [messages, is_speech_enabled, selected_agent_id, agents]);

    useEffect(() => {
        on_speech_input_ref.current = on_speech_input;
    }, [on_speech_input]);

    const clear_voice_error = useCallback(() => {
        set_voice_error(null);
    }, []);

    const stop_fallback_recording = useCallback(async () => {
        is_fallback_recording_ref.current = false;
        set_voice_status('idle');
        try {
            const blob = await voice_client.stop_recording?.(CHAT_RECORDER_OWNER);
            if (!blob || blob.size === 0) return;
            const stt_engine = selected_agent_ref.current?.stt_engine;
            const text = stt_engine
                ? await system_api_service.engine.transcribe(blob, { engine: stt_engine })
                : await system_api_service.engine.transcribe(blob);
            if (is_mounted_ref.current && text && text.trim()) {
                set_voice_error(null);
                on_speech_input_ref.current?.(text.trim());
            }
        } catch (err) {
            console.error(`[useChatVoice] Fallback STT transcription failed:`, err);
            if (is_mounted_ref.current) {
                set_voice_error(
                    err instanceof Error
                        ? `Voice transcription failed: ${err.message}`
                        : 'Voice transcription failed. Check STT provider settings.'
                );
            }
        }
    }, []);

    const start_fallback_recording = useCallback(async () => {
        if (typeof voice_client.start_recording !== 'function') {
            set_voice_error('Voice dictation is not supported in this browser/WebView.');
            return;
        }
        set_voice_status('initializing');
        const started = await voice_client.start_recording(CHAT_RECORDER_OWNER);
        if (!is_mounted_ref.current) {
            if (started) void voice_client.stop_recording?.(CHAT_RECORDER_OWNER);
            return;
        }
        if (started) {
            is_fallback_recording_ref.current = true;
            set_voice_error(null);
            set_voice_status('active');
        } else {
            is_fallback_recording_ref.current = false;
            set_voice_status('idle');
            set_voice_error(
                voice_client.get_last_mic_error?.()?.message ||
                    'Microphone capture failed. Check browser permission and audio device availability.'
            );
        }
    }, []);

    const toggle_voice = useCallback(() => {
        if (is_fallback_recording_ref.current) {
            void stop_fallback_recording();
            return;
        }

        if (voice_status !== 'idle') {
            voice_client.stop_listening();
            return;
        }

        set_voice_error(null);
        const speech_supported =
            typeof voice_client.is_speech_recognition_supported === 'function'
                ? voice_client.is_speech_recognition_supported()
                : true;

        if (!speech_supported) {
            void start_fallback_recording();
            return;
        }

        const started = voice_client.start_listening((text: string) => {
            if (
                is_tts_active_ref.current ||
                Date.now() < tts_echo_cooldown_until_ref.current ||
                voice_client.is_speaking_tts?.()
            ) {
                return;
            }
            if (text && text.trim()) {
                on_speech_input_ref.current?.(text.trim());
            }
        });

        // When Web Speech API explicitly returns false (unavailable/failed to start), fall back to recorder STT
        if (started === false) {
            void start_fallback_recording();
        }
    }, [voice_status, start_fallback_recording, stop_fallback_recording]);

    const toggle_speech = useCallback(() => {
        set_is_speech_enabled((prev) => !prev);
    }, []);

    return {
        voice_status,
        voice_error,
        clear_voice_error,
        is_speech_enabled,
        is_speaking,
        toggle_voice,
        toggle_speech,
    };
}
