import { TadpoleOSService } from './tadpoleosService';
import { TadpoleOSSocket } from './socket';

/** Minimal typed shape for a single SpeechRecognitionAlternative. */
interface SpeechAlternative {
    readonly transcript: string;
}

/** Minimal typed shape for a SpeechRecognitionResult. */
interface SpeechResult {
    readonly length: number;
    [index: number]: SpeechAlternative;
}

/** Minimal typed shape for SpeechRecognitionResultList. */
interface SpeechResultList {
    readonly length: number;
    [index: number]: SpeechResult;
}

/** Cross-browser SpeechRecognition interface abstraction. */
interface SpeechRecognitionInterface {
    continuous: boolean;
    interimResults: boolean;
    lang: string;
    onresult: (event: { results: SpeechResultList }) => void;
    onerror: (event: { error: string }) => void;
    onend: () => void;
    start: () => void;
    stop: () => void;
}

export class VoiceClient {
    private recognition: SpeechRecognitionInterface | null = null;
    private isListening = false;
    private onTranscriptCallback: ((text: string) => void) | null = null;
    private mediaRecorder: MediaRecorder | null = null;
    private audioChunks: Blob[] = [];
    private audioContext: AudioContext | null = null;
    private nextStartTime = 0;

    constructor() {
        const win = window as unknown as {
            SpeechRecognition?: new () => SpeechRecognitionInterface,
            webkitSpeechRecognition?: new () => SpeechRecognitionInterface
        };
        const SpeechRecognition = win.SpeechRecognition || win.webkitSpeechRecognition;
        if (SpeechRecognition) {
            this.recognition = new SpeechRecognition();
            this.recognition.continuous = true;
            this.recognition.interimResults = false;
            this.recognition.lang = 'en-US';

            this.recognition.onresult = (event) => {
                const results = event.results;
                const transcript = results[results.length - 1][0].transcript.trim();
                if (transcript && this.onTranscriptCallback) {
                    this.onTranscriptCallback(transcript);
                }
            };

            this.recognition.onend = () => {
                if (this.isListening && this.recognition) {
                    this.recognition.start(); // Keep listening if we're supposed to be
                }
            };

            this.recognition.onerror = (event) => {
                console.error('[VoiceClient] Recognition error:', event.error);
                if (event.error === 'not-allowed') {
                    this.isListening = false;
                }
            };
        } else {
            console.warn('[VoiceClient] Web Speech API not supported in this browser.');
        }

        // Initialize streaming audio handler
        TadpoleOSSocket.subscribeAudioStream((chunk) => {
            this.handleAudioChunk(chunk);
        });
    }

    startListening(callback: (text: string) => void) {
        if (!this.recognition) return;
        this.onTranscriptCallback = callback;
        this.isListening = true;
        try {
            this.recognition.start();
        } catch {
            // Silently fail if already started
        }
    }

    stopListening() {
        this.isListening = false;
        if (this.recognition) {
            this.recognition.stop();
        }
    }

    async startRecording() {
        try {
            const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
            this.mediaRecorder = new MediaRecorder(stream);
            this.audioChunks = [];

            this.mediaRecorder.ondataavailable = (event) => {
                this.audioChunks.push(event.data);
            };

            this.mediaRecorder.start();
        } catch (err) {
            console.error('[VoiceClient] Failed to start recording:', err);
        }
    }

    async stopRecording(): Promise<Blob | null> {
        return new Promise((resolve) => {
            if (!this.mediaRecorder) return resolve(null);

            this.mediaRecorder.onstop = () => {
                const audioBlob = new Blob(this.audioChunks, { type: 'audio/wav' });
                resolve(audioBlob);
            };

            this.mediaRecorder.stop();
            this.mediaRecorder.stream.getTracks().forEach(track => track.stop());
            this.mediaRecorder = null;
        });
    }

    async speak(text: string, voiceId?: string, engine: 'browser' | 'openai' | 'groq' | 'piper' | 'gemini-live' = 'browser') {
        if (!text) return;

        // Cancel any ongoing speech
        if (window.speechSynthesis) {
            window.speechSynthesis.cancel();
        }

        if (engine === 'browser') {
            this.speakBrowser(text, voiceId);
        } else {
            await this.speakPremium(text, voiceId, engine);
        }
    }

    private speakBrowser(text: string, preferredVoiceName?: string) {
        if (!window.speechSynthesis) return;

        const utterance = new SpeechSynthesisUtterance(text);
        utterance.rate = 1.0;
        utterance.pitch = 1.0;
        utterance.volume = 1.0;

        // Try to find the preferred voice or a nice default
        const voices = window.speechSynthesis.getVoices();
        const voice = voices.find(v => v.name === preferredVoiceName) ||
            voices.find(v => v.name.includes('Google') || v.name.includes('Premium') || v.name.includes('Samantha'));

        if (voice) {
            utterance.voice = voice;
        }

        window.speechSynthesis.speak(utterance);
    }

    private async speakPremium(text: string, voiceId?: string, engine: string = 'openai') {
        try {
            const audioBlob = await TadpoleOSService.speak(text, voiceId, engine);
            const audioUrl = URL.createObjectURL(audioBlob);
            const audio = new Audio(audioUrl);
            await audio.play();

            // Cleanup URL after playing
            audio.onended = () => URL.revokeObjectURL(audioUrl);
        } catch (err) {
            console.error('[VoiceClient] Premium TTS error:', err);
            this.speakBrowser(text);
        }
    }

    private async handleAudioChunk(chunk: ArrayBuffer) {
        if (!this.audioContext) {
            const win = window as unknown as { AudioContext: typeof AudioContext; webkitAudioContext: typeof AudioContext };
            const AudioContextClass = win.AudioContext || win.webkitAudioContext;
            this.audioContext = new AudioContextClass();
            this.nextStartTime = this.audioContext.currentTime;
        }

        if (this.audioContext.state === 'suspended') {
            await this.audioContext.resume();
        }

        try {
            // Assume 16-bit PCM at 22050Hz (standard for Piper)
            const int16Array = new Int16Array(chunk);
            const float32Array = new Float32Array(int16Array.length);
            for (let i = 0; i < int16Array.length; i++) {
                float32Array[i] = int16Array[i] / 32768.0;
            }

            const buffer = this.audioContext.createBuffer(1, float32Array.length, 22050);
            buffer.getChannelData(0).set(float32Array);

            const source = this.audioContext.createBufferSource();
            source.buffer = buffer;
            source.connect(this.audioContext.destination);

            const startTime = Math.max(this.audioContext.currentTime, this.nextStartTime);
            source.start(startTime);
            this.nextStartTime = startTime + buffer.duration;
        } catch (err) {
            console.error('[VoiceClient] Chunk playback failed:', err);
        }
    }
}

export const voiceClient = new VoiceClient();
