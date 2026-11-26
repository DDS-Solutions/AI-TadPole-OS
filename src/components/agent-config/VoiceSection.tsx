/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Agent-Config / VoiceSection
 * - **Primary Entrypoints**: `VoiceSection`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 * - `[Structural]` Displays live backend audio capabilities so unconfigured or privacy-blocked engines are clearly surfaced.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { useEffect, useState } from 'react';
import { Info, AlertTriangle } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import { system_api_service } from '../../services/system_api_service';
import type { AudioCapabilities } from '../../services/system/engine_api';
import type { Agent_Stt_Engine, Agent_Voice_Engine } from '../../types';

const OPENAI_VOICES = new Set(['alloy', 'echo', 'fable', 'onyx', 'nova', 'shimmer']);

interface VoiceSectionProps {
    voice: {
        voice_id: string;
        voice_engine: Agent_Voice_Engine;
    };
    stt_engine: Agent_Stt_Engine;
    theme_color: string;
    on_update_voice: (field: 'voice_id' | 'voice_engine' | 'stt_engine', value: string) => void;
}

/**
 * Voice_Section
 * Handles agent vocal synthesis and speech-to-text configuration.
 * Queries backend audio capabilities to warn when a selected engine requires API keys,
 * is blocked by Privacy Shield, or requires optional build features.
 */
export function VoiceSection({ voice, stt_engine, theme_color, on_update_voice }: VoiceSectionProps) {
    const [capabilities, set_capabilities] = useState<AudioCapabilities | null>(null);

    useEffect(() => {
        let cancelled = false;
        const loader = system_api_service?.engine?.get_audio_capabilities;
        if (typeof loader !== 'function') return;

        void loader()
            .then((caps) => {
                if (!cancelled && caps) set_capabilities(caps);
            })
            .catch(() => {
                // Keep UI usable when engine is offline or mocked in tests
            });

        return () => {
            cancelled = true;
        };
    }, []);

    const tts_cap = capabilities?.tts?.[voice.voice_engine];
    const active_stt = stt_engine || 'groq';
    const stt_cap = capabilities?.stt?.[active_stt];

    const handle_tts_engine_change = (next_engine: string) => {
        on_update_voice('voice_engine', next_engine);
        if (next_engine === 'openai' && !OPENAI_VOICES.has(voice.voice_id)) {
            on_update_voice('voice_id', 'alloy');
        }
    };

    const format_option_label = (label: string, cap?: { available: boolean }) => {
        if (!cap || cap.available) return label;
        return `${label} (Unavailable)`;
    };

    return (
        <div className="space-y-6">
            <div className="pt-4 border-t border-[color:var(--color-border)]/50">
                <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-2 flex items-center gap-2">
                    {i18n.t('agent_config.voice_integration')}
                    <Tooltip content={i18n.t('agent_config.tooltip_voice_engine')} position="top">
                        <Info size={10} className="text-zinc-700 hover:text-zinc-300 cursor-help transition-colors" />
                    </Tooltip>
                </label>
                <div
                    className="grid grid-cols-2 gap-3 bg-[color:var(--color-surface)]/50 border border-[color:var(--color-border)] rounded-lg p-3 transition-all"
                    style={{ borderLeft: `2px solid ${theme_color}40` }}
                >
                    <div className="space-y-1.5">
                        <label
                            htmlFor="agent-voice-engine"
                            className="block text-[9px] font-bold text-zinc-600 uppercase opacity-60"
                        >
                            {i18n.t('agent_config.label_engine')}
                        </label>
                        <select
                            id="agent-voice-engine"
                            aria-label={i18n.t('agent_config.aria_voice_engine')}
                            value={voice.voice_engine}
                            onChange={(e) => handle_tts_engine_change(e.target.value)}
                            className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 appearance-none font-bold cursor-pointer"
                        >
                            <option value="browser">
                                {format_option_label(i18n.t('agent_config.label_browser_local'), capabilities?.tts?.browser)}
                            </option>
                            <option value="openai">
                                {format_option_label(i18n.t('agent_config.voice_openai_hifi'), capabilities?.tts?.openai)}
                            </option>
                            <option value="gemini-live">
                                {format_option_label(i18n.t('agent_config.voice_gemini_live'), capabilities?.tts?.['gemini-live'])}
                            </option>
                            <option value="piper">
                                {format_option_label(i18n.t('agent_config.voice_piper'), capabilities?.tts?.piper)}
                            </option>
                            {voice.voice_engine === 'groq' && (
                                <option value="groq">
                                    {format_option_label(i18n.t('agent_config.label_fast'), capabilities?.tts?.groq ?? { available: false })}
                                </option>
                            )}
                        </select>
                    </div>
                    <div className="space-y-1.5">
                        <label
                            htmlFor="agent-voice-id"
                            className="block text-[9px] font-bold text-zinc-600 uppercase opacity-60 flex items-center gap-1.5"
                        >
                            {i18n.t('agent_config.label_vocal_identity')}
                            <Tooltip content={i18n.t('agent_config.tooltip_vocal_identity')} position="top">
                                <Info size={9} className="text-zinc-700 hover:text-zinc-300 cursor-help transition-colors" />
                            </Tooltip>
                        </label>
                        {voice.voice_engine === 'openai' ? (
                            <select
                                id="agent-voice-id"
                                aria-label={i18n.t('agent_config.aria_voice_id')}
                                value={OPENAI_VOICES.has(voice.voice_id) ? voice.voice_id : 'alloy'}
                                onChange={(e) => on_update_voice('voice_id', e.target.value)}
                                className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 appearance-none font-bold cursor-pointer"
                            >
                                <option value="alloy">{i18n.t('agent_config.voice_alloy')}</option>
                                <option value="echo">{i18n.t('agent_config.label_echo_deep')}</option>
                                <option value="fable">{i18n.t('agent_config.voice_fable')}</option>
                                <option value="onyx">{i18n.t('agent_config.voice_onyx')}</option>
                                <option value="nova">{i18n.t('agent_config.voice_nova')}</option>
                                <option value="shimmer">{i18n.t('agent_config.voice_shimmer')}</option>
                            </select>
                        ) : (
                            <input
                                id="agent-voice-id"
                                aria-label={i18n.t('agent_config.aria_voice_id')}
                                type="text"
                                placeholder={i18n.t('agent_config.placeholder_voice_id')}
                                value={voice.voice_id}
                                onChange={(e) => on_update_voice('voice_id', e.target.value)}
                                className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 font-mono"
                            />
                        )}
                    </div>
                </div>
                {tts_cap && !tts_cap.available && tts_cap.reason && (
                    <p role="status" className="mt-2 flex items-center gap-1.5 text-[11px] text-amber-400">
                        <AlertTriangle size={12} className="shrink-0" />
                        <span>{tts_cap.reason}</span>
                    </p>
                )}
            </div>

            <div className="pt-4 border-t border-[color:var(--color-border)]/50">
                <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-2 flex items-center gap-2">
                    {i18n.t('agent_config.stt_integration')}
                    <Tooltip content={i18n.t('agent_config.tooltip_stt')} position="top">
                        <Info size={10} className="text-zinc-700 hover:text-zinc-300 cursor-help transition-colors" />
                    </Tooltip>
                </label>
                <div
                    className="bg-[color:var(--color-surface)]/50 border border-[color:var(--color-border)] rounded-lg p-3 transition-all"
                    style={{ borderLeft: `2px solid ${theme_color}40` }}
                >
                    <div className="space-y-1.5">
                        <label
                            htmlFor="agent-stt-engine"
                            className="block text-[9px] font-bold text-zinc-600 uppercase opacity-60"
                        >
                            {i18n.t('agent_config.label_engine')}
                        </label>
                        <select
                            id="agent-stt-engine"
                            aria-label={i18n.t('agent_config.aria_stt_engine')}
                            value={active_stt}
                            onChange={(e) => on_update_voice('stt_engine', e.target.value)}
                            className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 appearance-none font-bold cursor-pointer"
                        >
                            <option value="groq">
                                {format_option_label(i18n.t('agent_config.label_groq_ultra'), capabilities?.stt?.groq)}
                            </option>
                            <option value="whisper">
                                {format_option_label(i18n.t('agent_config.voice_whisper_private'), capabilities?.stt?.whisper)}
                            </option>
                        </select>
                    </div>
                </div>
                {stt_cap && !stt_cap.available && stt_cap.reason && (
                    <p role="status" className="mt-2 flex items-center gap-1.5 text-[11px] text-amber-400">
                        <AlertTriangle size={12} className="shrink-0" />
                        <span>{stt_cap.reason}</span>
                    </p>
                )}
            </div>
        </div>
    );
}
