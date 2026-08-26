/**
 * @docs ARCHITECTURE:Infrastructure
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / providers
 * - **Primary Entrypoints**: `PROVIDERS`, `DEFAULT_PROVIDER`, `MODEL_IDS`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export const PROVIDERS = {
    GOOGLE: 'google',
    OPENAI: 'openai',
    ANTHROPIC: 'anthropic',
    GROQ: 'groq',
    OLLAMA: 'ollama',
    INCEPTION: 'inception',
    LOCAL: 'local',
} as const;

export const DEFAULT_PROVIDER = PROVIDERS.GOOGLE;

export const MODEL_IDS = {
    GEMINI_PRO: 'gemini-pro',
    GEMINI_FLASH: 'gemini-2.0-flash',
    CLAUDE_OPUS: 'claude-3-opus-20240229',
    GPT4_O: 'gpt-4o',
} as const;
