/**
 * @docs ARCHITECTURE:AudioIntelligence
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / voice/tts_engine
 * - **Primary Entrypoints**: `speak_browser`, `speak_premium`
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[VoiceClient]`
 */

import { system_api_service } from '../system_api_service';

function select_browser_voice(
    voices: SpeechSynthesisVoice[],
    base_lang: string,
    preferred_voice_name?: string
): SpeechSynthesisVoice | undefined {
    const matches_locale = (v: SpeechSynthesisVoice) =>
        Boolean(v.lang && v.lang.toLowerCase().startsWith(base_lang));
    const is_preferred_quality = (v: SpeechSynthesisVoice) =>
        v.name.includes('Google') ||
        v.name.includes('Natural') ||
        v.name.includes('Premium') ||
        v.name.includes('Samantha');

    return (
        (preferred_voice_name ? voices.find((v) => v.name === preferred_voice_name) : undefined) ||
        voices.find((v) => matches_locale(v) && is_preferred_quality(v)) ||
        voices.find((v) => matches_locale(v) && v.default) ||
        voices.find(matches_locale) ||
        voices.find((v) => v.default)
    );
}

/** Speaks text via the browser Web SpeechSynthesis API with locale-aware voice matching. */
export function speak_browser(text: string, preferred_voice_name?: string): Promise<void> {
    if (!window.speechSynthesis) return Promise.resolve();

    const target_lang = (document.documentElement?.lang || navigator.language || 'en-US').trim() || 'en-US';
    const base_lang = target_lang.split('-')[0].toLowerCase();

    const utterance = new SpeechSynthesisUtterance(text);
    utterance.lang = target_lang;
    utterance.rate = 1.0;
    utterance.pitch = 1.0;
    utterance.volume = 1.0;

    const voice = select_browser_voice(window.speechSynthesis.getVoices(), base_lang, preferred_voice_name);
    if (voice) {
        utterance.voice = voice;
        if (voice.lang) {
            utterance.lang = voice.lang;
        }
    }

    return new Promise((resolve) => {
        let done = false;
        const finish = () => {
            if (done) return;
            done = true;
            resolve();
        };

        utterance.onend = finish;
        utterance.onerror = finish;
        window.speechSynthesis.speak(utterance);

        // Safety fallback for synthetic environments where onend never fires
        if (typeof utterance.addEventListener !== 'function') {
            finish();
        }
    });
}

function play_audio_blob(audio_blob: Blob): Promise<void> {
    let audio_url: string | null = URL.createObjectURL(audio_blob);
    const audio = new Audio(audio_url);

    return new Promise<void>((resolve, reject) => {
        const cleanup = () => {
            if (audio_url) {
                URL.revokeObjectURL(audio_url);
                audio_url = null;
            }
        };
        audio.onended = () => {
            cleanup();
            resolve();
        };
        audio.onerror = () => {
            cleanup();
            reject(new Error('Audio element playback error'));
        };
        audio.play().catch((play_err) => {
            cleanup();
            reject(play_err);
        });
    });
}

/** Synthesizes audio via `/v1/engine/speak` (OpenAI or Piper), falling back to browser TTS on error. */
export async function speak_premium(
    text: string,
    voice_id: string | undefined,
    engine: 'openai' | 'piper',
    ensure_piper_stream: () => void
): Promise<void> {
    try {
        if (engine === 'piper') {
            ensure_piper_stream();
        }
        const audio_blob = await system_api_service.engine.speak(text, voice_id, engine);

        if (audio_blob.type && audio_blob.type.toLowerCase().includes('application/json')) {
            const raw = await audio_blob.text().catch(() => '');
            if (raw.includes('browser_fallback')) {
                await speak_browser(text, voice_id);
            }
            return;
        }

        await play_audio_blob(audio_blob);
    } catch (err) {
        console.error('[VoiceClient] Premium TTS error:', err);
        await speak_browser(text, voice_id);
    }
}
