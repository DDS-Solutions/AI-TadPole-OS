/**
 * @docs ARCHITECTURE:AudioIntelligence
 *
 * ### AI Assist Note
 * - Keep audio subscription callbacks synchronous and handle rejected playback startup at their boundary.
 * - Exclusive microphone lease prevents concurrent MediaRecorder and Live Voice capture contention.
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / voice_client
 * - **Primary Entrypoints**: `voice_client`, `Voice_Client`
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: `Mic_Capture_Error`
 * - **Telemetry Targets**: `[VoiceClient]`
 */

import { system_api_service } from './system_api_service';
import {
    classify_mic_error,
    type Mic_Error_Code,
    type Mic_Capture_Error,
    type Audio_Input_Device,
} from './voice_diagnostics';
import { Pcm_Audio_Player } from './voice/pcm_player';
import { Mic_Recorder } from './voice/recorder';
import { Speech_Controller } from './voice/speech_controller';
import { speak_browser, speak_premium } from './voice/tts_engine';
import type {
    Speech_Recognition_Interface,
    Voice_Status,
    Voice_Tts_Engine,
} from './voice/types';

export { classify_mic_error, type Mic_Error_Code, type Mic_Capture_Error, type Audio_Input_Device };
export type { Voice_Status };

/**
 * Voice_Client
 * Orchestrates local and cloud-based speech synthesis, recognition,
 * exclusive microphone capture leasing, and real-time input level metering.
 */
export class Voice_Client {
    private readonly status_callbacks = new Set<(status: Voice_Status) => void>();
    private readonly error_callbacks = new Set<(error: Mic_Capture_Error) => void>();
    private status: Voice_Status = 'idle';
    private is_tts_speaking = false;
    private tts_generation = 0;

    private last_mic_error: Mic_Capture_Error | null = null;
    private mic_owner: string | null = null;

    private backend_features: string[] = [];
    private features_detected = false;
    private is_destroyed = false;

    private readonly pcm_player = new Pcm_Audio_Player();
    private readonly recorder: Mic_Recorder;
    private readonly speech: Speech_Controller;

    constructor() {
        this.recorder = new Mic_Recorder({
            acquire_lease: (owner) => this.acquire_and_emit_lease(owner),
            release_lease: (owner) => this.release_mic_lease(owner),
            emit_error: (err) => this.emit_error(err),
            clear_last_error: () => {
                this.last_mic_error = null;
            },
            is_destroyed: () => this.is_destroyed,
        });

        this.speech = new Speech_Controller({
            acquire_lease: (owner) => this.acquire_and_emit_lease(owner),
            release_lease: (owner) => this.release_mic_lease(owner),
            emit_error: (err) => this.emit_error(err),
            set_status: (status) => this.set_status(status),
            get_status: () => this.status,
            ensure_feature_detection: () => this.ensure_feature_detection(),
            is_tts_speaking: () => this.is_tts_speaking,
            is_destroyed: () => this.is_destroyed,
        });
    }

    /** Internal accessor preserved for unit-test event simulation hooks. */
    protected get recognition(): Speech_Recognition_Interface | null {
        return this.speech.get_recognition();
    }

    /** Internal accessor preserved for unit-test MediaRecorder inspection hooks. */
    protected get media_recorder(): MediaRecorder | null {
        return this.recorder.get_media_recorder();
    }

    // ─── Exclusive Microphone Lease ─────────────────────────────────────────

    public acquire_mic_lease(owner: string): boolean {
        if (this.mic_owner && this.mic_owner !== owner) {
            this.last_mic_error = {
                code: 'mic_in_use',
                message: `Microphone is already active in another voice session (${this.mic_owner}). Stop that session first.`,
            };
            return false;
        }
        this.mic_owner = owner;
        return true;
    }

    public release_mic_lease(owner: string): void {
        if (this.mic_owner === owner) {
            this.mic_owner = null;
        }
    }

    public get_mic_owner(): string | null {
        return this.mic_owner;
    }

    public get_last_mic_error(): Mic_Capture_Error | null {
        return this.last_mic_error;
    }

    public is_speech_recognition_supported(): boolean {
        return this.speech.is_supported();
    }

    public set_input_device(device_id: string | null): void {
        this.recorder.set_input_device(device_id);
    }

    public get_input_device(): string | null {
        return this.recorder.get_input_device();
    }

    public async list_input_devices(): Promise<Audio_Input_Device[]> {
        if (typeof navigator === 'undefined' || !navigator.mediaDevices?.enumerateDevices) {
            return [];
        }
        try {
            const devices = await navigator.mediaDevices.enumerateDevices();
            return devices
                .filter((d) => d.kind === 'audioinput')
                .map((d, idx) => ({
                    device_id: d.deviceId,
                    label: d.label || `Microphone ${idx + 1}`,
                }));
        } catch (err) {
            console.debug('[VoiceClient] Failed to enumerate audio devices:', err);
            return [];
        }
    }

    public get_muted(): boolean {
        return this.recorder.get_muted();
    }

    public set_muted(muted: boolean): boolean {
        return this.recorder.set_muted(muted);
    }

    public toggle_mute(): boolean {
        return this.recorder.set_muted(!this.recorder.get_muted());
    }

    public get_input_level(): number {
        return this.recorder.get_input_level();
    }

    public ensure_audio_subscription(): void {
        this.pcm_player.ensure_subscription();
    }

    public get_status = (): Voice_Status => {
        return this.status;
    };

    public on_status_change = (callback: (status: Voice_Status) => void): (() => void) => {
        this.status_callbacks.add(callback);
        return () => {
            this.status_callbacks.delete(callback);
        };
    };

    public on_error = (callback: (error: Mic_Capture_Error) => void): (() => void) => {
        this.error_callbacks.add(callback);
        return () => {
            this.error_callbacks.delete(callback);
        };
    };

    public is_speaking_tts(): boolean {
        return this.is_tts_speaking;
    }

    public start_listening(callback: (text: string) => void): boolean {
        return this.speech.start(callback);
    }

    public stop_listening(): void {
        this.speech.stop();
    }

    public start_recording(owner: string = 'media-recorder'): Promise<boolean> {
        return this.recorder.start(owner);
    }

    public stop_recording(owner: string = 'media-recorder'): Promise<Blob | null> {
        return this.recorder.stop(owner);
    }

    public async speak(text: string, voice_id?: string, engine: Voice_Tts_Engine = 'browser'): Promise<void> {
        if (!text) return;

        this.ensure_feature_detection();
        const generation = ++this.tts_generation;
        this.is_tts_speaking = true;
        this.speech.pause_for_tts();

        if (window.speechSynthesis) {
            window.speechSynthesis.cancel();
        }

        const synthesis_engine: 'browser' | 'openai' | 'piper' =
            engine === 'openai' || engine === 'piper' ? engine : 'browser';

        try {
            if (synthesis_engine === 'browser') {
                await speak_browser(text, voice_id);
            } else {
                await speak_premium(text, voice_id, synthesis_engine, () => this.ensure_audio_subscription());
            }
        } finally {
            if (generation === this.tts_generation) {
                this.is_tts_speaking = false;
                this.speech.schedule_resume_after_tts(450);
            }
        }
    }

    public destroy(): void {
        this.is_destroyed = true;
        this.stop_listening();
        void this.stop_recording();
        this.recorder.teardown_meter();
        this.status_callbacks.clear();
        this.error_callbacks.clear();
        this.mic_owner = null;
        this.pcm_player.destroy();
    }

    private acquire_and_emit_lease(owner: string): boolean {
        if (this.acquire_mic_lease(owner)) {
            return true;
        }
        if (this.last_mic_error) {
            this.emit_error(this.last_mic_error);
        }
        return false;
    }

    private emit_error(error: Mic_Capture_Error): void {
        this.last_mic_error = error;
        this.error_callbacks.forEach((cb) => {
            try {
                cb(error);
            } catch (err) {
                console.error('[VoiceClient] Error in error callback:', err);
            }
        });
    }

    private set_status(status: Voice_Status): void {
        this.status = status;
        this.status_callbacks.forEach((callback) => {
            try {
                callback(status);
            } catch (err) {
                console.error('[VoiceClient] Error in status callback:', err);
            }
        });
    }

    private ensure_feature_detection(): void {
        if (this.features_detected) return;
        this.features_detected = true;
        void this.init_feature_detection();
    }

    private async init_feature_detection(): Promise<void> {
        try {
            if (this.is_destroyed) return;
            const status = await system_api_service.engine.get_engine_status();
            if (this.is_destroyed) return;
            this.backend_features = status?.features || [];

            if (!this.backend_features.includes('neural-audio')) {
                console.debug('[VoiceClient] Backend "neural-audio" feature missing. Local Piper/Whisper disabled; cloud and browser engines available.');
            }
        } catch (err) {
            if (!this.is_destroyed) {
                console.debug('[VoiceClient] Feature detection failed, engine unreachable:', err);
            }
        }
    }
}

export const voice_client = new Voice_Client();
