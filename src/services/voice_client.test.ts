/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / voice_client.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import { Voice_Client, classify_mic_error } from './voice_client';
import { engine_api } from './system/engine_api';

// Mock socket to avoid side effects during construction
vi.mock('./socket', () => ({
    get_tadpole_os_socket: () => ({
        subscribe_audio_stream: vi.fn(),
    }),
    is_allowed_origin: () => true,
    is_private_network_origin: () => false
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

    afterEach(() => {
        voice_client_instance.destroy();
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
        const started = await voice_client_instance.start_recording();
        expect(started).toBe(true);
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

    it('should report recording startup failure', async () => {
        (navigator.mediaDevices.getUserMedia as ReturnType<typeof vi.fn>).mockRejectedValue(new Error('permission denied'));

        const started = await voice_client_instance.start_recording();

        expect(started).toBe(false);
        expect((voice_client_instance as any).media_recorder).toBeNull();
    });

    it('should perform text-to-speech', () => {
        voice_client_instance.speak('Hello from Tadpole');

        expect(window.speechSynthesis.cancel).toHaveBeenCalled();
        expect(window.speechSynthesis.speak).toHaveBeenCalled();
        expect(global.SpeechSynthesisUtterance).toHaveBeenCalledWith('Hello from Tadpole');
    });

    it('classifies NotAllowedError, NotFoundError, and NotReadableError into typed diagnostics', () => {
        const denied = Object.assign(new Error('Permission denied'), { name: 'NotAllowedError' });
        const missing = Object.assign(new Error('Requested device not found'), { name: 'NotFoundError' });
        const busy = Object.assign(new Error('Could not start audio source'), { name: 'NotReadableError' });

        expect(classify_mic_error(denied).code).toBe('permission_denied');
        expect(classify_mic_error(missing).code).toBe('no_device');
        expect(classify_mic_error(busy).code).toBe('device_busy');
    });

    it('enforces exclusive mic lease and preserves recorded mimeType on stop_recording', async () => {
        expect(voice_client_instance.acquire_mic_lease('live-voice-hub')).toBe(true);
        const blocked = await voice_client_instance.start_recording('standups-recorder');
        expect(blocked).toBe(false);
        expect(voice_client_instance.get_last_mic_error()?.code).toBe('mic_in_use');

        voice_client_instance.release_mic_lease('live-voice-hub');
        const started = await voice_client_instance.start_recording('standups-recorder');
        expect(started).toBe(true);

        const recorder = (voice_client_instance as any).media_recorder;
        recorder.mimeType = 'audio/ogg;codecs=opus';
        const stop_promise = voice_client_instance.stop_recording('standups-recorder');
        recorder.ondataavailable({ data: new Blob(['chunk'], { type: 'audio/ogg' }) });
        recorder.onstop();

        const blob = await stop_promise;
        expect(blob?.type).toBe('audio/ogg;codecs=opus');
        expect(voice_client_instance.get_mic_owner()).toBeNull();
    });

    it('routes openai TTS to backend without requiring neural-audio compile feature', async () => {
        const speak_spy = vi.spyOn(engine_api, 'speak').mockResolvedValue(
            new Blob(['{"status":"browser_fallback"}'], { type: 'application/json' })
        );
        await voice_client_instance.speak('Cloud voice test', 'nova', 'openai');
        expect(speak_spy).toHaveBeenCalledWith('Cloud voice test', 'nova', 'openai');
        speak_spy.mockRestore();
    });

    it('routes local piper TTS to backend so loopback LOCAL_TTS_URL works without neural-audio compile flag', async () => {
        const speak_spy = vi.spyOn(engine_api, 'speak').mockResolvedValue(
            new Blob(['{"status":"browser_fallback"}'], { type: 'application/json' })
        );
        await voice_client_instance.speak('Local piper voice test', 'af_heart', 'piper');
        expect(speak_spy).toHaveBeenCalledWith('Local piper voice test', 'af_heart', 'piper');
        speak_spy.mockRestore();
    });

    it('selects matching English browser voice instead of Google Deutsch listed first alphabetically', async () => {
        const german_voice = { name: 'Google Deutsch', lang: 'de-DE', default: false } as SpeechSynthesisVoice;
        const english_voice = { name: 'Google US English', lang: 'en-US', default: false } as SpeechSynthesisVoice;
        const orig_get_voices = window.speechSynthesis.getVoices;
        const orig_lang = document.documentElement.lang;
        document.documentElement.lang = 'en';
        window.speechSynthesis.getVoices = vi.fn().mockReturnValue([german_voice, english_voice]);

        try {
            await voice_client_instance.speak('Good evening', 'alloy', 'browser');
            const calls = (window.speechSynthesis.speak as ReturnType<typeof vi.fn>).mock.calls;
            const last_utterance = calls[calls.length - 1][0] as SpeechSynthesisUtterance;
            expect(last_utterance.voice?.name).toBe('Google US English');
            expect(last_utterance.lang).toBe('en-US');
        } finally {
            window.speechSynthesis.getVoices = orig_get_voices;
            document.documentElement.lang = orig_lang;
        }
    });

    it('pauses recognition and drops echo transcripts while TTS is speaking, then resumes', async () => {
        vi.useFakeTimers();
        const on_transcript = vi.fn();
        voice_client_instance.start_listening(on_transcript);

        const rec = (voice_client_instance as any).recognition;
        rec.abort = vi.fn();

        // Hold browser TTS open until we manually finish the utterance
        const tts_ctl: { finish?: () => void } = {};
        const orig_speak = window.speechSynthesis.speak;
        window.speechSynthesis.speak = vi.fn((utt: SpeechSynthesisUtterance) => {
            tts_ctl.finish = () => utt.onend?.(new Event('end') as SpeechSynthesisEvent);
        });

        try {
            const speak_promise = voice_client_instance.speak('Agent reply text', 'alloy', 'browser');
            expect(rec.abort).toHaveBeenCalledTimes(1);
            expect(voice_client_instance.is_speaking_tts()).toBe(true);

            // Echo transcript emitted during TTS playback must be dropped
            rec.onresult({ results: [[{ transcript: 'Agent reply text' }]] });
            expect(on_transcript).not.toHaveBeenCalled();

            // Finish TTS playback
            tts_ctl.finish?.();
            await speak_promise;
            expect(voice_client_instance.is_speaking_tts()).toBe(false);

            // Echo transcript during 450ms tail cooldown must still be dropped
            rec.onresult({ results: [[{ transcript: 'reply text' }]] });
            expect(on_transcript).not.toHaveBeenCalled();

            // After cooldown elapses, recognition resumes and user speech is accepted
            rec.start.mockClear();
            vi.advanceTimersByTime(500);
            expect(rec.start).toHaveBeenCalledTimes(1);

            rec.onresult({ results: [[{ transcript: 'Next user prompt' }]] });
            expect(on_transcript).toHaveBeenCalledWith('Next user prompt');
        } finally {
            window.speechSynthesis.speak = orig_speak;
            vi.useRealTimers();
        }
    });
});
