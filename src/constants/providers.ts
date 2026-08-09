/**
 * @docs ARCHITECTURE:Infrastructure
 * 
 * ### AI Assist Note
 * **Providers & Models**: Central registry for global provider IDs and default model configurations.
 * GAP-FE-01: Extracted from root `constants.ts` into `constants/providers.ts`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Incorrect provider string mapping or missing model IDs.
 * - **Telemetry Link**: Search for `[Constants]` in source audits.
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

// Metadata: [providers_constants]
