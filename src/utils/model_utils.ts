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
    "Gemini 3 Pro": "gemini-3-pro-preview",
    "Gemini 3.5 Pro": "gemini-3.5-pro",
    "Gemini 3.5 Flash": "gemini-3.5-flash",
    "Gemini 3.1 Pro": "gemini-3.1-pro-preview",
    "Gemini 3.1 Flash": "gemini-3.1-flash-preview",
    "Gemini 1.5 Pro": "gemini-1.5-pro",

    // OpenAI
    "GPT-5.5 Pro": "gpt-5.5-pro",
    "GPT-5.5": "gpt-5.5",
    "GPT-5.3 Codex": "gpt-5.3-codex",
    "GPT-5.2": "gpt-5.2-preview",
    "o4-mini": "o4-mini-2026-02",

    // Anthropic
    "Claude Fable 5": "claude-5-fable",
    "Claude Mythos 5": "claude-5-mythos",
    "Claude Opus 4.8": "claude-4.8-opus",
    "Claude Sonnet 4.5": "claude-4.5-sonnet",
    "Claude Sonnet 4": "claude-4-sonnet",

    // DeepSeek
    "DeepSeek V3.2": "deepseek-v3.2",
    "DeepSeek R1": "deepseek-r1",

    // Mistral
    "Mistral Medium 3": "mistral-medium-3",
    "Mixtral 8x22B": "mixtral-8x22b",
    "Pixtral 12B": "pixtral-12b-2409",

    // Meta / Local / Other
    "LLaMA 4 Maverick": "llama-4-maverick",
    "LLaMA 4 Scout": "llama-4-scout",
    "Llama 3.3 70B": "llama-3.3-70b-instruct",
    "Nemotron 3 Ultra": "nemotron-3-ultra",
    "MAI Thinking-1": "mai-thinking-1",
    "MAI Code-1-Flash": "mai-code-1-flash",
    "MAI Image-2.5": "mai-image-2.5",
    "Kimi K2.6": "kimi-k2.6",
    "Grok 4.1": "grok-4.1",
    "Qwen 3": "qwen-3",
    "GLM-4.7": "glm-4.7",
    "Ernie 5.0": "ernie-5.0",
    "Mercury-2": "0c217af4-0621-4d1c-9ceb-29340da09a07",
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
    
    // Explicit provider prefix routing (e.g. "openai:gpt-4")
    if (lower.includes(':')) {
        const parts = lower.split(':');
        const prefix = parts[0];
        const known_providers = ['openai', 'anthropic', 'google', 'gemini', 'groq', 'mistral', 'deepseek', 'xai', 'ollama-cloud'];
        if (known_providers.includes(prefix)) {
            return prefix === 'gemini' ? 'google' : prefix;
        }
        if (prefix === 'ollama') {
            return 'ollama';
        }
    }

    // 1. Core Provider Keywords (Priority)
    if (lower.includes('ollama-cloud') || lower.includes('-cloud') || lower.includes(':cloud')) return 'ollama-cloud';
    if (lower.includes('ollama') || lower.includes(':') || lower.includes('phi')) return 'ollama';
    if (lower.includes('gpt') || lower.includes('o4')) return 'openai';
    if (lower.includes('claude')) return 'anthropic';
    if (lower.includes('gemini')) return 'google';
    
    // 2. Secondary Vendors
    if (lower.includes('mistral') || lower.includes('pixtral') || lower.includes('mixtral')) return 'mistral';
    if (lower.includes('deepseek')) return 'deepseek';
    if (lower.includes('llama')) {
        if (lower.includes('groq') || lower.includes('versatile') || lower.includes('instant') || lower.includes('specdec')) return 'groq';
        return 'meta';
    }
    if (lower.includes('grok')) return 'xai';
    if (lower.includes('groq')) return 'groq';
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
    // STEP 1: Check multi-slot override FIRST (authoritative source)
    // The slot configs are the user's explicit selection and must take priority.
    if (agent.active_model_slot === 2 && agent.model_config2) {
        const c2 = agent.model_config2;
        const model_id = resolve_technical_model_id(c2.modelId || agent.model_2 || agent.model || 'gemini-1.5-flash');
        const provider = c2.provider || resolve_provider(model_id);
        console.debug(`[ModelUtils] Agent ${agent.name}: Slot 2 active → model=${model_id}, provider=${provider}`);
        return { model_id, provider };
    } else if (agent.active_model_slot === 3 && agent.model_config3) {
        const c3 = agent.model_config3;
        const model_id = resolve_technical_model_id(c3.modelId || agent.model_3 || agent.model || 'gemini-1.5-flash');
        const provider = c3.provider || resolve_provider(model_id);
        console.debug(`[ModelUtils] Agent ${agent.name}: Slot 3 active → model=${model_id}, provider=${provider}`);
        return { model_id, provider };
    }

    // STEP 2: Slot 1 — Check model_config first (explicit config), then fallback to agent.model
    const config = agent.model_config;
    const config_model_id = config?.modelId;
    const model_str = (config_model_id || agent.model || '').toLowerCase();
    const has_key = !!config?.apiKey;

    const is_custom_model = model_str.includes(':') || model_str.includes('/') || model_str.startsWith('ollama');

    const is_generic = (model_str === 'gemini-1.5-flash' || 
                       model_str === 'unknown' ||
                       model_str === 'gemini' ||
                       model_str === '' ||
                       (!agent.model && !config_model_id)) && !is_custom_model;

    const is_agent_default = (is_generic || 
                             (!!config_model_id && !has_key && config?.provider === 'google')) && !is_custom_model;
    
    // Determine base model_id: Use global override if agent is using generic system defaults
    let raw_id = config_model_id || agent.model || 'gemini-1.5-flash';
    if (is_agent_default && global_default_model) {
        console.debug(`[ModelUtils] Agent ${agent.name}: Overriding default to Global Intelligence: ${global_default_model}`);
        raw_id = global_default_model;
    }

    // Resolve final Model & Provider
    const model_id = resolve_technical_model_id(raw_id);
    
    // Priority: If we're overriding to global default, we MUST re-resolve the provider 
    // to avoid sticking with 'google' for an 'ollama' global default.
    const provider = (is_agent_default && global_default_model)
        ? resolve_provider(model_id)
        : (config?.provider || resolve_provider(model_id));

    console.debug(`[ModelUtils] Agent ${agent.name}: Slot 1 → model=${model_id}, provider=${provider} (config.modelId=${config_model_id}, agent.model=${agent.model})`);
    return { model_id, provider };
}

/**
 * Returns the display name of the model currently active in the agent's slots.
 */
export function get_active_model_name(agent: Agent): string {
    if (agent.active_model_slot === 2) return agent.model_2 || agent.model || 'Unknown';
    if (agent.active_model_slot === 3) return agent.model_3 || agent.model || 'Unknown';
    return agent.model || 'Unknown';
}


/**
 * Returns a Tailwind color class based on the model or provider.
 */
export function get_model_color(model_name: string): string {
    if (!model_name || typeof model_name !== 'string') return 'text-zinc-400 border-[color:var(--color-border)] bg-[color:var(--color-surface)]';
    const lower = model_name.toLowerCase();

    // OpenAI - Emerald/Green
    if (lower.includes('gpt') || lower.includes('o4')) return 'text-emerald-400 border-emerald-900 bg-emerald-900/10';

    // Anthropic - Zinc
    if (lower.includes('claude')) return 'text-zinc-400 border-[color:var(--color-surface)] bg-[color:var(--color-surface)]/10';

    // Google - Blue/Sky
    if (lower.includes('gemini')) return 'text-green-400 border-blue-900 bg-blue-900/10';

    // Groq - Amber/Orange
    if (lower.includes('groq') || lower.includes('llama')) return 'text-amber-400 border-amber-900 bg-amber-900/10';

    // DeepSeek - Cyan/Teal
    if (lower.includes('deepseek')) return 'text-cyan-400 border-cyan-900 bg-cyan-900/10';

    // xAI / Grok - Zinc/White
    if (lower.includes('grok')) return 'text-zinc-100 border-zinc-700 bg-zinc-800/50';

    return 'text-zinc-400 border-[color:var(--color-border)] bg-[color:var(--color-surface)]';
}

// Metadata: [model_utils]
