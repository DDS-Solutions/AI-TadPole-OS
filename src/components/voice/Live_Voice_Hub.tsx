/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Real-time auditory Command & Control (C2) hub. 
 * Orchestrates voice client initialization (Azure/Groq), mission-aware transcript generation, and bi-directional swarm audio streams.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Client initialization stall (missing API key), Websocket audio buffer starvation, or audio context suspension by browser policy.
 * - **Telemetry Link**: Search for `[Live_Voice_Hub]` or `VOICE_SYNC` in service logs.
 * 
 * ### ⚡ Architecture Notes
 * - AudioContext is forced to 16kHz to match Gemini's expected PCM format. Without this,
 *   hardware defaults (44.1kHz / 48kHz) produce "chipmunk" or "slow-mo" audio on the model side.
 * - Playback uses a look-ahead scheduler (next_play_time_ref) to stitch audio chunks without
 *   pops or overlapping. Each chunk starts exactly when the previous one ends.
 * - Visualizer uses a ref + requestAnimationFrame to avoid triggering React reconciliation
 *   at 40Hz (every onaudioprocess event).
 * - ScriptProcessorNode is deprecated but retained for broad browser compatibility.
 *   Future improvement: migrate to AudioWorklet for off-main-thread PCM processing.
 * - Known race: closing the AudioContext while decodeAudioData promises are in-flight will
 *   cause those promises to reject. The .catch handler logs the error; no audio is lost
 *   because stop_mic is only called on user intent or unmount.
 */

import { useEffect, useRef, useState, useCallback } from 'react'; // useCallback kept for play_audio_chunk / stop_mic / start_mic / toggle_mic
import { Mic, MicOff, Waves, Volume2, X } from 'lucide-react';
import { i18n } from '../../i18n';
import { Z_INDEX_MAP } from '../ui/theme_tokens';

// ─── Module-level Constants ──────────────────────────────────────────────────

/** Max positive value for a signed 16-bit integer. Used for PCM encoding. */
const PCM16_MAX = 0x7FFF; // 32767

// ─── Module-level Pure Utilities ────────────────────────────────────────────
// Defined outside the component so they are never recreated on render and
// never captured as a stale closure inside onaudioprocess callbacks.

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

// ─── Types ───────────────────────────────────────────────────────────────────

/**
 * Live_Voice_Hub_Props
 * Defines the interface for the Live_Voice_Hub component.
 * Refactored for strict snake_case compliance for backend parity.
 */
interface Live_Voice_Hub_Props {
    agent_id: string;
    /**
     * Accent color for the mic button glow and visualizer bars.
     * Must be a 6-digit hex string (e.g. `#10b981`) — the component appends
     * a 2-digit hex alpha suffix for shadow generation (`${theme_color}40`).
     * CSS color names or rgb() values will produce an invalid box-shadow.
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

    // Refs for audio infrastructure — never stored in state to avoid re-renders
    const ws_ref = useRef<WebSocket | null>(null);
    const audio_context_ref = useRef<AudioContext | null>(null);
    const stream_ref = useRef<MediaStream | null>(null);
    const processor_ref = useRef<ScriptProcessorNode | null>(null);

    // Scheduler ref: tracks when the next audio chunk should begin playing
    const next_play_time_ref = useRef<number>(0);

    // Visualizer: volume is tracked in a ref + RAF to avoid React re-renders at 40Hz
    const volume_ref = useRef<number>(0);
    const bar_refs = useRef<(HTMLDivElement | null)[]>([]);
    const volume_bar_ref = useRef<HTMLDivElement | null>(null);
    const raf_ref = useRef<number | null>(null);
    const [visualizer_offsets] = useState(() => Array.from({ length: 8 }, () => Math.random()));

    // ─── Visualizer & Volume Bar Animation Loop ──────────────────────────────
    // Drives bar heights and the live volume meter directly via DOM to bypass
    // React reconciliation. Both are updated in the same RAF tick.
    //
    // Pattern: store the latest frame logic in a ref so the RAF loop always
    // calls the current version without creating a self-referential useCallback
    // (which triggers react-hooks/immutability — access before declaration).
    // The ref is updated in useLayoutEffect (not during render) to satisfy
    // react-hooks/refs which forbids writing .current in the render body.
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
    }); // no dep array — runs every render to keep ref current

    useEffect(() => {
        // The outer RAF trampoline calls the ref (always latest) — no stale closure.
        const tick = () => {
            visualizer_frame_ref.current?.();
            raf_ref.current = requestAnimationFrame(tick);
        };
        raf_ref.current = requestAnimationFrame(tick);
        return () => {
            if (raf_ref.current !== null) cancelAnimationFrame(raf_ref.current);
        };
    }, []); // intentionally empty — tick trampoline reads from ref, not closure

    // ─── Scheduled Audio Playback ────────────────────────────────────────────
    // Uses audio_context.currentTime look-ahead to stitch chunks without pops.
    const play_audio_chunk = useCallback((base64: string) => {
        const ctx = audio_context_ref.current;
        if (!ctx) return;

        // Resume context if browser policy has suspended it
        if (ctx.state === 'suspended') ctx.resume();

        const bytes = Uint8Array.from(atob(base64), c => c.charCodeAt(0));

        ctx.decodeAudioData(bytes.buffer.slice(0), (buffer) => {
            const source = ctx.createBufferSource();
            source.buffer = buffer;
            source.connect(ctx.destination);

            // Schedule the chunk to start exactly when the previous one ends
            const start_time = Math.max(ctx.currentTime, next_play_time_ref.current);
            source.start(start_time);
            next_play_time_ref.current = start_time + buffer.duration;
        }).catch(e => console.error('[Live_Voice_Hub] Audio Decode Error:', e));
    }, []);

    // ─── WebSocket Lifecycle ─────────────────────────────────────────────────
    useEffect(() => {
        let token: string | null = null;
        try {
            token = localStorage.getItem('tadpole_token');
        } catch {
            // localStorage is blocked (e.g. Safari private mode, strict CSP)
        }
        const protocol = token ? `bearer.${token}` : 'bearer.anonymous';

        const ws_url = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/v1/engine/live-voice`;
        const ws = new WebSocket(ws_url, [protocol]);
        ws_ref.current = ws;

        ws.onopen = () => {
            set_connected(true);
            ws.send(JSON.stringify({
                setup: {
                    model: "models/gemini-3-flash-preview",
                    generation_config: { response_modalities: ["audio"] },
                    agent_id,
                }
            }));
        };

        ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data as string);
                const inline_data = data.serverContent?.modelTurn?.parts?.[0]?.inlineData;
                if (inline_data?.data) play_audio_chunk(inline_data.data);
            } catch (e) {
                console.error('[Live_Voice_Hub] WS message parse error:', e);
            }
        };

        ws.onclose = () => set_connected(false);
        ws.onerror = (e) => console.error('[Live_Voice_Hub] WS Error:', e);

        return () => ws.close();
    }, [agent_id, play_audio_chunk]);

    // ─── Mic Teardown (Comprehensive Cleanup) ───────────────────────────────
    const stop_mic = useCallback(() => {
        // Stop all media tracks to release the hardware mic indicator
        stream_ref.current?.getTracks().forEach(t => t.stop());
        stream_ref.current = null;

        processor_ref.current?.disconnect();
        processor_ref.current = null;

        audio_context_ref.current?.close();
        audio_context_ref.current = null;

        next_play_time_ref.current = 0;
        volume_ref.current = 0;
        set_active(false);
    }, []);

    // Ensure cleanup on unmount (guards against navigation mid-session).
    // stop_mic is stable (useCallback with [] deps) so this effect is correct.
    useEffect(() => {
        return stop_mic;
    }, [stop_mic]);

    // ─── Mic Capture ────────────────────────────────────────────────────────
    const start_mic = useCallback(async () => {
        try {
            const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
            stream_ref.current = stream;

            // Force 16kHz so PCM data matches the mime_type sent to the backend.
            // Without this, hardware defaults (44.1kHz / 48kHz) produce pitch-shifted audio.
            const AudioContextClass = window.AudioContext
                || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
            const ctx = new AudioContextClass({ sampleRate: 16000 });
            audio_context_ref.current = ctx;

            const source = ctx.createMediaStreamSource(stream);
            // Buffer size 4096 @ 16kHz ≈ 256ms — acceptable latency for voice
            const processor = ctx.createScriptProcessor(4096, 1, 1);
            processor_ref.current = processor;

            processor.onaudioprocess = (e) => {
                const input_data = e.inputBuffer.getChannelData(0);

                // Update volume ref (not state) to feed the RAF visualizer loop
                let sum = 0;
                for (let i = 0; i < input_data.length; i++) sum += Math.abs(input_data[i]);
                volume_ref.current = sum / input_data.length;

                // Encode and transmit PCM 16-bit chunk
                if (ws_ref.current?.readyState === WebSocket.OPEN) {
                    const pcm_data = convert_float32_to_pcm16(input_data);
                    // Use spread-safe encoding for large buffers
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
            processor.connect(ctx.destination);
            set_active(true);
        } catch (err) {
            console.error('[Live_Voice_Hub] Failed to start mic:', err);
        }
    }, []);

    const toggle_mic = useCallback(async () => {
        if (active) stop_mic();
        else await start_mic();
    }, [active, stop_mic, start_mic]);

    return (
        <div
            className="fixed bottom-6 right-6 w-80 bg-zinc-950 border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden"
            style={{ zIndex: Z_INDEX_MAP.toast - 1 }}
        >
            {/* Header */}
            <div className="px-4 py-3 bg-zinc-900/50 border-b border-zinc-800 flex items-center justify-between">
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
            <div className="h-40 flex items-center justify-center relative bg-black/20">
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

                {/* Live volume meter — width driven by RAF loop via volume_bar_ref */}
                <div className="flex items-center gap-4 text-zinc-600">
                    <Volume2 size={16} />
                    <div className="w-24 h-1 bg-zinc-900 rounded-full overflow-hidden">
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


// Metadata: [Live_Voice_Hub]
