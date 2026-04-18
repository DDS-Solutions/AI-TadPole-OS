/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Zustand State**: Intelligence Forge inventory and model routing orchestrator. 
 * Manages model availability, modality mapping (LLM/Vision/Voice), and cross-tab inventory synchronization via `BroadcastChannel`.
 * Integrates **IMR-01 (Intelligent Model Registry)** capabilities for automated provider model discovery.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Inventory desync between tabs (check `BroadcastChannel` heartbeat), or modality routing failure during agent initialization.
 * - **IMR-01 Link**: See `sync_models` for ingestion of backend inferred capabilities.
 * - **Telemetry Link**: Search for `[ModelStore]` or `MODELS_SYNC` in UI logs.
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { Crypto_Service } from '../services/crypto_service';
import { MODEL_OPTIONS } from '../data/models';
import { PROVIDERS } from '../constants';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { log_error } from '../services/system_utils';

const SYNC_CHANNEL = 'tadpole-os-sync';
const sync_channel = typeof window !== 'undefined' ? new BroadcastChannel(SYNC_CHANNEL) : null;

export interface Model_Entry {
    id: string;
    name: string;
    provider: string; // Master provider ID used in frontend
    provider_id?: string; // Mapped from backend JSON if exists
    modality: 'llm' | 'vision' | 'voice' | 'reasoning' | string;
    rpm?: number;
    rpd?: number;
    tpm?: number;
    tpd?: number;
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
    add_model: (name: string, provider: string, modality?: Model_Entry['modality'], limits?: Partial<Pick<Model_Entry, 'rpm' | 'rpd' | 'tpm' | 'tpd'>>) => Promise<void>;
    edit_model: (id: string, name: string, provider: string, modality: Model_Entry['modality'], limits?: Partial<Pick<Model_Entry, 'rpm' | 'rpd' | 'tpm' | 'tpd'>>) => Promise<void>;
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

            add_model: async (name, provider, modality = 'llm', limits) => {
                const id = Crypto_Service.generate_id();
                const new_model = {
                    id,
                    name,
                    provider,
                    modality,
                    rpm: limits?.rpm ?? 10,
                    tpm: limits?.tpm ?? 100000,
                    rpd: limits?.rpd ?? 1000,
                    tpd: limits?.tpd ?? 10000000
                };

                set(state => ({
                    models: [...state.models, new_model]
                }));

                try {
                    await tadpole_os_service.update_model(id, {
                        ...new_model,
                        provider_id: provider
                    });
                } catch (error) {
                    log_error('ModelStore', 'Model Sync Failed', error);
                }
            },

            edit_model: async (id, name, provider, modality, limits) => {
                set(state => ({
                    models: (state.models || []).map(m => m.id === id ? { ...m, name, provider, modality, ...limits } : m)
                }));

                try {
                    const model = (get().models || []).find(m => m.id === id);
                    if (model) {
                        await tadpole_os_service.update_model(id, {
                            ...model,
                            provider_id: provider
                        });
                    }
                } catch (error) {
                    log_error('ModelStore', 'Model Update Sync Failed', error);
                }
            },

            delete_model: async (id) => {
                set(state => {
                    const new_deleting = new Set(state.deleting_ids || []);
                    new_deleting.add(id);
                    return {
                        models: (state.models || []).filter(m => m.id !== id),
                        deleting_ids: new_deleting
                    };
                });

                try {
                    await tadpole_os_service.delete_model(id);
                    setTimeout(() => {
                        set(state => {
                            const new_deleting = new Set(state.deleting_ids);
                            new_deleting.delete(id);
                            return { deleting_ids: new_deleting };
                        });
                    }, 5000);
                } catch (error) {
                    log_error('ModelStore', 'Model Deletion Sync Failed', error);
                    set(state => {
                        const new_deleting = new Set(state.deleting_ids);
                        new_deleting.delete(id);
                        return { deleting_ids: new_deleting };
                    });
                }
            },

            sync_models: async () => {
                try {
                    const raw_models = (await tadpole_os_service.get_models()) || [];
                    const backend_models: Model_Entry[] = (raw_models as Record<string, unknown>[]).map(bm => ({
                        ...bm,
                        id: String(bm.id),
                        name: String(bm.name || bm.id),
                        provider: String(bm.provider || bm.provider_id || bm.providerId || 'local'),
                        provider_id: String(bm.provider_id || bm.providerId || bm.provider || 'local'),
                        modality: String(bm.modality || 'llm'),
                        input_tokens: bm.input_tokens as number | undefined,
                        output_tokens: bm.output_tokens as number | undefined,
                        capabilities: bm.capabilities as Model_Entry['capabilities']
                    } as Model_Entry));
                    
                    const models = get().models || [];
                    const deleting_ids = get().deleting_ids || new Set();
                    const filtered_models = models.filter(m => 
                        (backend_models || []).some(bm => bm.id === m.id) || deleting_ids.has(m.id)
                    );

                    const final_models = [...filtered_models];
                    let changed = filtered_models.length !== models.length;

                    backend_models.forEach((bm) => {
                        const existing = final_models.find(m => m.id === bm.id);
                        if (!existing && !deleting_ids.has(bm.id)) {
                            final_models.push(bm);
                            changed = true;
                        } else if (existing) {
                            // Authoritative Update: Backend always wins for routing critical fields
                            if (existing.provider !== bm.provider || 
                                existing.modality !== bm.modality || 
                                existing.name !== bm.name ||
                                JSON.stringify(existing.capabilities) !== JSON.stringify(bm.capabilities)) {
                                Object.assign(existing, bm);
                                changed = true;
                            }
                        }
                    });

                    if (changed) {
                        set({ models: final_models });
                    }
                } catch (error) {
                    log_error('ModelStore', 'Backend Sync Failed', error);
                }
            },

            sync_defaults: (providers_length) => {
                const models = get().models || [];
                if (providers_length === 0 && models.length === 0) {
                    set({ models: INITIAL_MODELS });
                    return;
                }

                const updated_models = (models || []).map(m => {
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
            migrate: (persisted_state: unknown) => {
                const state = (persisted_state || {}) as { models?: Record<string, unknown>[] };
                const models = state.models || [];

                return {
                    ...state,
                    models: (models || []).map(m => ({
                        ...m,
                        provider_id: m.provider_id ?? m.providerId,
                        input_tokens: m.input_tokens ?? m.input_tokens,
                        output_tokens: m.output_tokens ?? m.output_tokens
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
        const { type, payload } = event.data;
        if (type === 'models:sync') {
            // Update fingerprint BEFORE setting state to prevent local subscribe from echoing back
            last_broadcast = JSON.stringify(payload);
            use_model_store.setState({ models: payload });
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

