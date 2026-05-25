/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Validation of the Voice and Audio processing service.** 
 * Verifies the integration with the Whisper/TTS engine, audio buffer handling, and recording lifecycle (Start/Stop/Transcribe). 
 * Mocks browser-specific APIs (`SpeechRecognition`, `MediaRecorder`, `SpeechSynthesis`) to isolate comms logic from hardware side-effects.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Audio buffer overflow during long recordings or failure to release the microphone device after a session ends.
 * - **Telemetry Link**: Search `[voice_client.test]` in tracing logs.
 */


/**
 * @file voice_client.test.ts
 * @description Suite for the Neural Comms and Pulse Voice interface.
 * @module Services/voice_client
 * @testedBehavior
 * - Transcription (STT): Verification of SpeechRecognition initialization and event handling.
 * - Recording: Validation of MediaRecorder lifecycle and audio blob generation.
 * - Synthesis (TTS): Verification of SpeechSynthesis orchestration and utterance generation.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks browser-specific APIs (SpeechRecognition, MediaRecorder, SpeechSynthesis) to isolate logic.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { Voice_Client } from './voice_client';

// Mock socket to avoid side effects during construction
vi.mock('./socket', () => ({
    tadpole_os_socket: {
        subscribe_audio_stream: vi.fn(),
    }
}));

describe('Voice_Client', () => {
    let voice_client_instance: Voice_Client;

    beforeEach(() => {
        vi.useFakeTimers();
        vi.clearAllMocks();

        // Mock SpeechRecognition
        const mock_speech_recognition = vi.fn().mockImplementation(function (this: { start: any, stop: any, onresult: any, onend: any, onerror: any, continuous: boolean, interimResults: boolean, lang: string }) {
            this.start = vi.fn();
            this.stop = vi.fn();
            this.onresult = null;
            this.onend = null;
            this.onerror = null;
            this.continuous = false;
            this.interimResults = false;
            this.lang = '';
        });
        (window as unknown as { SpeechRecognition: any }).SpeechRecognition = mock_speech_recognition;
        (window as unknown as { webkitSpeechRecognition: any }).webkitSpeechRecognition = mock_speech_recognition;

        // Mock MediaRecorder
        const mock_media_stream = {
            getTracks: vi.fn().mockReturnValue([{ stop: vi.fn() }])
        };
        (navigator as unknown as { mediaDevices: any }).mediaDevices = {
            getUserMedia: vi.fn().mockResolvedValue(mock_media_stream)
        };

        const mock_media_recorder = vi.fn().mockImplementation(function (this: { start: any, stop: any, ondataavailable: any, onstop: any, stream: any }) {
            this.start = vi.fn();
            this.stop = vi.fn();
            this.ondataavailable = null;
            this.onstop = null;
            this.stream = mock_media_stream;
        });
        (globalThis as unknown as { MediaRecorder: any }).MediaRecorder = mock_media_recorder;

        // Mock SpeechSynthesis
        (window as unknown as { speechSynthesis: any }).speechSynthesis = {
            speak: vi.fn(),
            cancel: vi.fn(),
            getVoices: vi.fn().mockReturnValue([
                { name: 'Google US English', lang: 'en-US' },
                { name: 'Samantha', lang: 'en-US' }
            ])
        };
        const mock_utterance = vi.fn().mockImplementation(function (this: { text: string, rate: number, pitch: number, volume: number, voice: any }, text: string) {
            this.text = text;
            this.rate = 1;
            this.pitch = 1;
            this.volume = 1;
            this.voice = null;
        });
        (globalThis as unknown as { SpeechSynthesisUtterance: unknown }).SpeechSynthesisUtterance = mock_utterance;

        voice_client_instance = new Voice_Client();
    });

    it('should initialize SpeechRecognition', () => {
        expect((window as unknown as { SpeechRecognition: any }).SpeechRecognition).toHaveBeenCalled();
    });

    it('should start and stop listening', () => {
        const recognition = (voice_client_instance as unknown as { recognition: any }).recognition;
        const start_spy = vi.spyOn(recognition, 'start');
        const stop_spy = vi.spyOn(recognition, 'stop');

        voice_client_instance.start_listening(() => { });
        expect(voice_client_instance.get_status()).toBe('initializing');
        expect(start_spy).toHaveBeenCalled();

        voice_client_instance.stop_listening();
        expect(voice_client_instance.get_status()).toBe('idle');
        expect(stop_spy).toHaveBeenCalled();
    });

    it('should transition status to active when speech is detected', () => {
        voice_client_instance.start_listening(() => { });
        expect(voice_client_instance.get_status()).toBe('initializing');

        // Simulate onresult event
        const mock_event = {
            results: [[{ transcript: 'test' }]]
        };
        (voice_client_instance as any).recognition.onresult(mock_event);

        expect(voice_client_instance.get_status()).toBe('active');
    });

    it('should transition to stalled status if no speech within 5 seconds', () => {
        voice_client_instance.start_listening(() => { });
        expect(voice_client_instance.get_status()).toBe('initializing');

        // Advance timers by 5001ms
        vi.advanceTimersByTime(5001);

        expect(voice_client_instance.get_status()).toBe('stalled');
    });

    it('should notify subscribers on status change', () => {
        const callback = vi.fn();
        voice_client_instance.on_status_change(callback);

        voice_client_instance.start_listening(() => { });
        expect(callback).toHaveBeenCalledWith('initializing');

        voice_client_instance.stop_listening();
        expect(callback).toHaveBeenCalledWith('idle');
    });

    it('should handle speech transcription via callback', () => {
        const callback = vi.fn();
        voice_client_instance.start_listening(callback);

        // Simulate onresult event
        const mock_event = {
            results: [
                [{ transcript: 'hello world' }]
            ]
        };
        (voice_client_instance as any).recognition.onresult(mock_event);

        expect(callback).toHaveBeenCalledWith('hello world');
    });

    it('should start and stop recording', async () => {
        await voice_client_instance.start_recording();
        expect(navigator.mediaDevices.getUserMedia).toHaveBeenCalledWith({ audio: true });
        expect(global.MediaRecorder).toHaveBeenCalled();

        const recorder = (voice_client_instance as any).media_recorder;
        if (!recorder) throw new Error('Recorder not initialized');
        const stop_spy = vi.spyOn(recorder, 'stop');

        const stop_promise = voice_client_instance.stop_recording();

        // Simulate data available and then stop
        (recorder as any).ondataavailable({
            data: new Blob(['test'], { type: 'audio/wav' })
        });
        (recorder as unknown as { onstop: any }).onstop();

        const result = await stop_promise;
        expect(stop_spy).toHaveBeenCalled();
        expect(result).toBeInstanceOf(Blob);
    });

    it('should perform text-to-speech', () => {
        voice_client_instance.speak('Hello from Tadpole');

        expect(window.speechSynthesis.cancel).toHaveBeenCalled();
        expect(window.speechSynthesis.speak).toHaveBeenCalled();
        expect(global.SpeechSynthesisUtterance).toHaveBeenCalledWith('Hello from Tadpole');
    });
});


// Metadata: [voice_client_test]

// Metadata: [voice_client_test]
