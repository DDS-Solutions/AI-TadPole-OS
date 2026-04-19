/**
 * @docs ARCHITECTURE:Relational_Knowledge
 * @docs OPERATIONS_MANUAL:Models
 * 
 * ### AI Assist Note
 * **Model Resolver**: Central utility for normalizing friendly model names into technical IDs and resolving providers. 
 * Maps latest Gemini 3.1, GPT-5.2, and Claude 4.5 Sonnet IDs for backend parity.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Model resolution mismatch leading to 404 (wrong ID), or incorrect provider detection causing API key routing errors.
 * - **Telemetry Link**: Look for `resolve_technical_model_id` in call stacks when model switching fails.
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
export function resolve_provider(model_id: string): string {
    const lower = (model_id || '').toLowerCase();
    
    // 1. Core Provider Keywords (Priority)
    if (lower.includes('ollama') || lower.includes(':') || lower.includes('phi')) return 'ollama';
    if (lower.includes('gpt') || lower.includes('o4')) return 'openai';
    if (lower.includes('claude')) return 'anthropic';
    if (lower.includes('gemini')) return 'google';
    
    // 2. Secondary Vendors
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
    
    // 3. Fallback: Default to 'ollama' if we're in a "Global Default" state, otherwise 'google'
    if (lower.includes('global default')) return 'ollama';
    return 'google'; // System default fallback
}

import type { Agent } from '../types';

/**
 * Resolves the active model ID and provider for an agent based on its current slot.
 * Supports global intelligence overrides to ensure swarm-wide synchronization.
 */
export function resolve_agent_model_config(agent: Agent, global_default_model?: string): { model_id: string, provider: string } {
    const model_str = (agent.model || '').toLowerCase();
    
    // aggressive Default Detection: 
    // 1. Generic Gemini models (Flash, Pro, etc.)
    // 2. Agents with no specialized model config
    // 3. Agents with NO API KEY AND (Google Provider OR Gemini Model)
    //    - This targets legacy system defaults without breaking specialized GPT/Claude agents.
    const is_agent_default = !agent.model_config?.model_id || 
                             (!agent.model_config?.api_key && (agent.model_config?.provider === 'google' || model_str.includes('gemini'))) ||
                             model_str === 'unknown' || 
                             model_str === 'gemini-1.5-flash' || 
                             model_str.includes('gemini');
    
    // 1. Determine base model_id: Use global override if agent is using generic system defaults
    let raw_id = agent.model_config?.model_id || agent.model || 'gemini-1.5-flash';
    if (is_agent_default && global_default_model) {
        console.debug(`[ModelUtils] Overriding Default Agent ${agent.name} to Global Intelligence: ${global_default_model}`);
        raw_id = global_default_model;
    }

    // 2. Resolve final Model & Provider
    let model_id = resolve_technical_model_id(raw_id);
    
    // Priority: If we're overriding to global default, we MUST re-resolve the provider 
    // to avoid sticking with 'google' for an 'ollama' global default.
    let provider = (is_agent_default && global_default_model)
        ? resolve_provider(model_id)
        : (agent.model_config?.provider || resolve_provider(model_id));

    // 2. Handle Multi-Slot Overrides
    if (agent.active_model_slot === 2 && agent.model_config2) {
        model_id = resolve_technical_model_id(agent.model_config2.model_id || agent.model_2 || model_id);
        provider = agent.model_config2.provider || resolve_provider(model_id);
    } else if (agent.active_model_slot === 3 && agent.model_config3) {
        model_id = resolve_technical_model_id(agent.model_config3.model_id || agent.model_3 || model_id);
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
