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

/**
 * Voice_Client
 * Orchestrates local and cloud-based speech synthesis and recognition.
 * Refactored for strict snake_case compliance for backend parity.
 */
export class Voice_Client {
    private recognition: Speech_Recognition_Interface | null = null;
    private is_listening = false;
    private on_transcript_callback: ((text: string) => void) | null = null;
    private media_recorder: MediaRecorder | null = null;
    private audio_chunks: Blob[] = [];
    private audio_context: AudioContext | null = null;
    private next_start_time = 0;

    constructor() {
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
                if (transcript && this.on_transcript_callback) {
                    this.on_transcript_callback(transcript);
                }
            };

            this.recognition.onend = () => {
                if (this.is_listening && this.recognition) {
                    this.recognition.start(); // Keep listening if we're supposed to be
                }
            };

            this.recognition.onerror = (event) => {
                console.error('[VoiceClient] Recognition error:', event.error);
                if (event.error === 'not-allowed') {
                    this.is_listening = false;
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

    start_listening(callback: (text: string) => void) {
        if (!this.recognition) return;
        this.on_transcript_callback = callback;
        this.is_listening = true;
        try {
            this.recognition.start();
        } catch {
            // Silently fail if already started
        }
    }

    stop_listening() {
        this.is_listening = false;
        if (this.recognition) {
            this.recognition.stop();
        }
    }

    async start_recording() {
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

    async speak(text: string, voice_id?: string, engine: 'browser' | 'openai' | 'groq' | 'piper' | 'gemini-live' = 'browser') {
        if (!text) return;

        // Cancel any ongoing speech
        if (window.speechSynthesis) {
            window.speechSynthesis.cancel();
        }

        if (engine === 'browser') {
            this.speak_browser(text, voice_id);
        } else {
            await this.speak_premium(text, voice_id, engine);
        }
    }

    private speak_browser(text: string, preferred_voice_name?: string) {
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

    private async handle_audio_chunk(chunk: ArrayBuffer) {
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
}

export const voice_client = new Voice_Client();

