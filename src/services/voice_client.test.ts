/**
 * @file voiceClient.test.ts
 * @description Suite for browser-based voice capturing and speech synthesis (TTS/ASR).
 * @module Services/VoiceClient
 * @testedBehavior
 * - Speech Recognition: Integration with the Web Speech API for real-time transcription.
 * - Audio Capturing: Integration with MediaRecorder for blob-based audio recording.
 * - Synthesis: Integration with window.speechSynthesis for text-to-speech feedback.
 * @aiContext
 * - Extensively mocks browser globals: SpeechRecognition, MediaRecorder, and SpeechSynthesis.
 * - Simulates asynchronous blob generation during the recording lifecycle.
 */
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { VoiceClient } from './voice_client';

describe('voice_client', () => {
    let voiceClient: VoiceClient;

    beforeEach(() => {
        vi.clearAllMocks();

        // Mock SpeechRecognition
        const mockSpeechRecognition = vi.fn().mockImplementation(function (this: Record<string, unknown>) {
            this.start = vi.fn();
            this.stop = vi.fn();
            this.onresult = null;
            this.onend = null;
            this.onerror = null;
            this.continuous = false;
            this.interimResults = false;
            this.lang = '';
        });
        (window as unknown as Record<string, unknown>).SpeechRecognition = mockSpeechRecognition;
        (window as unknown as Record<string, unknown>).webkitSpeechRecognition = mockSpeechRecognition;

        // Mock MediaRecorder
        const mockMediaStream = {
            getTracks: vi.fn().mockReturnValue([{ stop: vi.fn() }])
        };
        (global as unknown as { navigator: { mediaDevices: unknown } }).navigator.mediaDevices = {
            getUserMedia: vi.fn().mockResolvedValue(mockMediaStream)
        };

        const mockMediaRecorder = vi.fn().mockImplementation(function (this: Record<string, unknown>) {
            this.start = vi.fn();
            this.stop = vi.fn();
            this.ondataavailable = null;
            this.onstop = null;
            this.stream = mockMediaStream;
        });
        (global as unknown as Record<string, unknown>).MediaRecorder = mockMediaRecorder;

        // Mock SpeechSynthesis
        (global as unknown as { window: { speechSynthesis: unknown } }).window.speechSynthesis = {
            speak: vi.fn(),
            cancel: vi.fn(),
            getVoices: vi.fn().mockReturnValue([
                { name: 'Google US English', lang: 'en-US' },
                { name: 'Samantha', lang: 'en-US' }
            ])
        };
        const mockUtterance = vi.fn().mockImplementation(function (this: Record<string, unknown>, text: string) {
            this.text = text;
            this.rate = 1;
            this.pitch = 1;
            this.volume = 1;
            this.voice = null;
        });
        (global as unknown as Record<string, unknown>).SpeechSynthesisUtterance = mockUtterance;

        voiceClient = new VoiceClient();
    });

    it('should initialize SpeechRecognition', () => {
        expect((window as unknown as Record<string, unknown>).SpeechRecognition).toHaveBeenCalled();
    });

    it('should start and stop listening', () => {
        const spy = vi.spyOn((voiceClient as unknown as { recognition: { start: () => void } }).recognition, 'start');
        const stopSpy = vi.spyOn((voiceClient as unknown as { recognition: { stop: () => void } }).recognition, 'stop');

        voiceClient.startListening(() => { });
        expect(spy).toHaveBeenCalled();

        voiceClient.stopListening();
        expect(stopSpy).toHaveBeenCalled();
    });

    it('should handle speech transcription via callback', () => {
        const callback = vi.fn();
        voiceClient.startListening(callback);

        // Simulate onresult event
        const mockEvent = {
            results: [
                [{ transcript: 'hello world' }]
            ]
        };
        (voiceClient as unknown as { recognition: { onresult: (ev: unknown) => void } }).recognition.onresult(mockEvent);

        expect(callback).toHaveBeenCalledWith('hello world');
    });

    it('should start and stop recording', async () => {
        await voiceClient.startRecording();
        expect(navigator.mediaDevices.getUserMedia).toHaveBeenCalledWith({ audio: true });
        expect(global.MediaRecorder).toHaveBeenCalled();

        const recorder = (voiceClient as unknown as { mediaRecorder: MediaRecorder | null }).mediaRecorder;
        if (!recorder) throw new Error('Recorder not initialized');
        const stopSpy = vi.spyOn(recorder, 'stop');

        const stopPromise = voiceClient.stopRecording();

        // Simulate data available and then stop
        (recorder as unknown as { ondataavailable: (ev: { data: Blob }) => void }).ondataavailable({
            data: new Blob(['test'], { type: 'audio/wav' })
        });
        (recorder as unknown as { onstop: () => void }).onstop();

        const result = await stopPromise;
        expect(stopSpy).toHaveBeenCalled();
        expect(result).toBeInstanceOf(Blob);
    });

    it('should perform text-to-speech', () => {
        voiceClient.speak('Hello from Tadpole');

        expect(window.speechSynthesis.cancel).toHaveBeenCalled();
        expect(window.speechSynthesis.speak).toHaveBeenCalled();
        expect(global.SpeechSynthesisUtterance).toHaveBeenCalledWith('Hello from Tadpole');
    });
});

