/**
 * @docs ARCHITECTURE:Infrastructure
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / models
 * - **Primary Entrypoints**: `MODEL_OPTIONS`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export const MODEL_OPTIONS = [
    // OpenAI
    "GPT-5.5 Pro",
    "GPT-5.5",
    "GPT-5.3 Codex",
    "GPT-5.2",
    "o4-mini",
    // Anthropic
    "Claude Fable 5",
    "Claude Mythos 5",
    "Claude Opus 4.8",
    "Claude Sonnet 4.5",
    "Claude Sonnet 4",
    // Google
    "Gemini 3.5 Pro",
    "Gemini 3.5 Flash",
    "Gemini 3.1 Pro",
    "Gemini 3.1 Flash",
    "Gemini 3 Pro",
    // Meta
    "LLaMA 4 Maverick",
    "LLaMA 4 Scout",
    "Llama 3.3 70B",
    // Groq (Provider)
    "Llama 3.3 70B (Groq)",
    "Mixtral 8x7B (Groq)",
    // DeepSeek
    "DeepSeek V3.2",
    "DeepSeek R1",
    // Mistral
    "Mistral Medium 3",
    "Mixtral 8x22B",
    "Pixtral 12B",
    // Open-source / Other / Local
    "Nemotron 3 Ultra",
    "MAI Thinking-1",
    "MAI Code-1-Flash",
    "MAI Image-2.5",
    "Kimi K2.6",
    "Grok 4.1",
    "Qwen 3",
    "GLM-4.7",
    "Ernie 5.0",
    "Mercury-2",
    "gemma4:31b-cloud",
] as const;
