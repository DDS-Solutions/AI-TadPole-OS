/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Sub-Component**: Extracted input bar for the Sovereign Chat.
 * Isolates voice controls, safety mode toggle, and text submission
 * from the parent message rendering pipeline to prevent cascading re-renders.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Voice client initialization stall, or input focus loss after scope switch.
 * - **Telemetry Link**: Search for `[Chat_Input_Bar]` in browser logs.
 */

import React from 'react';
import {
    Send,
    Mic,
    MicOff,
    BrainCircuit,
    Volume2,
    VolumeX
} from 'lucide-react';
import clsx from 'clsx';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

interface Chat_Input_Bar_Props {
    active_scope: string;
    is_safe_mode: boolean;
    is_speech_enabled: boolean;
    is_speaking: boolean;
    input_value: string;
    on_change: (value: string) => void;
    on_send: () => Promise<void>;
    on_toggle_voice: () => void;
    on_toggle_speech: () => void;
    on_toggle_safety: () => void;
    is_listening: boolean;
}

/**
 * Chat_Input_Bar
 * Handles text input, voice controls, and message submission.
 * Memoized to avoid re-rendering when messages change.
 */
export const Chat_Input_Bar = React.memo<Chat_Input_Bar_Props>(({
    active_scope,
    is_safe_mode,
    is_speech_enabled,
    is_speaking,
    input_value,
    on_change,
    on_send,
    on_toggle_voice,
    on_toggle_speech,
    on_toggle_safety,
    is_listening,
}) => {
    const handle_send = async () => {
        if (!input_value.trim()) return;
        await on_send();
    };

    return (
        <div className="relative z-10 p-4 bg-zinc-950/50 backdrop-blur-xl border-t border-zinc-800/50">
            <div className="flex items-center gap-2 bg-black/40 border border-zinc-800/60 rounded-xl p-1 focus-within:border-zinc-500/50 transition-all shadow-inner relative">
                {is_listening && (
                    <div className="absolute inset-0 bg-green-500/10 animate-pulse pointer-events-none rounded-xl" />
                )}

                <Tooltip content={is_listening ? i18n.t('chat.stop_listening_tooltip') : i18n.t('chat.start_listening_tooltip')} position="top">
                    <button
                        onClick={on_toggle_voice}
                        className={clsx(
                            "p-2.5 rounded-lg transition-all active:scale-90",
                            is_listening ? "bg-red-500 text-white animate-pulse" : "text-zinc-500 hover:text-zinc-100 hover:bg-zinc-800"
                        )}
                    >
                        {is_listening ? <MicOff size={18} /> : <Mic size={18} />}
                    </button>
                </Tooltip>

                <Tooltip content={is_speech_enabled ? i18n.t('chat.mute_output_tooltip') : i18n.t('chat.enable_output_tooltip')} position="top">
                    <button
                        onClick={on_toggle_speech}
                        className={clsx(
                            "p-2.5 rounded-lg transition-all active:scale-90",
                            is_speech_enabled ? "text-green-400 bg-green-500/10" : "text-zinc-500 hover:text-zinc-100 hover:bg-zinc-800"
                        )}
                    >
                        {is_speech_enabled ? <Volume2 size={18} className={is_speaking ? "animate-bounce" : ""} /> : <VolumeX size={18} />}
                    </button>
                </Tooltip>

                <input
                    type="text"
                    value={input_value}
                    onChange={(e) => on_change(e.target.value)}
                    onKeyDown={(e) => {
                        if (e.key === 'Enter') {
                            e.preventDefault();
                            handle_send();
                        }
                    }}
                    placeholder={is_listening ? i18n.t('chat.listening_placeholder') : i18n.t('chat.input_placeholder', { scope: i18n.t(`chat.scope_${active_scope}`) })}
                    className="flex-1 min-w-0 bg-transparent border-none focus:ring-0 text-sm py-2 px-2 text-zinc-100 placeholder:text-zinc-600 font-medium"
                    aria-label={i18n.t('chat.input_label')}
                />

                <Tooltip content={is_safe_mode ? i18n.t('chat.safety_on_tooltip') : i18n.t('chat.safety_off_tooltip')} position="top">
                    <button
                        onClick={on_toggle_safety}
                        className={clsx(
                            "p-2.5 rounded-lg transition-all active:scale-90",
                            is_safe_mode ? "bg-green-500/20 text-green-400 border border-green-500/30" : "text-zinc-500 hover:text-zinc-100 hover:bg-zinc-800"
                        )}
                    >
                        <BrainCircuit size={18} className={is_safe_mode ? "animate-pulse" : ""} />
                    </button>
                </Tooltip>

                <Tooltip content={i18n.t('chat.send_tooltip')} position="top">
                    <button
                        onClick={handle_send}
                        className="p-2.5 bg-zinc-100 text-black rounded-lg hover:bg-white hover:scale-105 active:scale-95 transition-all shadow-lg"
                    >
                        <Send size={16} />
                    </button>
                </Tooltip>
            </div>
        </div>
    );
});

Chat_Input_Bar.displayName = 'Chat_Input_Bar';

// Metadata: [Chat_Input_Bar]
