/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Voice / Live_Voice_Hub
 * - **Primary Entrypoints**: `Live_Voice_Hub`
 *
 * ### AI Assist Note
 * - Voice sessions use the configured engine and in-memory token; settings changes close the old session.
 * - Acquires exclusive microphone lease via `voice_client` and decodes raw PCM16 Gemini Live frames cleanly.
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[Live_Voice_Hub]`
 */

import { useEffect, useRef, useState, useCallback } from 'react';
import { Mic, MicOff, Waves, Volume2, X } from 'lucide-react';
import { i18n } from '../../i18n';
import { Z_INDEX_MAP } from '../ui/theme_tokens';
import { use_settings_store } from '../../stores/settings_store';
import { validate_and_sanitize_url } from '../../api/utils/url';
import { is_allowed_origin, is_private_network_origin } from '../../services/socket/security/origin_guard';
import { voice_client, classify_mic_error } from '../../services/voice_client';

// ─── Module-level Constants ──────────────────────────────────────────────────

/** Max positive value for a signed 16-bit integer. Used for PCM encoding. */
const PCM16_MAX = 0x7FFF; // 32767
const LIVE_MIC_LEASE_OWNER = 'live-voice-hub';
const DEFAULT_GEMINI_PCM_RATE = 24000;

// ─── Module-level Pure Utilities ────────────────────────────────────────────

/**
 * Converts a Float32 PCM buffer (range -1.0 to 1.0) to Int16 PCM.
 * Clamps out-of-range samples to prevent integer overflow artifacts.
 */
function convert_float32_to_pcm16(buffer: Float32Array): Int16Array {
    const out = new Int16Array(buffer.length);
    for (let i = 0; i < buffer.length; i++) {
        out[i] = Math.max(-1, Math.min(1, buffer[i])) * PCM16_MAX;
    }
    return out;
}

/**
 * Parses sample rate from a PCM mime type (e.g. `audio/pcm;rate=24000`).
 */
function parse_pcm_sample_rate(mime_type?: string): number {
    if (!mime_type) return DEFAULT_GEMINI_PCM_RATE;
    const match = /rate\s*=\s*(\d+)/i.exec(mime_type);
    const parsed = match ? Number(match[1]) : NaN;
    return Number.isFinite(parsed) && parsed >= 8000 && parsed <= 96000 ? parsed : DEFAULT_GEMINI_PCM_RATE;
}

// ─── Types ───────────────────────────────────────────────────────────────────

interface Live_Voice_Hub_Props {
    agent_id: string;
    /**
     * Accent color for the mic button glow and visualizer bars.
     * Must be a 6-digit hex string (e.g. `#10b981`) — the component appends
     * a 2-digit hex alpha suffix for shadow generation (`${theme_color}40`).
     */
    theme_color: string;
    on_close?: () => void;
}

// ─── Component ───────────────────────────────────────────────────────────────

/**
 * Live_Voice_Hub
 * A specialized voice communication overlay for real-time Gemini Live sessions.
 * Manages WebSocket streams, audio processing, and high-fidelity visual feedback.
 */
export function Live_Voice_Hub({ agent_id, theme_color, on_close }: Live_Voice_Hub_Props) {
    const [connected, set_connected] = useState(false);
    const [active, set_active] = useState(false);
    const [connection_error, set_connection_error] = useState<string | null>(null);
    const engine_url = use_settings_store(state => state.settings.tadpole_os_url);
    const engine_token = use_settings_store(state => state.settings.tadpole_os_api_key);
    const allowed_origins = use_settings_store(state => state.settings.allowed_origins);

    // Refs for audio infrastructure — never stored in state to avoid re-renders
    const ws_ref = useRef<WebSocket | null>(null);
    const audio_context_ref = useRef<AudioContext | null>(null);
    const stream_ref = useRef<MediaStream | null>(null);
    const processor_ref = useRef<AudioNode | null>(null);
    const sink_gain_ref = useRef<GainNode | null>(null);
    const mic_generation = useRef(0);
    const mic_pending = useRef(false);

    // Scheduler ref: tracks when the next audio chunk should begin playing
    const next_play_time_ref = useRef<number>(0);

    // Visualizer: volume is tracked in a ref + RAF to avoid React re-renders at 40Hz
    const volume_ref = useRef<number>(0);
    const bar_refs = useRef<(HTMLDivElement | null)[]>([]);
    const volume_bar_ref = useRef<HTMLDivElement | null>(null);
    const raf_ref = useRef<number | null>(null);
    const [visualizer_offsets] = useState(() => Array.from({ length: 8 }, () => Math.random()));

    // ─── Visualizer & Volume Bar Animation Loop ──────────────────────────────
    const visualizer_frame_ref = useRef<(() => void) | null>(null);

    useEffect(() => {
        visualizer_frame_ref.current = () => {
            const vol = volume_ref.current;

            bar_refs.current.forEach((bar, idx) => {
                if (!bar) return;
                const height = active ? `${20 + visualizer_offsets[idx] * vol * 200}%` : '4px';
                bar.style.height = height;
                bar.style.opacity = active ? '0.8' : '0.2';
            });

            if (volume_bar_ref.current) {
                volume_bar_ref.current.style.width = active ? `${Math.min(vol * 500, 100)}%` : '0%';
            }
        };
    });

    useEffect(() => {
        const tick = () => {
            visualizer_frame_ref.current?.();
            raf_ref.current = requestAnimationFrame(tick);
        };
        raf_ref.current = requestAnimationFrame(tick);
        return () => {
            if (raf_ref.current !== null) cancelAnimationFrame(raf_ref.current);
        };
    }, []);

    // ─── Scheduled Audio Playback ────────────────────────────────────────────
    // Supports raw PCM16 (`audio/pcm;rate=24000` from Gemini Live) as well as container audio.
    const play_audio_chunk = useCallback((base64: string, mime_type?: string) => {
        const ctx = audio_context_ref.current;
        if (!ctx) return;

        if (ctx.state === 'suspended') {
            ctx.resume().catch(e => console.error('[Live_Voice_Hub] Audio Resume Error:', e));
        }

        const bytes = Uint8Array.from(atob(base64), c => c.charCodeAt(0));
        const schedule_buffer = (buffer: AudioBuffer) => {
            const source = ctx.createBufferSource();
            source.buffer = buffer;
            source.connect(ctx.destination);
            const start_time = Math.max(ctx.currentTime, next_play_time_ref.current);
            source.start(start_time);
            next_play_time_ref.current = start_time + buffer.duration;
        };

        const is_raw_pcm = Boolean(mime_type && mime_type.toLowerCase().startsWith('audio/pcm'));
        if (is_raw_pcm && typeof ctx.createBuffer === 'function') {
            try {
                const sample_count = Math.floor(bytes.byteLength / 2);
                if (sample_count <= 0) return;
                const sample_rate = parse_pcm_sample_rate(mime_type);
                const pcm16 = new Int16Array(bytes.buffer, bytes.byteOffset, sample_count);
                const float32 = new Float32Array(sample_count);
                for (let i = 0; i < sample_count; i++) {
                    float32[i] = pcm16[i] / 32768.0;
                }
                const buffer = ctx.createBuffer(1, sample_count, sample_rate);
                buffer.getChannelData(0).set(float32);
                schedule_buffer(buffer);
                return;
            } catch (e) {
                console.error('[Live_Voice_Hub] Raw PCM playback error:', e);
                return;
            }
        }

        ctx.decodeAudioData(bytes.buffer.slice(0), schedule_buffer)
            .catch(e => console.error('[Live_Voice_Hub] Audio Decode Error:', e));
    }, []);

    const stop_mic = useCallback(() => {
        mic_generation.current++;
        stream_ref.current?.getTracks().forEach(t => t.stop());
        stream_ref.current = null;
        processor_ref.current?.disconnect();
        processor_ref.current = null;
        sink_gain_ref.current?.disconnect();
        sink_gain_ref.current = null;
        void audio_context_ref.current?.close();
        audio_context_ref.current = null;
        next_play_time_ref.current = 0;
        volume_ref.current = 0;
        voice_client.release_mic_lease?.(LIVE_MIC_LEASE_OWNER);
        set_active(false);
    }, []);

    // ─── WebSocket Lifecycle ─────────────────────────────────────────────────
    useEffect(() => {
        let disposed = false;
        let ws: WebSocket | null = null;
        void Promise.resolve().then(() => {
            if (disposed) return;
            set_connected(false);
            set_connection_error(null);
            const token = engine_token.trim();
            if (!token) {
                set_connection_error('Add your engine token in Settings to connect voice.');
                return;
            }
            try {
                const base_url = validate_and_sanitize_url(engine_url);
                const url = new URL(`${base_url}/v1/engine/live-voice`);
                url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
                if (!is_allowed_origin(url.toString(), allowed_origins, is_private_network_origin(url.hostname))) {
                    throw new Error('Voice engine origin is not allowed. Check Settings.');
                }
                const socket = new WebSocket(url.toString(), [`bearer.${token}`]);
                ws = socket;
                ws_ref.current = socket;
                socket.onopen = () => {
                    if (disposed) return;
                    set_connected(true);
                    socket.send(JSON.stringify({ setup: {
                        model: 'models/gemini-3-flash-preview',
                        generation_config: { response_modalities: ['audio'] },
                        agent_id,
                    } }));
                };
                socket.onmessage = (event) => {
                    if (disposed) return;
                    if (typeof event.data === 'string' && event.data.startsWith('Error:')) {
                        set_connection_error(event.data);
                        return;
                    }
                    try {
                        const data = JSON.parse(event.data as string);
                        const inline_data = data.serverContent?.modelTurn?.parts?.[0]?.inlineData;
                        if (inline_data?.data) {
                            play_audio_chunk(inline_data.data, inline_data.mimeType || inline_data.mime_type);
                        }
                    } catch (e) {
                        console.error('[Live_Voice_Hub] WS message parse error:', e);
                    }
                };
                socket.onclose = () => {
                    if (disposed) return;
                    stop_mic();
                    set_connected(false);
                    set_connection_error('Voice connection closed. Check your engine connection and token in Settings.');
                };
                socket.onerror = () => {
                    if (!disposed) set_connection_error('Unable to connect voice. Check your engine connection and token in Settings.');
                };
            } catch (error) {
                set_connection_error(error instanceof Error ? error.message : 'Unable to connect voice.');
            }
        });
        return () => {
            disposed = true;
            stop_mic();
            ws?.close();
            if (ws_ref.current === ws) ws_ref.current = null;
        };
    }, [agent_id, engine_url, engine_token, allowed_origins, play_audio_chunk, stop_mic]);

    // ─── Mic Capture ────────────────────────────────────────────────────────
    const start_mic = useCallback(async () => {
        if (mic_pending.current || ws_ref.current?.readyState !== WebSocket.OPEN) return;

        if (typeof window !== 'undefined' && window.isSecureContext === false) {
            set_connection_error(classify_mic_error(new Error('insecure context')).message);
            return;
        }
        if (typeof navigator === 'undefined' || !navigator.mediaDevices?.getUserMedia) {
            set_connection_error(classify_mic_error(new TypeError('navigator.mediaDevices.getUserMedia is unavailable')).message);
            return;
        }
        if (typeof voice_client.acquire_mic_lease === 'function' && !voice_client.acquire_mic_lease(LIVE_MIC_LEASE_OWNER)) {
            set_connection_error(voice_client.get_last_mic_error?.()?.message || 'Microphone is already in use.');
            return;
        }

        mic_pending.current = true;
        set_connection_error(null);
        const generation = mic_generation.current;
        try {
            const preferred_device = voice_client.get_input_device?.();
            const audio_constraints: boolean | MediaTrackConstraints = preferred_device
                ? { deviceId: { ideal: preferred_device } }
                : true;
            const stream = await navigator.mediaDevices.getUserMedia({ audio: audio_constraints });
            if (generation !== mic_generation.current || ws_ref.current?.readyState !== WebSocket.OPEN) {
                stream.getTracks().forEach(t => t.stop());
                voice_client.release_mic_lease?.(LIVE_MIC_LEASE_OWNER);
                return;
            }
            stream_ref.current = stream;

            const AudioContextClass = window.AudioContext
                || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
            const ctx = new AudioContextClass({ sampleRate: 16000 });
            audio_context_ref.current = ctx;

            const source = ctx.createMediaStreamSource(stream);
            const processor = ctx.createScriptProcessor(4096, 1, 1);
            processor_ref.current = processor;

            processor.onaudioprocess = (e) => {
                const input_data = e.inputBuffer.getChannelData(0);

                let sum = 0;
                for (let i = 0; i < input_data.length; i++) sum += Math.abs(input_data[i]);
                volume_ref.current = sum / input_data.length;

                if (ws_ref.current?.readyState === WebSocket.OPEN) {
                    const pcm_data = convert_float32_to_pcm16(input_data);
                    const raw = new Uint8Array(pcm_data.buffer);
                    let binary = '';
                    for (let i = 0; i < raw.length; i++) binary += String.fromCharCode(raw[i]);
                    ws_ref.current.send(JSON.stringify({
                        realtime_input: {
                            media_chunks: [{ mime_type: "audio/pcm;rate=16000", data: btoa(binary) }]
                        }
                    }));
                }
            };

            source.connect(processor);
            // Route through a muted GainNode (gain = 0) when available so mic audio never echoes out of speakers
            if (typeof ctx.createGain === 'function') {
                const silent_sink = ctx.createGain();
                silent_sink.gain.value = 0;
                sink_gain_ref.current = silent_sink;
                processor.connect(silent_sink);
                silent_sink.connect(ctx.destination);
            } else {
                processor.connect(ctx.destination);
            }
            set_active(true);
        } catch (err) {
            stop_mic();
            const classified = classify_mic_error(err);
            set_connection_error(classified.message);
            console.error('[Live_Voice_Hub] Failed to start mic:', err);
        } finally {
            mic_pending.current = false;
        }
    }, [stop_mic]);

    const toggle_mic = useCallback(async () => {
        if (active) stop_mic();
        else await start_mic();
    }, [active, stop_mic, start_mic]);

    return (
        <div
            className="fixed bottom-6 right-6 w-80 bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-2xl shadow-2xl overflow-hidden"
            style={{ zIndex: Z_INDEX_MAP.toast - 1 }}
        >
            {/* Header */}
            <div className="px-4 py-3 bg-[color:var(--color-surface)]/50 border-b border-[color:var(--color-border)] flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <div className={`w-2 h-2 rounded-full ${connected ? 'bg-emerald-500 animate-pulse' : 'bg-red-500'}`} />
                    <span className="text-[10px] font-bold text-zinc-400 uppercase tracking-wider">{i18n.t('voice.service_gemini')}</span>
                </div>
                <button
                    onClick={on_close}
                    aria-label={i18n.t('voice.close')}
                    className="text-zinc-500 hover:text-white transition-colors"
                >
                    <X size={14} />
                </button>
            </div>

            {/* Visualizer — bars driven by RAF, not React state */}
            <div className="h-40 flex items-center justify-center relative bg-zinc-950/20">
                <div className="flex items-center gap-1">
                    {visualizer_offsets.map((_, idx) => (
                        <div
                            key={idx}
                            ref={el => { bar_refs.current[idx] = el; }}
                            className="w-1.5 rounded-full transition-all duration-75"
                            style={{
                                height: '4px',
                                background: theme_color,
                                opacity: '0.2',
                            }}
                        />
                    ))}
                </div>
                {active && (
                    <div className="absolute inset-0 pointer-events-none">
                        <Waves className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-zinc-900 w-32 h-32 opacity-10 animate-pulse" />
                    </div>
                )}
            </div>

            {/* Controls */}
            <div className="p-6 flex flex-col items-center gap-4">
                <button
                    onClick={toggle_mic}
                    aria-label={active ? i18n.t('voice.stop') : i18n.t('voice.start')}
                    aria-pressed={active}
                    disabled={!connected}
                    className={`w-16 h-16 rounded-full flex items-center justify-center transition-all duration-300 ${
                        active ? 'scale-110 shadow-[0_0_30px_rgba(0,0,0,0.5)]' : 'hover:scale-105'
                    }`}
                    style={{
                        background: active ? theme_color : '#18181b',
                        boxShadow: active ? `0 0 20px ${theme_color}40` : 'none'
                    }}
                >
                    {active ? <Mic size={28} className="text-white" /> : <MicOff size={28} className="text-zinc-500" />}
                </button>

                <p className="text-[11px] font-medium text-zinc-500 text-center px-4">
                    {active ? i18n.t('voice.listening') : i18n.t('voice.tap_to_start')}
                </p>
                {connection_error && <p role="alert" className="text-xs text-red-400 text-center">{connection_error}</p>}

                {/* Live volume meter — width driven by RAF loop via volume_bar_ref */}
                <div className="flex items-center gap-4 text-zinc-600">
                    <Volume2 size={16} />
                    <div className="w-24 h-1 bg-[color:var(--color-surface)] rounded-full overflow-hidden">
                        <div
                            ref={volume_bar_ref}
                            className="h-full rounded-full transition-none"
                            style={{ width: '0%', background: theme_color }}
                        />
                    </div>
                </div>
            </div>
        </div>
    );
}
