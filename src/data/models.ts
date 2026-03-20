/**
 * @module models
 * Shared list of available AI models for selection across the dashboard.
 * Used by both the agent card model dropdowns and the AgentConfigPanel.
 * Last updated: Feb 2026
 */

/** Top 20 available AI models as of February 2026. */
export const MODEL_OPTIONS = [
    // OpenAI
    "GPT-5.2",
    "GPT-5.3 Codex",
    "GPT-4.1",
    "o4-mini",
    // Anthropic
    "Claude Opus 4.5",
    "Claude Sonnet 4.5",
    "Claude Sonnet 4",
    // Google
    "Gemini 3 Pro",
    "Gemini 3 Flash",
    // Meta
    "LLaMA 4 Maverick",
    "LLaMA 4 Scout",
    // Groq (Provider)
    "Llama 3.3 70B (Groq)",
    "Mixtral 8x7B (Groq)",
    // xAI
    "Grok 4.1",
    // Google
    "Gemini 1.5 Flash",
    // DeepSeek
    "DeepSeek V3.2",
    "DeepSeek R1",
    // Mistral
    "Mistral Medium 3",
    "Mixtral 8x22B",
    // Open-source / Other
    "Qwen 3",
    "GLM-4.7",
    "Ernie 5.0",
] as const;

/** Union type of all valid model names. */
export type ModelName = typeof MODEL_OPTIONS[number];
