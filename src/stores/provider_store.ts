/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Zustand State**: Intelligence Provider registry and connectivity manager. 
 * Refactored to be a pure state container. Side-effects are handled by `provider_service`.
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { PROVIDERS } from '../constants';
import type { Model_Entry } from './model_store';
import { use_model_store } from './model_store';

export type { Model_Entry };

export interface Provider_Config {
    id: string;
    name: string;
    icon?: string;
    api_key?: string;
    base_url?: string;
    external_id?: string;
    protocol?: 'openai' | 'anthropic' | 'google' | 'ollama' | 'deepseek' | 'groq' | 'local' | 'xai' | 'together' | 'mistral' | 'openrouter' | 'custom';
    custom_headers?: Record<string, string>;
    audio_model?: string;
    persist_to_engine?: boolean;
    supports_steering_vectors?: boolean;
    metadata?: Record<string, unknown>;
}

export interface Provider_State {
    providers: Provider_Config[];
    base_urls: Record<string, string>;
    deleting_ids: Set<string>;

    // Pure State Mutations
    sync_defaults: () => void;
}

const DEFAULT_PROVIDERS: Provider_Config[] = [
    { id: PROVIDERS.OPENAI, name: 'OpenAI', icon: '⚡' },
    { id: PROVIDERS.ANTHROPIC, name: 'Anthropic', icon: '🏺' },
    { id: PROVIDERS.GOOGLE, name: 'Google Vertex', icon: '☁️' },
    { id: PROVIDERS.GROQ, name: 'Groq', icon: '⚡' },
    { id: PROVIDERS.OLLAMA, name: 'Ollama', icon: '🦙' },
    { id: 'meta', name: 'Meta / Llama', icon: '🦙' },
    { id: PROVIDERS.INCEPTION, name: 'Inception AI', icon: '🧠' },
    { id: PROVIDERS.LOCAL, name: 'Local Infrastructure', icon: '🏠' },
];

export const use_provider_store = create<Provider_State>()(
    persist(
        (set, get) => ({
            providers: DEFAULT_PROVIDERS,
            base_urls: {},
            deleting_ids: new Set(),

            sync_defaults: () => {
                const providers = get().providers || [];
                const model_store = use_model_store.getState();
                
                if (providers.length === 0 && (model_store.models || []).length === 0) {
                    set({ providers: DEFAULT_PROVIDERS });
                }
                
                model_store.sync_defaults(providers.length);
            }
        }),
        {
            name: 'tadpole-infrastructure-v3',
            partialize: (state) => ({
                providers: state.providers,
                base_urls: state.base_urls,
            }),
            migrate: (persisted_state: unknown) => {
                const state = (persisted_state || {}) as { providers?: Record<string, unknown>[]; base_urls?: Record<string, string> };
                const providers = state.providers || [];
                
                return {
                    ...state,
                    providers: (providers || []).map((p: Record<string, unknown>) => ({
                        ...p,
                        api_key: p.api_key ?? p.apiKey,
                        base_url: p.base_url ?? p.baseUrl,
                        external_id: p.external_id ?? p.externalId,
                        custom_headers: p.custom_headers ?? p.customHeaders,
                        audio_model: p.audio_model ?? p.audioModel,
                        persist_to_engine: p.persist_to_engine ?? p.persistToEngine,
                        supports_steering_vectors: p.supports_steering_vectors ?? p.supportsSteeringVectors
                    })),
                    base_urls: state.base_urls ?? (state as Record<string, unknown>).baseUrls ?? {}
                } as unknown as Record<string, unknown>;
            }
        }
    )
);
