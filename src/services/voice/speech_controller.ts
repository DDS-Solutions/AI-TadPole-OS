/**
 * @docs ARCHITECTURE:AudioIntelligence
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / voice/speech_controller
 * - **Primary Entrypoints**: `Speech_Controller`
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: `Mic_Capture_Error`
 * - **Telemetry Targets**: `[VoiceClient]`
 */

import { classify_mic_error, type Mic_Capture_Error } from '../voice_diagnostics';
import type { Speech_Recognition_Interface, Voice_Status } from './types';

export interface Speech_Controller_Host {
    acquire_lease: (owner: string) => boolean;
    release_lease: (owner: string) => void;
    emit_error: (error: Mic_Capture_Error) => void;
    set_status: (status: Voice_Status) => void;
    get_status: () => Voice_Status;
    ensure_feature_detection: () => void;
    is_tts_speaking: () => boolean;
    is_destroyed: () => boolean;
}

/** Manages Web SpeechRecognition lifecycle, stall/restart timers, and half-duplex TTS echo suppression. */
export class Speech_Controller {
    private recognition: Speech_Recognition_Interface | null = null;
    private is_listening = false;
    private on_transcript_callback: ((text: string) => void) | null = null;
    private stall_timer: ReturnType<typeof setTimeout> | null = null;
    private restart_timer: ReturnType<typeof setTimeout> | null = null;
    private paused_for_tts = false;
    private tts_resume_timer: ReturnType<typeof setTimeout> | null = null;
    private tts_cooldown_until = 0;
    private readonly host: Speech_Controller_Host;

    constructor(host: Speech_Controller_Host) {
        this.host = host;
        this.init_recognition();
    }

    public get_recognition(): Speech_Recognition_Interface | null {
        return this.recognition;
    }

    public is_supported(): boolean {
        return this.recognition !== null;
    }

    public start(callback: (text: string) => void): boolean {
        if (!this.recognition) {
            console.error('[VoiceClient] Recognition engine unavailable.');
            this.host.emit_error({
                code: 'speech_api_unavailable',
                message: 'Web Speech API is not supported in this browser/WebView.',
            });
            return false;
        }

        if (!this.host.acquire_lease('speech-recognition')) {
            return false;
        }

        this.host.ensure_feature_detection();
        console.debug('[VoiceClient] Initializing voice link...');
        this.on_transcript_callback = callback;
        this.is_listening = true;
        this.host.set_status('initializing');
        this.arm_stall_timer();

        try {
            this.recognition.start();
            return true;
        } catch (err) {
            return this.handle_start_failure(err);
        }
    }

    public stop(): void {
        console.debug('[VoiceClient] Terminating voice link.');
        this.is_listening = false;
        this.paused_for_tts = false;
        this.clear_all_timers();
        this.host.release_lease('speech-recognition');
        this.host.set_status('idle');

        if (!this.recognition) return;
        try {
            this.recognition.stop();
        } catch {
            // Ignore stop errors when recognition wasn't active
        }
    }

    /** Half-duplex echo suppression: pause recognition before TTS audio plays. */
    public pause_for_tts(): void {
        this.clear_timer('tts_resume');
        this.clear_timer('restart');

        if (!this.is_listening || !this.recognition || this.paused_for_tts) {
            return;
        }
        this.paused_for_tts = true;
        try {
            if (typeof this.recognition.abort === 'function') {
                this.recognition.abort();
            } else {
                this.recognition.stop();
            }
        } catch {
            // Ignore stop/abort errors if recognition was already idle
        }
    }

    /** Restores recognition after TTS finishes and a brief acoustic echo tail cooldown elapses. */
    public schedule_resume_after_tts(cooldown_ms: number = 450): void {
        this.tts_cooldown_until = Date.now() + cooldown_ms;

        if (!this.paused_for_tts || !this.is_listening || !this.recognition || this.host.is_destroyed()) {
            this.paused_for_tts = false;
            return;
        }

        this.tts_resume_timer = setTimeout(() => {
            this.tts_resume_timer = null;
            this.paused_for_tts = false;
            if (!this.is_listening || !this.recognition || this.host.is_tts_speaking() || this.host.is_destroyed()) {
                return;
            }
            try {
                this.recognition.start();
            } catch (e) {
                console.debug('[VoiceClient] Failed to resume recognition after TTS:', e);
            }
        }, cooldown_ms);
    }

    private init_recognition(): void {
        const win = window as unknown as {
            SpeechRecognition?: new () => Speech_Recognition_Interface;
            webkitSpeechRecognition?: new () => Speech_Recognition_Interface;
        };
        const SpeechRecognitionClass = win.SpeechRecognition || win.webkitSpeechRecognition;
        if (!SpeechRecognitionClass) {
            console.warn('[VoiceClient] Web Speech API not supported in this browser.');
            return;
        }

        const rec = new SpeechRecognitionClass();
        rec.continuous = true;
        rec.interimResults = false;
        rec.lang = 'en-US';
        rec.onresult = (event) => this.handle_result(event);
        rec.onend = () => this.handle_end();
        rec.onerror = (event) => this.handle_error(event.error);
        this.recognition = rec;
    }

    private handle_result(event: { results: { readonly length: number; [index: number]: { [index: number]: { transcript: string } } } }): void {
        if (this.host.is_tts_speaking() || this.paused_for_tts || Date.now() < this.tts_cooldown_until) {
            return;
        }
        const results = event.results;
        const transcript = results[results.length - 1][0].transcript.trim();

        const current_status = this.host.get_status();
        if (current_status === 'initializing' || current_status === 'stalled') {
            this.clear_timer('stall');
            this.host.set_status('active');
        }

        if (transcript && this.on_transcript_callback) {
            this.on_transcript_callback(transcript);
        }
    }

    private handle_end(): void {
        if (this.paused_for_tts || this.host.is_tts_speaking()) {
            return;
        }
        if (!this.is_listening || !this.recognition) {
            if (!this.is_listening) {
                this.host.set_status('idle');
            }
            return;
        }

        this.clear_timer('restart');
        this.restart_timer = setTimeout(() => {
            this.restart_timer = null;
            if (!this.is_listening || !this.recognition) return;
            try {
                this.recognition.start();
            } catch (e) {
                console.debug('[VoiceClient] Failed to restart recognition:', e);
            }
        }, 250);
    }

    private handle_error(err_code: string): void {
        console.error('[VoiceClient] Recognition error:', err_code);
        if (err_code === 'not-allowed' || err_code === 'service-not-allowed') {
            this.abort_listening_with_error(classify_mic_error(new Error(err_code)));
            return;
        }
        if (err_code === 'audio-capture') {
            this.abort_listening_with_error(classify_mic_error(new Error('audio-capture')));
            return;
        }
        if (err_code === 'network') {
            this.abort_listening_with_error({
                code: 'speech_network_error',
                message: 'Browser Web Speech service is unreachable. Falling back to engine audio recording when available.',
                raw_name: 'network',
            });
            return;
        }
        this.host.set_status('stalled');
    }

    private abort_listening_with_error(error: Mic_Capture_Error): void {
        this.is_listening = false;
        this.host.release_lease('speech-recognition');
        this.host.set_status('idle');
        this.host.emit_error(error);
    }

    private handle_start_failure(err: unknown): boolean {
        const is_already_started = err instanceof Error && err.name === 'InvalidStateError';
        if (is_already_started) {
            console.debug('[VoiceClient] Recognition.start() ignored (already started):', err);
            return true;
        }
        console.error('[VoiceClient] Recognition.start() failed:', err);
        this.is_listening = false;
        this.clear_timer('stall');
        this.host.release_lease('speech-recognition');
        this.host.set_status('idle');
        this.host.emit_error(classify_mic_error(err));
        return false;
    }

    private arm_stall_timer(): void {
        this.clear_timer('stall');
        this.stall_timer = setTimeout(() => {
            if (this.host.get_status() === 'initializing') {
                console.warn('[VoiceClient] Voice link stalled: No signal detected after 5s.');
                this.host.set_status('stalled');
            }
        }, 5000);
    }

    private clear_timer(which: 'stall' | 'restart' | 'tts_resume'): void {
        if (which === 'stall' && this.stall_timer) {
            clearTimeout(this.stall_timer);
            this.stall_timer = null;
        } else if (which === 'restart' && this.restart_timer) {
            clearTimeout(this.restart_timer);
            this.restart_timer = null;
        } else if (which === 'tts_resume' && this.tts_resume_timer) {
            clearTimeout(this.tts_resume_timer);
            this.tts_resume_timer = null;
        }
    }

    private clear_all_timers(): void {
        this.clear_timer('tts_resume');
        this.clear_timer('restart');
        this.clear_timer('stall');
    }
}
