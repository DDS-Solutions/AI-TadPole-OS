/**
 * @docs ARCHITECTURE:AudioIntelligence
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / voice/recorder
 * - **Primary Entrypoints**: `Mic_Recorder`
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: `Mic_Capture_Error`
 * - **Telemetry Targets**: `[VoiceClient]`
 */

import {
    classify_mic_error,
    select_supported_recorder_mime,
    type Mic_Capture_Error,
} from '../voice_diagnostics';
import { Input_Level_Meter } from './input_meter';

export interface Mic_Recorder_Host {
    acquire_lease: (owner: string) => boolean;
    release_lease: (owner: string) => void;
    emit_error: (error: Mic_Capture_Error) => void;
    clear_last_error: () => void;
    is_destroyed: () => boolean;
}

/** Manages MediaRecorder lifecycle, MediaStream hardware track release, mute state, and input metering. */
export class Mic_Recorder {
    private media_recorder: MediaRecorder | null = null;
    private active_stream: MediaStream | null = null;
    private audio_chunks: Blob[] = [];
    private recording_generation = 0;
    private recording_pending = false;
    private is_muted = false;
    private selected_device_id: string | null = null;
    private readonly meter = new Input_Level_Meter();
    private readonly host: Mic_Recorder_Host;

    constructor(host: Mic_Recorder_Host) {
        this.host = host;
    }

    public get_media_recorder(): MediaRecorder | null {
        return this.media_recorder;
    }

    public set_input_device(device_id: string | null): void {
        this.selected_device_id = device_id && device_id.trim() ? device_id : null;
    }

    public get_input_device(): string | null {
        return this.selected_device_id;
    }

    public get_muted(): boolean {
        return this.is_muted;
    }

    public set_muted(muted: boolean): boolean {
        this.is_muted = muted;
        this.active_stream?.getAudioTracks?.().forEach((track) => {
            track.enabled = !muted;
        });
        return this.is_muted;
    }

    public get_input_level(): number {
        return this.meter.get_level(this.is_muted);
    }

    public async start(owner: string = 'media-recorder'): Promise<boolean> {
        this.host.clear_last_error();

        if (this.recording_pending || this.media_recorder) {
            return true;
        }
        if (!this.validate_capture_environment()) {
            return false;
        }
        if (!this.host.acquire_lease(owner)) {
            return false;
        }

        this.recording_pending = true;
        const generation = ++this.recording_generation;

        try {
            const audio_constraints: boolean | MediaTrackConstraints = this.selected_device_id
                ? { deviceId: { ideal: this.selected_device_id } }
                : true;
            const stream = await navigator.mediaDevices.getUserMedia({ audio: audio_constraints });

            if (this.host.is_destroyed() || generation !== this.recording_generation) {
                stream.getTracks().forEach((track) => track.stop());
                this.host.release_lease(owner);
                return false;
            }

            this.attach_stream(stream);
            return true;
        } catch (err) {
            console.error('[VoiceClient] Failed to start recording:', err);
            this.cleanup_failed_start(owner);
            this.host.emit_error(classify_mic_error(err));
            return false;
        } finally {
            this.recording_pending = false;
        }
    }

    public async stop(owner: string = 'media-recorder'): Promise<Blob | null> {
        this.recording_generation++;

        const recorder = this.media_recorder;
        const stream = this.active_stream || recorder?.stream || null;
        this.media_recorder = null;
        this.active_stream = null;
        this.meter.teardown();

        const release = () => {
            stream?.getTracks().forEach((track) => track.stop());
            this.host.release_lease(owner);
        };

        if (!recorder) {
            release();
            return null;
        }

        return this.await_recorder_stop(recorder, release);
    }

    public teardown_meter(): void {
        this.meter.teardown();
    }

    private validate_capture_environment(): boolean {
        if (typeof window !== 'undefined' && window.isSecureContext === false) {
            this.host.emit_error(classify_mic_error(new Error('insecure context')));
            return false;
        }
        if (typeof navigator === 'undefined' || !navigator.mediaDevices?.getUserMedia) {
            this.host.emit_error(classify_mic_error(new TypeError('navigator.mediaDevices.getUserMedia is unavailable')));
            return false;
        }
        return true;
    }

    private attach_stream(stream: MediaStream): void {
        stream.getAudioTracks?.().forEach((track) => {
            track.enabled = !this.is_muted;
        });

        const preferred_mime = select_supported_recorder_mime();
        const recorder = preferred_mime
            ? new MediaRecorder(stream, { mimeType: preferred_mime })
            : new MediaRecorder(stream);

        this.active_stream = stream;
        this.media_recorder = recorder;
        this.audio_chunks = [];

        this.media_recorder.ondataavailable = (event) => {
            if (event.data && event.data.size > 0) {
                this.audio_chunks.push(event.data);
            }
        };

        this.meter.attach(stream);
        this.media_recorder.start();
    }

    private cleanup_failed_start(owner: string): void {
        this.meter.teardown();
        this.active_stream?.getTracks().forEach((track) => track.stop());
        this.active_stream = null;
        this.media_recorder = null;
        this.host.release_lease(owner);
    }

    private await_recorder_stop(recorder: MediaRecorder, release: () => void): Promise<Blob | null> {
        const recorded_mime = recorder.mimeType || 'audio/webm';

        return new Promise((resolve) => {
            let settled = false;
            const finish = (blob: Blob | null) => {
                if (settled) return;
                settled = true;
                release();
                resolve(blob);
            };

            recorder.onstop = () => {
                const chunks = this.audio_chunks;
                this.audio_chunks = [];
                finish(new Blob(chunks, { type: recorded_mime }));
            };

            recorder.onerror = () => {
                this.audio_chunks = [];
                finish(null);
            };

            try {
                recorder.stop();
            } catch (err) {
                console.debug('[VoiceClient] MediaRecorder.stop() threw (likely inactive):', err);
                const chunks = this.audio_chunks;
                this.audio_chunks = [];
                finish(chunks.length > 0 ? new Blob(chunks, { type: recorded_mime }) : null);
            }
        });
    }
}
