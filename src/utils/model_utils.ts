/**
 * @module modelUtils
 * Utility for mapping friendly display names to technical model IDs.
 */

const MODEL_MAP: Record<string, string> = {
    // Groq
    "Llama 3.3 70B (Groq)": "llama-3.3-70b-versatile",
    "Mixtral 8x7B (Groq)": "mixtral-8x7b-32768",

    // Google (Tadpole OS matches technically anyway, but for safety)
    "Gemini 1.5 Pro": "gemini-1.5-pro",
    "Gemini 1.5 Flash": "gemini-1.5-flash",
    "Gemini 3 Pro": "gemini-3-pro-preview",
    "Gemini 3 Flash": "gemini-3-flash-preview",
    "Gemini 3.1 Pro": "gemini-3.1-pro-preview",
    "Gemini 3.1 Flash": "gemini-3.1-flash-preview",

    // OpenAI
    "GPT-5.2": "gpt-5.2-preview",
    "GPT-4.1": "gpt-4.1-turbo",
    "o4-mini": "o4-mini-2026-02",

    // Anthropic
    "Claude Opus 4.5": "claude-4.5-opus",
    "Claude Sonnet 4.5": "claude-4.5-sonnet",
};

/**
 * Resolves a friendly model name into its technical ID.
 * Returns the original name if no mapping is found.
 */
export function resolve_technical_model_id(model_name: string | undefined): string {
    if (!model_name) return 'unknown';
    return MODEL_MAP[model_name] || model_name;
}

/**
 * Resolves the provider for a given model ID based on naming conventions.
 */
export function resolve_provider(modelId: string): string {
    const lower = (modelId || '').toLowerCase();
    if (lower.includes(':') || lower.includes('ollama') || lower.includes('phi')) return 'ollama';
    if (lower.includes('gpt') || lower.includes('o4')) return 'openai';
    if (lower.includes('claude')) return 'anthropic';
    if (lower.includes('gemini')) return 'google';
    if (lower.includes('llama')) {
        if (lower.includes('groq') || lower.includes('versatile') || lower.includes('instant') || lower.includes('specdec')) return 'groq';
        return 'meta';
    }
    if (lower.includes('grok')) return 'xai';
    if (lower.includes('groq')) return 'groq';
    if (lower.includes('deepseek')) return 'deepseek';
    if (lower.includes('mistral') || lower.includes('mixtral')) return 'mistral';
    if (lower.includes('qwen')) return 'alibaba';
    if (lower.includes('inception') || lower.includes('mercury')) return 'inception';
    return 'google'; // Global fallback for unknown models (system default)
}

import type { Agent } from '../types';

/**
 * Resolves the active model ID and provider for an agent based on its current slot.
 */
export function resolve_agent_model_config(agent: Agent): { model_id: string, provider: string } {
    let model_id = resolve_technical_model_id(agent.model_config?.model_id || agent.model || 'gemini-1.5-flash');
    let provider = agent.model_config?.provider || resolve_provider(model_id);

    if (agent.active_model_slot === 2 && agent.model_config2) {
        model_id = resolve_technical_model_id(agent.model_config2.model_id || agent.model2 || model_id);
        provider = agent.model_config2.provider || resolve_provider(model_id);
    } else if (agent.active_model_slot === 3 && agent.model_config3) {
        model_id = resolve_technical_model_id(agent.model_config3.model_id || agent.model3 || model_id);
        provider = agent.model_config3.provider || resolve_provider(model_id);
    }

    return { model_id, provider };
}

/**
 * Returns a Tailwind color class based on the model or provider.
 */
export function get_model_color(model_name: string): string {
    if (!model_name || typeof model_name !== 'string') return 'text-zinc-400 border-zinc-800 bg-zinc-900';
    const lower = model_name.toLowerCase();

    // OpenAI - Emerald/Green
    if (lower.includes('gpt') || lower.includes('o4')) return 'text-emerald-400 border-emerald-900 bg-emerald-900/10';

    // Anthropic - Zinc
    if (lower.includes('claude')) return 'text-zinc-400 border-zinc-900 bg-zinc-900/10';

    // Google - Blue/Sky
    if (lower.includes('gemini')) return 'text-blue-400 border-blue-900 bg-blue-900/10';

    // Groq - Amber/Orange
    if (lower.includes('groq') || lower.includes('llama')) return 'text-amber-400 border-amber-900 bg-amber-900/10';

    // DeepSeek - Cyan/Teal
    if (lower.includes('deepseek')) return 'text-cyan-400 border-cyan-900 bg-cyan-900/10';

    // xAI / Grok - Zinc/White
    if (lower.includes('grok')) return 'text-zinc-100 border-zinc-700 bg-zinc-800/50';

    return 'text-zinc-400 border-zinc-800 bg-zinc-900';
}
