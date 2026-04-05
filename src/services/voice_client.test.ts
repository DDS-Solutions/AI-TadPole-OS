/**
 * @docs ARCHITECTURE:Services
 * 
 * ### AI Assist Note
 * **Test Suite**: Validates the Audio telemetry and voice agent bridge. 
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
        vi.clearAllMocks();

        // Mock SpeechRecognition
        const mock_speech_recognition = vi.fn().mockImplementation(function (this: any) {
            this.start = vi.fn();
            this.stop = vi.fn();
            this.onresult = null;
            this.onend = null;
            this.onerror = null;
            this.continuous = false;
            this.interimResults = false;
            this.lang = '';
        });
        (window as any).SpeechRecognition = mock_speech_recognition;
        (window as any).webkitSpeechRecognition = mock_speech_recognition;

        // Mock MediaRecorder
        const mock_media_stream = {
            getTracks: vi.fn().mockReturnValue([{ stop: vi.fn() }])
        };
        (navigator as any).mediaDevices = {
            getUserMedia: vi.fn().mockResolvedValue(mock_media_stream)
        };

        const mock_media_recorder = vi.fn().mockImplementation(function (this: any) {
            this.start = vi.fn();
            this.stop = vi.fn();
            this.ondataavailable = null;
            this.onstop = null;
            this.stream = mock_media_stream;
        });
        (global as any).MediaRecorder = mock_media_recorder;

        // Mock SpeechSynthesis
        (window as any).speechSynthesis = {
            speak: vi.fn(),
            cancel: vi.fn(),
            getVoices: vi.fn().mockReturnValue([
                { name: 'Google US English', lang: 'en-US' },
                { name: 'Samantha', lang: 'en-US' }
            ])
        };
        const mock_utterance = vi.fn().mockImplementation(function (this: any, text: string) {
            this.text = text;
            this.rate = 1;
            this.pitch = 1;
            this.volume = 1;
            this.voice = null;
        });
        (global as any).SpeechSynthesisUtterance = mock_utterance;

        voice_client_instance = new Voice_Client();
    });

    it('should initialize SpeechRecognition', () => {
        expect((window as any).SpeechRecognition).toHaveBeenCalled();
    });

    it('should start and stop listening', () => {
        const recognition = (voice_client_instance as any).recognition;
        const start_spy = vi.spyOn(recognition, 'start');
        const stop_spy = vi.spyOn(recognition, 'stop');

        voice_client_instance.start_listening(() => { });
        expect(start_spy).toHaveBeenCalled();

        voice_client_instance.stop_listening();
        expect(stop_spy).toHaveBeenCalled();
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
        (recorder as any).onstop();

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

