import { useEffect, useRef, useState } from 'react';
import { Mic, MicOff, Waves, Volume2, Settings2 } from 'lucide-react';
import { i18n } from '../../i18n';

interface LiveVoiceHubProps {
    agentId: string;
    themeColor: string;
    onClose?: () => void;
}

export function LiveVoiceHub({ agentId, themeColor, onClose }: LiveVoiceHubProps) {
    const [connected, setConnected] = useState(false);
    const [active, setActive] = useState(false);
    const [volume, setVolume] = useState(0);
    const wsRef = useRef<WebSocket | null>(null);
    const audioContextRef = useRef<AudioContext | null>(null);
    const streamRef = useRef<MediaStream | null>(null);
    const processorRef = useRef<ScriptProcessorNode | null>(null);

    const playAudioChunk = (base64: string) => {
        if (!audioContextRef.current) return;
        const binaryString = atob(base64);
        const bytes = new Uint8Array(binaryString.length);
        for (let i = 0; i < binaryString.length; i++) {
            bytes[i] = binaryString.charCodeAt(i);
        }
        
        // Playback logic (simplified: decode and play)
        // In a production app, use an AudioWorklet or a buffer queue
        audioContextRef.current.decodeAudioData(bytes.buffer, (buffer) => {
            const source = audioContextRef.current!.createBufferSource();
            source.buffer = buffer;
            source.connect(audioContextRef.current!.destination);
            source.start();
        }).catch(e => console.error("Audio Decode Error:", e));
    };

    // 1. WebSocket Setup
    useEffect(() => {
        const token = localStorage.getItem('tadpole_token');
        const protocol = token ? `bearer.${token}` : 'bearer.anonymous';
        
        // Connect to our local Sovereign Proxy
        const wsUrl = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/v1/engine/live-voice`;
        const ws = new WebSocket(wsUrl, [protocol]);
        wsRef.current = ws;

        ws.onopen = () => {
            setConnected(true);
            // Send initial setup
            ws.send(JSON.stringify({
                setup: {
                    model: "models/gemini-3-flash-preview",
                    generation_config: {
                        response_modalities: ["audio"]
                    },
                    agent_id: agentId
                }
            }));
        };

        ws.onmessage = async (event) => {
            const data = JSON.parse(event.data);
            if (data.serverContent?.modelTurn?.parts?.[0]?.inlineData) {
                const base64Audio = data.serverContent.modelTurn.parts[0].inlineData.data;
                playAudioChunk(base64Audio);
            }
        };

        ws.onclose = () => setConnected(false);
        ws.onerror = (e) => console.error("LiveVoice WS Error:", e);

        return () => ws.close();
    }, [agentId]);

    // 2. Audio Capture (Mic)
    const toggleMic = async () => {
        if (active) {
            stopMic();
        } else {
            await startMic();
        }
    };

    const startMic = async () => {
        try {
            const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
            streamRef.current = stream;
            
            const AudioContextClass = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
            const ctx = new AudioContextClass();
            audioContextRef.current = ctx;

            const source = ctx.createMediaStreamSource(stream);
            // Gemini expects 16kHz mono PCM
            const processor = ctx.createScriptProcessor(4096, 1, 1);
            processorRef.current = processor;

            processor.onaudioprocess = (e) => {
                const inputData = e.inputBuffer.getChannelData(0);
                // Simple downsampling/normalization for visualization
                let sum = 0;
                for (let i = 0; i < inputData.length; i++) {
                    sum += Math.abs(inputData[i]);
                }
                setVolume(sum / inputData.length);

                // Convert to PCM 16-bit and send if connected
                if (wsRef.current?.readyState === WebSocket.OPEN) {
                    const pcmData = convertFloat32ToPcm16(inputData);
                    const base64 = btoa(String.fromCharCode(...new Uint8Array(pcmData.buffer)));
                    wsRef.current.send(JSON.stringify({
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
            setActive(true);
        } catch (err) {
            console.error("Failed to start mic:", err);
        }
    };

    const stopMic = () => {
        streamRef.current?.getTracks().forEach(t => t.stop());
        processorRef.current?.disconnect();
        audioContextRef.current?.close();
        setActive(false);
        setVolume(0);
    };

    const convertFloat32ToPcm16 = (buffer: Float32Array) => {
        let length = buffer.length;
        const buf = new Int16Array(length);
        while (length--) {
            buf[length] = Math.min(1, buffer[length]) * 0x7FFF;
        }
        return buf;
    };



    const [visualizerOffsets] = useState(() => Array.from({ length: 8 }, () => Math.random()));

    return (
        <div className="fixed bottom-6 right-6 w-80 bg-zinc-950 border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden z-50">
            {/* Header */}
            <div className="px-4 py-3 bg-zinc-900/50 border-b border-zinc-800 flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <div className={`w-2 h-2 rounded-full ${connected ? 'bg-emerald-500 animate-pulse' : 'bg-red-500'}`} />
                    <span className="text-[10px] font-bold text-zinc-400 uppercase tracking-wider">Gemini Live</span>
                </div>
                <button onClick={onClose} className="text-zinc-500 hover:text-white transition-colors">
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
                                height: active ? `${20 + visualizerOffsets[idx] * volume * 200}%` : '4px',
                                background: themeColor,
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
                    onClick={toggleMic}
                    className={`w-16 h-16 rounded-full flex items-center justify-center transition-all duration-300 ${
                        active ? 'scale-110 shadow-[0_0_30px_rgba(0,0,0,0.5)]' : 'hover:scale-105'
                    }`}
                    style={{ 
                        background: active ? themeColor : '#18181b',
                        boxShadow: active ? `0 0 20px ${themeColor}40` : 'none'
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
