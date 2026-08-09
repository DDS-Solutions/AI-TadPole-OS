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

import type { Agent } from '../types';
import { track_agent_slot_swap } from './telemetry';

/**
 * Normalized model map (lowercase keys for case-insensitive resolution).
 */
const MODEL_MAP: Record<string, string> = {
    // Groq
    "llama 3.3 70b (groq)": "llama-3.3-70b-versatile",
    "mixtral 8x7b (groq)": "mixtral-8x7b-32768",

    // Google
    "gemini 3 pro": "gemini-3-pro-preview",
    "gemini 3.5 pro": "gemini-3.5-pro",
    "gemini 3.5 flash": "gemini-3.5-flash",
    "gemini 3.1 pro": "gemini-3.1-pro-preview",
    "gemini 3.1 flash": "gemini-3.1-flash-preview",
    "gemini 1.5 pro": "gemini-1.5-pro",

    // OpenAI
    "gpt-5.5 pro": "gpt-5.5-pro",
    "gpt-5.5": "gpt-5.5",
    "gpt-5.3 codex": "gpt-5.3-codex",
    "gpt-5.2": "gpt-5.2-preview",
    "o4-mini": "o4-mini-2026-02",

    // Anthropic
    "claude fable 5": "claude-5-fable",
    "claude mythos 5": "claude-5-mythos",
    "claude opus 4.8": "claude-4.8-opus",
    "claude sonnet 4.5": "claude-4.5-sonnet",
    "claude sonnet 4": "claude-4-sonnet",

    // DeepSeek
    "deepseek v3.2": "deepseek-v3.2",
    "deepseek r1": "deepseek-r1",

    // Mistral
    "mistral medium 3": "mistral-medium-3",
    "mixtral 8x22b": "mixtral-8x22b",
    "pixtral 12b": "pixtral-12b-2409",

    // Meta / Local / Other
    "llama 4 maverick": "llama-4-maverick",
    "llama 4 scout": "llama-4-scout",
    "llama 3.3 70b": "llama-3.3-70b-instruct",
    "nemotron 3 ultra": "nemotron-3-ultra",
    "mai thinking-1": "mai-thinking-1",
    "mai code-1-flash": "mai-code-1-flash",
    "mai image-2.5": "mai-image-2.5",
    "kimi k2.6": "kimi-k2.6",
    "grok 4.1": "grok-4.1",
    "qwen 3": "qwen-3",
    "glm-4.7": "glm-4.7",
    "ernie 5.0": "ernie-5.0",
    "mercury-2": "0c217af4-0621-4d1c-9ceb-29340da09a07",
};

/**
 * Resolves a friendly model name into its technical ID (case-insensitive).
 */
export function resolve_technical_model_id(model_name: string | undefined): string {
    if (!model_name) return 'unknown';
    const normalized = model_name.toLowerCase().trim();
    return MODEL_MAP[normalized] || model_name;
}

export const resolveTechnicalModelId = resolve_technical_model_id;

/**
 * Resolves the provider for a given model ID based on naming conventions.
 */
export function resolve_provider(model_id: string): string {
    const lower = (model_id || '').toLowerCase().trim();

    // 1. Explicit provider prefix routing (e.g. "openai:gpt-4")
    if (lower.includes(':')) {
        const parts = lower.split(':');
        const prefix = parts[0];
        const known_providers = ['openai', 'anthropic', 'google', 'gemini', 'groq', 'mistral', 'deepseek', 'xai', 'ollama-cloud', 'ollama'];
        if (known_providers.includes(prefix)) {
            return prefix === 'gemini' ? 'google' : prefix;
        }
    }

    // 2. Core Provider Keywords
    if (lower.includes('ollama-cloud') || lower.includes('-cloud') || lower.includes(':cloud')) return 'ollama-cloud';
    if (lower.includes('gpt') || lower.includes('o4')) return 'openai';
    if (lower.includes('claude')) return 'anthropic';
    if (lower.includes('gemini')) return 'google';

    // 3. Secondary Vendors
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
    if (lower.includes('ollama') || lower.includes('phi')) return 'ollama';

    // 4. Fallback
    if (lower.includes('global default')) return 'ollama';
    return 'google';
}

export const resolveProvider = resolve_provider;

export function parse_active_model_slot(val: unknown): 1 | 2 | 3 {
    if (typeof val === 'number') {
        if (val === 2) return 2;
        if (val === 3) return 3;
        return 1;
    }
    if (typeof val === 'string') {
        const trimmed = val.trim().toLowerCase();
        if (trimmed === '2' || trimmed === 'execution' || trimmed === 'secondary') return 2;
        if (trimmed === '3' || trimmed === 'default' || trimmed === 'tertiary') return 3;
        if (trimmed === '1' || trimmed === 'planning' || trimmed === 'primary') return 1;
        const num = parseInt(trimmed, 10);
        if (num === 2) return 2;
        if (num === 3) return 3;
    }
    return 1;
}

export const parseActiveModelSlot = parse_active_model_slot;

/**
 * Resolves the active model ID and provider for an agent based on its current slot.
 */
export function resolve_agent_model_config(agent: Agent, global_default_model?: string): { model_id: string, provider: string } {
    const active_slot = parse_active_model_slot(agent.active_model_slot);

    // DRY Slot Resolution for Slots 2 & 3
    const slot_configs = [
        { slot: 2, config: agent.model_config2, fallback: agent.model_2 },
        { slot: 3, config: agent.model_config3, fallback: agent.model_3 },
    ];

    for (const item of slot_configs) {
        if (active_slot === item.slot && (item.config || item.fallback)) {
            const raw_model = item.config?.modelId || item.fallback || agent.model || 'gemini-1.5-flash';
            const model_id = resolve_technical_model_id(raw_model);
            const provider = item.config?.provider || resolve_provider(model_id);
            console.debug(`[ModelUtils] Agent ${agent.name}: Slot ${item.slot} active → model=${model_id}, provider=${provider}`);
            return { model_id, provider };
        }
    }

    // Slot 1 Logic
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

    let raw_id = config_model_id || agent.model || 'gemini-1.5-flash';
    if (is_agent_default && global_default_model) {
        console.debug(`[ModelUtils] Agent ${agent.name}: Overriding default to Global Intelligence: ${global_default_model}`);
        raw_id = global_default_model;
    }

    const model_id = resolve_technical_model_id(raw_id);
    const provider = (is_agent_default && global_default_model)
        ? resolve_provider(model_id)
        : (config?.provider || resolve_provider(model_id));

    console.debug(`[ModelUtils] Agent ${agent.name}: Slot 1 → model=${model_id}, provider=${provider} (config.modelId=${config_model_id}, agent.model=${agent.model})`);
    return { model_id, provider };
}

export const resolveAgentModelConfig = resolve_agent_model_config;

/**
 * Returns the display name of the model currently active in the agent's slots.
 */
export function get_active_model_name(agent: Agent): string {
    const slot = parse_active_model_slot(agent.active_model_slot);
    if (slot === 2) return agent.model_2 || agent.model || 'Unknown';
    if (slot === 3) return agent.model_3 || agent.model || 'Unknown';
    return agent.model || 'Unknown';
}

export const getActiveModelName = get_active_model_name;

/**
 * Returns a Tailwind color class based on the model or provider.
 */
export function get_model_color(model_name: string): string {
    if (!model_name || typeof model_name !== 'string') return 'text-zinc-400 border-[color:var(--color-border)] bg-[color:var(--color-surface)]';
    const lower = model_name.toLowerCase();

    if (lower.includes('gpt') || lower.includes('o4')) return 'text-emerald-400 border-emerald-900 bg-emerald-900/10';
    if (lower.includes('claude')) return 'text-zinc-400 border-[color:var(--color-surface)] bg-[color:var(--color-surface)]/10';
    if (lower.includes('gemini')) return 'text-green-400 border-blue-900 bg-blue-900/10';
    if (lower.includes('groq') || lower.includes('llama')) return 'text-amber-400 border-amber-900 bg-amber-900/10';
    if (lower.includes('deepseek')) return 'text-cyan-400 border-cyan-900 bg-cyan-900/10';
    if (lower.includes('grok')) return 'text-zinc-100 border-zinc-700 bg-zinc-800/50';

    return 'text-zinc-400 border-[color:var(--color-border)] bg-[color:var(--color-surface)]';
}

export const getModelColor = get_model_color;

/**
 * Resolves a prioritized chain of fallback model configurations for zero-downtime execution.
 */
export function resolve_fallback_provider_chain(agent: Agent, primary_model?: string): Array<{ model: string; provider: string }> {
    const chain: Array<{ model: string; provider: string }> = [];
    const primary = primary_model || get_active_model_name(agent);

    chain.push({
        model: resolve_technical_model_id(primary),
        provider: resolve_provider(primary)
    });

    const candidates = [agent.model_2, agent.model_3];
    for (const cand of candidates) {
        if (cand && cand !== primary && !chain.some(c => c.model === resolve_technical_model_id(cand))) {
            chain.push({
                model: resolve_technical_model_id(cand),
                provider: resolve_provider(cand)
            });
        }
    }

    if (!chain.some(c => c.model.includes('flash'))) {
        chain.push({
            model: 'gemini-3.5-flash',
            provider: 'google'
        });
    }

    return chain;
}

export const resolveFallbackProviderChain = resolve_fallback_provider_chain;
export { track_agent_slot_swap };

// Metadata: [model_utils]
