/**
 * @docs ARCHITECTURE:AudioIntelligence
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / voice_diagnostics
 * - **Primary Entrypoints**: `classify_mic_error`, `select_supported_recorder_mime`, `Mic_Capture_Error`, `Audio_Input_Device`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Pure deterministic classification of browser/WebView media errors with zero side effects.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: `Mic_Capture_Error`
 * - **Telemetry Targets**: none declared
 */

export type Mic_Error_Code =
    | 'insecure_context'
    | 'media_devices_unavailable'
    | 'permission_denied'
    | 'no_device'
    | 'device_busy'
    | 'unsupported_recorder'
    | 'mic_in_use'
    | 'speech_api_unavailable'
    | 'speech_network_error'
    | 'unknown';

export interface Mic_Capture_Error {
    code: Mic_Error_Code;
    message: string;
    raw_name?: string;
}

export interface Audio_Input_Device {
    device_id: string;
    label: string;
}

const PREFERRED_MIME_TYPES = [
    'audio/webm;codecs=opus',
    'audio/webm',
    'audio/ogg;codecs=opus',
    'audio/mp4',
];

/**
 * Classifies browser/WebView microphone errors into actionable, user-facing diagnostics.
 */
export function classify_mic_error(err: unknown): Mic_Capture_Error {
    if (typeof window !== 'undefined' && window.isSecureContext === false) {
        return {
            code: 'insecure_context',
            message: 'Microphone blocked on an insecure HTTP origin. Open Tadpole OS via localhost, 127.0.0.1, or HTTPS.',
            raw_name: 'SecurityError',
        };
    }

    const name = err instanceof Error ? err.name : typeof err === 'object' && err && 'name' in err ? String((err as { name: unknown }).name) : '';
    const raw_msg = err instanceof Error ? err.message : String(err || '');
    const lower = `${name} ${raw_msg}`.toLowerCase();

    if (name === 'NotAllowedError' || name === 'PermissionDeniedError' || lower.includes('permission denied') || lower.includes('not-allowed')) {
        return {
            code: 'permission_denied',
            message: 'Microphone permission denied. Allow microphone access in browser/WebView site settings and Windows Privacy → Microphone.',
            raw_name: name || 'NotAllowedError',
        };
    }

    if (name === 'NotFoundError' || name === 'DevicesNotFoundError' || lower.includes('requested device not found') || lower.includes('audio-capture')) {
        return {
            code: 'no_device',
            message: 'No microphone input device detected. Connect or enable an audio input device in OS sound settings.',
            raw_name: name || 'NotFoundError',
        };
    }

    if (name === 'NotReadableError' || name === 'TrackStartError' || lower.includes('could not start audio source')) {
        return {
            code: 'device_busy',
            message: 'Microphone is busy or locked by another application. Close other apps using the mic and retry.',
            raw_name: name || 'NotReadableError',
        };
    }

    if (name === 'NotSupportedError' || lower.includes('mediarecorder')) {
        return {
            code: 'unsupported_recorder',
            message: 'Audio recording codec is not supported by this browser/WebView.',
            raw_name: name || 'NotSupportedError',
        };
    }

    if (lower.includes('getusermedia') || lower.includes('mediadevices')) {
        return {
            code: 'media_devices_unavailable',
            message: 'Browser media capture API (navigator.mediaDevices) is unavailable in this environment.',
            raw_name: name || 'TypeError',
        };
    }

    return {
        code: 'unknown',
        message: raw_msg
            ? `Microphone capture failed (${name || 'Error'}: ${raw_msg}). Check browser/OS microphone permissions.`
            : 'Microphone capture failed. Check browser permission and audio device availability.',
        raw_name: name || undefined,
    };
}

export function select_supported_recorder_mime(): string | undefined {
    if (typeof MediaRecorder === 'undefined' || typeof MediaRecorder.isTypeSupported !== 'function') {
        return undefined;
    }
    return PREFERRED_MIME_TYPES.find((mime) => {
        try {
            return MediaRecorder.isTypeSupported(mime);
        } catch {
            return false;
        }
    });
}
