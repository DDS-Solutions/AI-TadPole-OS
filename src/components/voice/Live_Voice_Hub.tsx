import { useEffect, useRef, useState } from 'react';
import { Mic, MicOff, Waves, Volume2, Settings2 } from 'lucide-react';
import { i18n } from '../../i18n';

/**
 * Live_Voice_Hub_Props
 * Defines the interface for the Live_Voice_Hub component.
 * Refactored for strict snake_case compliance for backend parity.
 */
interface Live_Voice_Hub_Props {
    agent_id: string;
    theme_color: string;
    on_close?: () => void;
}

/**
 * Live_Voice_Hub
 * A specialized voice communication overlay for real-time Gemini Live sessions.
 * Manages WebSocket streams, audio processing, and high-fidelity visual feedback.
 */
export function Live_Voice_Hub({ agent_id, theme_color, on_close }: Live_Voice_Hub_Props) {
    const [connected, set_connected] = useState(false);
    const [active, set_active] = useState(false);
    const [volume, set_volume] = useState(0);
    const ws_ref = useRef<WebSocket | null>(null);
    const audio_context_ref = useRef<AudioContext | null>(null);
    const stream_ref = useRef<MediaStream | null>(null);
    const processor_ref = useRef<ScriptProcessorNode | null>(null);

    const play_audio_chunk = (base64: string) => {
        if (!audio_context_ref.current) return;
        const binary_string = atob(base64);
        const bytes = new Uint8Array(binary_string.length);
        for (let i = 0; i < binary_string.length; i++) {
            bytes[i] = binary_string.charCodeAt(i);
        }
        
        // Playback logic (simplified: decode and play)
        audio_context_ref.current.decodeAudioData(bytes.buffer, (buffer) => {
            if (!audio_context_ref.current) return;
            const source = audio_context_ref.current.createBufferSource();
            source.buffer = buffer;
            source.connect(audio_context_ref.current.destination);
            source.start();
        }).catch(e => console.error("Audio Decode Error:", e));
    };

    // 1. WebSocket Setup
    useEffect(() => {
        const token = localStorage.getItem('tadpole_token');
        const protocol = token ? `bearer.${token}` : 'bearer.anonymous';
        
        // Connect to our local Sovereign Proxy
        const ws_url = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/v1/engine/live-voice`;
        const ws = new WebSocket(ws_url, [protocol]);
        ws_ref.current = ws;

        ws.onopen = () => {
            set_connected(true);
            // Send initial setup
            ws.send(JSON.stringify({
                setup: {
                    model: "models/gemini-3-flash-preview",
                    generation_config: {
                        response_modalities: ["audio"]
                    },
                    agent_id: agent_id
                }
            }));
        };

        ws.onmessage = async (event) => {
            const data = JSON.parse(event.data);
            if (data.serverContent?.modelTurn?.parts?.[0]?.inlineData) {
                const base64_audio = data.serverContent.modelTurn.parts[0].inlineData.data;
                play_audio_chunk(base64_audio);
            }
        };

        ws.onclose = () => set_connected(false);
        ws.onerror = (e) => console.error("LiveVoice WS Error:", e);

        return () => ws.close();
    }, [agent_id]);

    // 2. Audio Capture (Mic)
    const toggle_mic = async () => {
        if (active) {
            stop_mic();
        } else {
            await start_mic();
        }
    };

    const start_mic = async () => {
        try {
            const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
            stream_ref.current = stream;
            
            const AudioContextClass = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
            const ctx = new AudioContextClass();
            audio_context_ref.current = ctx;

            const source = ctx.createMediaStreamSource(stream);
            // Gemini expects 16kHz mono PCM
            const processor = ctx.createScriptProcessor(4096, 1, 1);
            processor_ref.current = processor;

            processor.onaudioprocess = (e) => {
                const input_data = e.inputBuffer.getChannelData(0);
                // Simple downsampling/normalization for visualization
                let sum = 0;
                for (let i = 0; i < input_data.length; i++) {
                    sum += Math.abs(input_data[i]);
                }
                set_volume(sum / input_data.length);

                // Convert to PCM 16-bit and send if connected
                if (ws_ref.current?.readyState === WebSocket.OPEN) {
                    const pcm_data = convert_float32_to_pcm16(input_data);
                    const base64 = btoa(String.fromCharCode(...new Uint8Array(pcm_data.buffer)));
                    ws_ref.current.send(JSON.stringify({
                        realtime_input: {
                            media_chunks: [{
                                mime_type: "audio/pcm;rate=16000",
                                data: base64
                            }]
                        }
                    }));
                }
            };

            source.connect(processor);
            processor.connect(ctx.destination);
            set_active(true);
        } catch (err) {
            console.error("Failed to start mic:", err);
        }
    };

    const stop_mic = () => {
        stream_ref.current?.getTracks().forEach(t => t.stop());
        processor_ref.current?.disconnect();
        audio_context_ref.current?.close();
        set_active(false);
        set_volume(0);
    };

    const convert_float32_to_pcm16 = (buffer: Float32Array) => {
        let length = buffer.length;
        const buf = new Int16Array(length);
        while (length--) {
            buf[length] = Math.min(1, buffer[length]) * 0x7FFF;
        }
        return buf;
    };

    const [visualizer_offsets] = useState(() => Array.from({ length: 8 }, () => Math.random()));

    return (
        <div className="fixed bottom-6 right-6 w-80 bg-zinc-950 border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden z-50">
            {/* Header */}
            <div className="px-4 py-3 bg-zinc-900/50 border-b border-zinc-800 flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <div className={`w-2 h-2 rounded-full ${connected ? 'bg-emerald-500 animate-pulse' : 'bg-red-500'}`} />
                    <span className="text-[10px] font-bold text-zinc-400 uppercase tracking-wider">Gemini Live</span>
                </div>
                <button onClick={on_close} className="text-zinc-500 hover:text-white transition-colors">
                    <Settings2 size={14} />
                </button>
            </div>

            {/* Visualizer Area */}
            <div className="h-40 flex items-center justify-center relative bg-black/20">
                <div className="flex items-center gap-1">
                    {[1, 2, 3, 4, 5, 6, 7, 8].map((i, idx) => (
                        <div 
                            key={i}
                            className="w-1.5 rounded-full transition-all duration-75"
                            style={{ 
                                height: active ? `${20 + visualizer_offsets[idx] * volume * 200}%` : '4px',
                                background: theme_color,
                                opacity: active ? 0.8 : 0.2
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

                <div className="flex items-center gap-4 text-zinc-600">
                    <Volume2 size={16} />
                    <div className="w-24 h-1 bg-zinc-900 rounded-full overflow-hidden">
                        <div className="h-full bg-zinc-600" style={{ width: '70%' }} />
                    </div>
                </div>
            </div>
        </div>
    );
}
