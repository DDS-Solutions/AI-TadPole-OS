/**
 * @docs ARCHITECTURE:AudioIntelligence
 * 
 * ### AI Assist Note
 * **Audio Service**: Real-time voice synthesis and transcription orchestrator. 
 * Manages the streaming of binary audio chunks via WebSocket and orchestrates the integration with the `Live_Voice_Hub`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: WebSocket buffer underflow (audio stuttering), transcription latency spike, or sample-rate mismatch.
 * - **Telemetry Link**: Search for `[VoiceClient]` or `audio_stream_chunk` in UI tracing.
 * 
 * @aiContext
 * - **Dependencies**: `socket.ts` (audio sync), `tadpoleos_service.ts` (TTS).
 * - **Side Effects**: Requires Microhone permission. Accesses `AudioContext` and `SpeechRecognition` hardware.
 * - **Mocking**: Mock `window.SpeechRecognition` and `AudioContext` for unit tests.
 */


import { tadpole_os_service } from './tadpoleos_service';
import { tadpole_os_socket } from './socket';

/** Minimal typed shape for a single SpeechRecognitionAlternative. */
interface Speech_Alternative {
    readonly transcript: string;
}

/** Minimal typed shape for a SpeechRecognitionResult. */
interface Speech_Result {
    readonly length: number;
    [index: number]: Speech_Alternative;
}

/** Minimal typed shape for SpeechRecognitionResultList. */
interface Speech_Result_List {
    readonly length: number;
    [index: number]: Speech_Result;
}

/** Cross-browser SpeechRecognition interface abstraction. */
interface Speech_Recognition_Interface {
    continuous: boolean;
    interimResults: boolean;
    lang: string;
    onresult: (event: { results: Speech_Result_List }) => void;
    onerror: (event: { error: string }) => void;
    onend: () => void;
    start: () => void;
    stop: () => void;
}

/** Voice_Status: Current state of the voice recognition/synthesis system. */
export type Voice_Status = 'idle' | 'initializing' | 'active' | 'stalled';

/**
 * Voice_Client
 * Orchestrates local and cloud-based speech synthesis and recognition.
 * Refactored for strict snake_case compliance for backend parity.
 */
export class Voice_Client {
    private recognition: Speech_Recognition_Interface | null = null;
    private is_listening = false;
    private on_transcript_callback: ((text: string) => void) | null = null;
    private on_status_callback: ((status: Voice_Status) => void) | null = null;
    private status: Voice_Status = 'idle';
    private stall_timer: ReturnType<typeof setTimeout> | null = null;
    
    private media_recorder: MediaRecorder | null = null;
    private audio_chunks: Blob[] = [];
    private audio_context: AudioContext | null = null;
    private next_start_time = 0;
    private backend_features: string[] = [];

    constructor() {
        this.init_feature_detection();
        const win = window as unknown as {
            SpeechRecognition?: new () => Speech_Recognition_Interface,
            webkitSpeechRecognition?: new () => Speech_Recognition_Interface
        };
        const SpeechRecognitionClass = win.SpeechRecognition || win.webkitSpeechRecognition;
        if (SpeechRecognitionClass) {
            this.recognition = new SpeechRecognitionClass();
            this.recognition.continuous = true;
            this.recognition.interimResults = false; // Set to false to reduce flicker/overhead
            this.recognition.lang = 'en-US';

            this.recognition.onresult = (event) => {
                const results = event.results;
                const transcript = results[results.length - 1][0].transcript.trim();
                
                // Clear stall timer on first valid result
                if (this.status === 'initializing' || this.status === 'stalled') {
                    if (this.stall_timer) {
                        clearTimeout(this.stall_timer);
                        this.stall_timer = null;
                    }
                    this.set_status('active');
                }

                if (transcript && this.on_transcript_callback) {
                    this.on_transcript_callback(transcript);
                }
            };

            this.recognition.onend = () => {
                if (this.is_listening && this.recognition) {
                    this.recognition.start(); // Keep listening if we're supposed to be
                } else if (!this.is_listening) {
                    this.set_status('idle');
                }
            };

            this.recognition.onerror = (event) => {
                console.error('[VoiceClient] Recognition error:', event.error);
                if (event.error === 'not-allowed') {
                    this.is_listening = false;
                    this.set_status('idle');
                } else {
                    this.set_status('stalled');
                }
            };
        } else {
            console.warn('[VoiceClient] Web Speech API not supported in this browser.');
        }

        // Initialize streaming audio handler
        tadpole_os_socket.subscribe_audio_stream((chunk) => {
            this.handle_audio_chunk(chunk);
        });
    }

    public get_status = (): Voice_Status => {
        return this.status;
    };

    public on_status_change = (callback: (status: Voice_Status) => void): void => {
        this.on_status_callback = callback;
    };

    private set_status(status: Voice_Status): void {
        this.status = status;
        if (this.on_status_callback) {
            this.on_status_callback(status);
        }
    }

    start_listening(callback: (text: string) => void): void {
        if (!this.recognition) {
            console.error('[VoiceClient] Recognition engine unavailable.');
            return;
        }

        console.debug('[VoiceClient] Initializing voice link...');
        this.on_transcript_callback = callback;
        this.is_listening = true;
        this.set_status('initializing');

        // Start 5s stall detection timer
        if (this.stall_timer) clearTimeout(this.stall_timer);
        this.stall_timer = setTimeout(() => {
            if (this.status === 'initializing') {
                console.warn('[VoiceClient] Voice link stalled: No signal detected after 5s.');
                this.set_status('stalled');
            }
        }, 5000);

        try {
            this.recognition.start();
        } catch {
            // Silently fail if already started
        }
    }

    stop_listening(): void {
        console.debug('[VoiceClient] Terminating voice link.');
        this.is_listening = false;
        this.set_status('idle');
        if (this.stall_timer) {
            clearTimeout(this.stall_timer);
            this.stall_timer = null;
        }
        if (this.recognition) {
            this.recognition.stop();
        }
    }

    async start_recording(): Promise<void> {
        try {
            const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
            this.media_recorder = new MediaRecorder(stream);
            this.audio_chunks = [];

            this.media_recorder.ondataavailable = (event) => {
                this.audio_chunks.push(event.data);
            };

            this.media_recorder.start();
        } catch (err) {
            console.error('[VoiceClient] Failed to start recording:', err);
        }
    }

    async stop_recording(): Promise<Blob | null> {
        return new Promise((resolve) => {
            if (!this.media_recorder) return resolve(null);

            this.media_recorder.onstop = () => {
                const audio_blob = new Blob(this.audio_chunks, { type: 'audio/wav' });
                resolve(audio_blob);
            };

            this.media_recorder.stop();
            this.media_recorder.stream.getTracks().forEach(track => track.stop());
            this.media_recorder = null;
        });
    }

    async speak(text: string, voice_id?: string, engine: 'browser' | 'openai' | 'groq' | 'piper' | 'gemini-live' = 'browser'): Promise<void> {
        if (!text) return;

        // Cancel any ongoing speech
        if (window.speechSynthesis) {
            window.speechSynthesis.cancel();
        }

        const synthesis_engine = this.backend_features.includes('neural-audio') ? engine : 'browser';

        if (synthesis_engine === 'browser') {
            this.speak_browser(text, voice_id);
        } else {
            await this.speak_premium(text, voice_id, synthesis_engine);
        }
    }

    private speak_browser(text: string, preferred_voice_name?: string): void {
        if (!window.speechSynthesis) return;

        const utterance = new SpeechSynthesisUtterance(text);
        utterance.rate = 1.0;
        utterance.pitch = 1.0;
        utterance.volume = 1.0;

        // Try to find the preferred voice or a nice default
        const voices = window.speechSynthesis.getVoices();
        const voice = voices.find(v => v.name === preferred_voice_name) ||
            voices.find(v => v.name.includes('Google') || v.name.includes('Premium') || v.name.includes('Samantha'));

        if (voice) {
            utterance.voice = voice;
        }

        window.speechSynthesis.speak(utterance);
    }

    private async speak_premium(text: string, voice_id?: string, engine: string = 'openai') {
        try {
            const audio_blob = await tadpole_os_service.speak(text, voice_id, engine);
            const audio_url = URL.createObjectURL(audio_blob);
            const audio = new Audio(audio_url);
            await audio.play();

            // Cleanup URL after playing
            audio.onended = () => URL.revokeObjectURL(audio_url);
        } catch (err) {
            console.error('[VoiceClient] Premium TTS error:', err);
            this.speak_browser(text);
        }
    }

    private async handle_audio_chunk(chunk: ArrayBuffer): Promise<void> {
        if (!this.audio_context) {
            const win = window as unknown as { AudioContext: typeof AudioContext; webkitAudioContext: typeof AudioContext };
            const AudioContextClass = win.AudioContext || win.webkitAudioContext;
            this.audio_context = new AudioContextClass();
            this.next_start_time = this.audio_context.currentTime;
        }

        if (this.audio_context.state === 'suspended') {
            await this.audio_context.resume();
        }

        try {
            // Assume 16-bit PCM at 22050Hz (standard for Piper)
            const int16_array = new Int16Array(chunk);
            const float32_array = new Float32Array(int16_array.length);
            for (let i = 0; i < int16_array.length; i++) {
                float32_array[i] = int16_array[i] / 32768.0;
            }

            const buffer = this.audio_context.createBuffer(1, float32_array.length, 22050);
            buffer.getChannelData(0).set(float32_array);

            const source = this.audio_context.createBufferSource();
            source.buffer = buffer;
            source.connect(this.audio_context.destination);

            const start_time = Math.max(this.audio_context.currentTime, this.next_start_time);
            source.start(start_time);
            this.next_start_time = start_time + buffer.duration;
        } catch (err) {
            console.error('[VoiceClient] Chunk playback failed:', err);
        }
    }

    private async init_feature_detection(): Promise<void> {
        try {
            const { system_api_service } = await import('./system_api_service');
            const status = await system_api_service.get_engine_status();
            this.backend_features = status?.features || [];
            
            if (!this.backend_features.includes('neural-audio')) {
                console.debug('[VoiceClient] Backend "neural-audio" feature missing. Defaulting to local browser synthesis.');
            }
        } catch {
            // Fallback to empty if engine is unreachable
        }
    }
}

export const voice_client = new Voice_Client();


// Metadata: [voice_client]

// Metadata: [voice_client]
