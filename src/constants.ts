/**
 * @docs ARCHITECTURE:Infrastructure
 * 
 * ### AI Assist Note
 * **Root/Core**: Central registry for global enums, provider IDs, and default model configurations. 
 * Part of the Tadpole-OS core layer.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Incorrect provider string mapping (causes Vault lookup failure) or missing model IDs when a new service is integrated.
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


// Metadata: [constants]

// Metadata: [constants]
