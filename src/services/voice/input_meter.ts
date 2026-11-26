/**
 * @docs ARCHITECTURE:AudioIntelligence
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / voice/input_meter
 * - **Primary Entrypoints**: `Input_Level_Meter`
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[VoiceClient]`
 */

/** Real-time WebAudio AnalyserNode RMS meter for active microphone capture streams. */
export class Input_Level_Meter {
    private context: AudioContext | null = null;
    private analyser: AnalyserNode | null = null;
    private buffer: Float32Array<ArrayBuffer> | null = null;

    public attach(stream: MediaStream): void {
        this.teardown();
        if (typeof window === 'undefined') return;

        const win = window as unknown as {
            AudioContext?: typeof AudioContext;
            webkitAudioContext?: typeof AudioContext;
        };
        const AudioContextClass = win.AudioContext || win.webkitAudioContext;
        if (!AudioContextClass) return;

        try {
            const ctx = new AudioContextClass();
            if (typeof ctx.createMediaStreamSource !== 'function' || typeof ctx.createAnalyser !== 'function') {
                void ctx.close?.();
                return;
            }
            const source = ctx.createMediaStreamSource(stream);
            const analyser = ctx.createAnalyser();
            analyser.fftSize = 256;
            source.connect(analyser);
            this.context = ctx;
            this.analyser = analyser;
            this.buffer = new Float32Array(new ArrayBuffer(analyser.fftSize * 4));
        } catch (err) {
            console.debug('[VoiceClient] Input level meter unavailable:', err);
            this.teardown();
        }
    }

    public get_level(is_muted: boolean): number {
        if (is_muted || !this.analyser || !this.buffer) {
            return 0;
        }
        try {
            this.analyser.getFloatTimeDomainData(this.buffer);
            let sum_sq = 0;
            for (let i = 0; i < this.buffer.length; i++) {
                const sample = this.buffer[i];
                sum_sq += sample * sample;
            }
            const rms = Math.sqrt(sum_sq / this.buffer.length);
            return Math.min(1, rms * 4);
        } catch {
            return 0;
        }
    }

    public teardown(): void {
        this.analyser = null;
        this.buffer = null;
        if (!this.context) return;

        try {
            void this.context.close();
        } catch {
            // Ignore close error on already-closed context
        }
        this.context = null;
    }
}
