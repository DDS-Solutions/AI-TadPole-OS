/**
 * @docs ARCHITECTURE:Logic
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / use_chat_voice
 * - **Primary Entrypoints**: `useChatVoice`, `ChatVoiceState`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useState, useRef, useEffect, useCallback } from 'react';
import { voice_client, type Voice_Status } from '../services/voice_client';
import type { Chat_Message } from '../stores/sovereign_store';
import type { Agent } from '../types';

const TELEMETRY_SOURCE = '[useChatVoice]';

export interface ChatVoiceState {
    voice_status: Voice_Status;
    is_speech_enabled: boolean;
    is_speaking: boolean;
    toggle_voice: () => void;
    toggle_speech: () => void;
}

/**
 * useChatVoice
 * Encapsulates all voice synthesis/recognition state and side-effects.
 * - Subscribes to `voice_client.on_status_change`
 * - Auto-speaks the latest agent message when speech is enabled
 * - Provides toggle callbacks for voice input and speech output
 */
export function useChatVoice(
    messages: Chat_Message[],
    selected_agent_id: string | null,
    agents: Agent[]
): ChatVoiceState {
    const is_mounted_ref = useRef(true);
    const [voice_status, set_voice_status] = useState<Voice_Status>('idle');
    const [is_speech_enabled, set_is_speech_enabled] = useState(false);
    const [is_speaking, set_is_speaking] = useState(false);

    const last_spoken_id_ref = useRef<string | null>(null);
    const speak_start_timeout_ref = useRef<ReturnType<typeof setTimeout> | null>(null);
    const speak_end_timeout_ref = useRef<ReturnType<typeof setTimeout> | null>(null);

    // Track mount state for async safety
    useEffect(() => {
        is_mounted_ref.current = true;
        return () => {
            is_mounted_ref.current = false;
        };
    }, []);

    // Subscribe to voice status changes for UI feedback
    useEffect(() => {
        const unsubscribe = voice_client.on_status_change(status => {
            if (is_mounted_ref.current) {
                console.debug(`${TELEMETRY_SOURCE} Voice status transition: ${status}`);
                set_voice_status(status);
            }
        });
        return unsubscribe;
    }, []);

    // AUTO-SPEAK LOGIC: speak the latest agent message
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

        if (is_speech_enabled && last_message.sender_id !== '0' && last_message.sender_id === selected_agent_id) {
            // Guard: Don't auto-speak technical errors or security alerts
            if (last_message.text.startsWith('❌') || last_message.text.startsWith('🛡️') || last_message.text.includes('Error:')) {
                return;
            }

            const agent = agents.find(a => a.id === selected_agent_id);
            if (agent) {
                last_spoken_id_ref.current = last_message.id; // Mark as handled
                speak_start_timeout_ref.current = setTimeout(() => {
                    if (is_mounted_ref.current) {
                        set_is_speaking(true);
                    }
                }, 0);
                voice_client.speak(last_message.text, agent.voice_id, agent.voice_engine || 'browser').finally(() => {
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

    const toggle_voice = useCallback(() => {
        if (voice_status !== 'idle') {
            voice_client.stop_listening();
        } else {
            set_is_speech_enabled(true);
            voice_client.start_listening(() => {
                // Potential hook for real-time log ingestion
            });
        }
    }, [voice_status]);

    const toggle_speech = useCallback(() => {
        set_is_speech_enabled(prev => !prev);
    }, []);

    return {
        voice_status,
        is_speech_enabled,
        is_speaking,
        toggle_voice,
        toggle_speech,
    };
}
