/**
 * @docs ARCHITECTURE:AudioIntelligence
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / voice/pcm_player
 * - **Primary Entrypoints**: `Pcm_Audio_Player`
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[VoiceClient]`
 */

import { get_tadpole_os_socket } from '../socket';

const PIPER_SAMPLE_RATE = 22050;

/** Schedules incoming 16-bit PCM WebSocket audio chunks onto a WebAudio output graph. */
export class Pcm_Audio_Player {
    private audio_context: AudioContext | null = null;
    private next_start_time = 0;
    private audio_unsub: (() => void) | null = null;

    public ensure_subscription(): void {
        if (this.audio_unsub) return;

        this.audio_unsub = get_tadpole_os_socket().subscribe_audio_stream((chunk) => {
            this.handle_audio_chunk(chunk).catch((error: unknown) => {
                console.error('[VoiceClient] Audio stream initialization failed:', error);
            });
        });
    }

    private ensure_context(): AudioContext {
        if (!this.audio_context) {
            const win = window as unknown as {
                AudioContext: typeof AudioContext;
                webkitAudioContext: typeof AudioContext;
            };
            const AudioContextClass = win.AudioContext || win.webkitAudioContext;
            this.audio_context = new AudioContextClass();
            this.next_start_time = this.audio_context.currentTime;
        }
        return this.audio_context;
    }

    private async handle_audio_chunk(chunk: ArrayBuffer): Promise<void> {
        const ctx = this.ensure_context();

        if (ctx.state === 'suspended') {
            try {
                await ctx.resume();
            } catch (e) {
                console.debug('[VoiceClient] AudioContext resume deferred pending user gesture:', e);
                return;
            }
        }

        try {
            const int16_array = new Int16Array(chunk);
            const float32_array = new Float32Array(int16_array.length);
            for (let i = 0; i < int16_array.length; i++) {
                float32_array[i] = int16_array[i] / 32768.0;
            }

            const buffer = ctx.createBuffer(1, float32_array.length, PIPER_SAMPLE_RATE);
            buffer.getChannelData(0).set(float32_array);

            const source = ctx.createBufferSource();
            source.buffer = buffer;
            source.connect(ctx.destination);

            const start_time = Math.max(ctx.currentTime, this.next_start_time);
            source.start(start_time);
            this.next_start_time = Math.max(ctx.currentTime, start_time + buffer.duration);
        } catch (err) {
            console.error('[VoiceClient] Chunk playback failed:', err);
        }
    }

    public destroy(): void {
        if (this.audio_unsub) {
            this.audio_unsub();
            this.audio_unsub = null;
        }
        if (this.audio_context) {
            try {
                void this.audio_context.close();
            } catch {
                // Ignore failure on closing already-closed audio context
            }
            this.audio_context = null;
        }
    }
}
