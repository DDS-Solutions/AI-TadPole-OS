/**
 * @docs ARCHITECTURE:State
 *
 * ### AI Assist Note
 * - Keep side effects at service boundaries and preserve the public contracts of shared domain transitions.
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / model_store
 * - **Primary Entrypoints**: `use_model_store`, `Model_Entry`, `Model_State`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { Crypto_Service } from '../services/crypto_service';
import { MODEL_OPTIONS } from '../data/models';
import { PROVIDERS } from '../constants/providers';
import { create_model_commands } from '../services/model_commands';

const SYNC_CHANNEL = 'tadpole-os-sync';
const sync_channel = typeof window !== 'undefined' ? new BroadcastChannel(SYNC_CHANNEL) : null;

export interface Model_Entry {
    id: string;
    name: string;
    provider: string; // Master provider ID used in frontend
    provider_id?: string; // Mapped from backend JSON if exists
    modality: 'llm' | 'vision' | 'voice' | 'reasoning' | 'image' | 'embedding';
    rpm?: number;
    rpd?: number;
    tpm?: number;
    tpd?: number;
    context_window?: number;
    input_cost_per_m?: number;
    output_cost_per_m?: number;
    input_tokens?: number;
    output_tokens?: number;
    capabilities?: {
        supports_tools: boolean;
        supports_vision: boolean;
        supports_structured_output: boolean;
        supports_reasoning: boolean;
        context_window: number;
        max_output_tokens: number;
    };
}

export interface Model_State {
    models: Model_Entry[];
    deleting_ids: Set<string>;

    // Actions
    add_model: (name: string, provider: string, modality?: Model_Entry['modality'], limits?: Partial<Pick<Model_Entry, 'rpm' | 'rpd' | 'tpm' | 'tpd' | 'context_window' | 'input_cost_per_m' | 'output_cost_per_m'>>) => Promise<void>;
    edit_model: (id: string, name: string, provider: string, modality: Model_Entry['modality'], limits?: Partial<Pick<Model_Entry, 'rpm' | 'rpd' | 'tpm' | 'tpd' | 'context_window' | 'input_cost_per_m' | 'output_cost_per_m'>>) => Promise<void>;
    delete_model: (id: string) => Promise<void>;
    sync_models: () => Promise<void>;
    sync_defaults: (providers_length: number) => void;
}

// Initial models from static list
const INITIAL_MODELS: Model_Entry[] = (MODEL_OPTIONS || []).map(m => {
    let provider: string = PROVIDERS.LOCAL;
    let modality: Model_Entry['modality'] = 'llm';
    const lower = (m || '').toLowerCase();

    if (lower.includes('gpt') || lower.includes('o4')) provider = PROVIDERS.OPENAI;
    else if (lower.includes('claude')) provider = PROVIDERS.ANTHROPIC;
    else if (lower.includes('gemini')) provider = PROVIDERS.GOOGLE;
    else if (lower.includes('llama')) {
        if (lower.includes('groq') || lower.includes('versatile') || lower.includes('instant')) provider = PROVIDERS.GROQ;
        else provider = 'meta';
    }
    else if (lower.includes('grok')) provider = 'xai';
    else if (lower.includes('groq')) provider = PROVIDERS.GROQ;

    // Heuristic for modality
    if (lower.includes('vision') || lower.includes('flash') || lower.includes('pro')) modality = 'vision';
    if (lower.includes('audio') || lower.includes('voice') || lower.includes('tts')) modality = 'voice';
    if (lower.includes('coder') || lower.includes('reasoning') || lower.includes('o1') || lower.includes('o2') || lower.includes('o3') || lower.includes('o4') || lower.includes('r1') || lower.includes('thought')) modality = 'reasoning';

    return { id: Crypto_Service.generate_id(), name: m, provider, modality };
});

/**
 * use_model_store
 * Inventory management for neural models available to the swarm.
 * Refactored for strict snake_case compliance for backend parity.
 */
export const use_model_store = create<Model_State>()(
    persist(
        (set, get) => ({
            models: INITIAL_MODELS,
            deleting_ids: new Set(),

            ...create_model_commands({ set, get }),

            sync_defaults: (providers_length) => {
                const models = get().models || [];
                if (providers_length === 0 && models.length === 0) {
                    set({ models: INITIAL_MODELS });
                    return;
                }

                const updated_models = (models || []).map(m => {
                    // Only infer provider if currently missing or set to placeholder 'local'
                    if (m.provider) return m;
                    const lower = m.name.toLowerCase();
                    let new_provider = m.provider;
                    if (lower.includes('claude')) new_provider = PROVIDERS.ANTHROPIC;
                    else if (lower.includes('gemini')) new_provider = PROVIDERS.GOOGLE;
                    else if (lower.includes('llama') && (lower.includes('groq') || lower.includes('versatile'))) new_provider = PROVIDERS.GROQ;

                    if (new_provider !== m.provider) {
                        return { ...m, provider: new_provider };
                    }
                    return m;
                });

                if (JSON.stringify(updated_models) !== JSON.stringify(models)) {
                    set({ models: updated_models });
                }
            }
        }),
        {
            name: 'tadpole-model-inventory',
            partialize: (state) => ({
                models: state.models,
            }),
            version: 1,
            migrate: (persisted_state: unknown) => {
                const state = (persisted_state || {}) as { models?: Record<string, unknown>[] };
                const models = state.models || [];

                return {
                    ...state,
                    models: (models || []).map(m => ({
                        ...m,
                        provider_id: m.provider_id ?? m.providerId,
                        input_tokens: m.input_tokens ?? m.inputTokens,
                        output_tokens: m.output_tokens ?? m.outputTokens
                    }))
                } as unknown as Record<string, unknown>;
            }
        }
    )
);

// ── Cross-Tab Synchronization ────────────────────────────────
if (sync_channel) {
    // Initial fingerprint
    let last_broadcast = JSON.stringify(use_model_store.getState().models);

    sync_channel.onmessage = (event) => {
        if (!event || !event.data || typeof event.data !== 'object') return;
        const { type, payload } = event.data;
        if (type === 'models:sync' && Array.isArray(payload)) {
            const valid_models = payload.filter((m): m is Model_Entry => Boolean(m && typeof m === 'object' && typeof m.id === 'string' && typeof m.name === 'string'));
            // Update fingerprint BEFORE setting state to prevent local subscribe from echoing back
            last_broadcast = JSON.stringify(valid_models);
            use_model_store.setState({ models: valid_models });
        }
    };

    use_model_store.subscribe((state) => {
        const current = JSON.stringify(state.models);
        if (current !== last_broadcast) {
            last_broadcast = current;
            sync_channel.postMessage({ type: 'models:sync', payload: state.models });
        }
    });
}


import { register_store_for_test_reset } from '../utils/store_registry';

register_store_for_test_reset(() => {
    use_model_store.setState(use_model_store.getInitialState(), true);
});
