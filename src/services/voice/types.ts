/**
 * @docs ARCHITECTURE:AudioIntelligence
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / voice/types
 * - **Primary Entrypoints**: `Voice_Status`, `Speech_Recognition_Interface`
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

/** Minimal typed shape for a single SpeechRecognitionAlternative. */
export interface Speech_Alternative {
    readonly transcript: string;
}

/** Minimal typed shape for a SpeechRecognitionResult. */
export interface Speech_Result {
    readonly length: number;
    [index: number]: Speech_Alternative;
}

/** Minimal typed shape for SpeechRecognitionResultList. */
export interface Speech_Result_List {
    readonly length: number;
    [index: number]: Speech_Result;
}

/** Cross-browser SpeechRecognition interface abstraction. */
export interface Speech_Recognition_Interface {
    continuous: boolean;
    interimResults: boolean;
    lang: string;
    onresult: (event: { results: Speech_Result_List }) => void;
    onerror: (event: { error: string }) => void;
    onend: () => void;
    start: () => void;
    stop: () => void;
    abort?: () => void;
}

/** Voice_Status: Current state of the voice recognition/synthesis system. */
export type Voice_Status = 'idle' | 'initializing' | 'active' | 'stalled';

/** Supported text-to-speech synthesis engines accepted by Voice_Client.speak(). */
export type Voice_Tts_Engine = 'browser' | 'openai' | 'groq' | 'piper' | 'gemini-live';
