/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Zustand State**: Intelligence Provider registry and connectivity manager. 
 * Orchestrates the management of external APIs (Anthropic, OpenAI, Groq, etc.) and local LLM backends via the `vault_store`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Provider authentication failure (check base64 vault tokens), or provider discovery timeout during backend sync.
 * - **Telemetry Link**: Search for `[ProviderStore]` or `PROVIDERS_SYNC` in UI logs.
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { PROVIDERS } from '../constants';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { use_vault_store } from './vault_store';
import { use_model_store, type Model_Entry } from './model_store';

const SYNC_CHANNEL = 'tadpole-os-sync';
const sync_channel = typeof window !== 'undefined' ? new BroadcastChannel(SYNC_CHANNEL) : null;

export type { Model_Entry };

export interface Provider_Config {
    id: string; // 'openai', 'anthropic', etc.
    name: string;
    icon?: string;
    api_key?: string; // Encrypted string in legacy, now handled by vaultStore
    base_url?: string;
    external_id?: string; // Provider identity for ToS/Tracking
    protocol?: 'openai' | 'anthropic' | 'google' | 'ollama' | 'deepseek';
    custom_headers?: Record<string, string>;
    audio_model?: string;
    persist_to_engine?: boolean;
    metadata?: Record<string, unknown>;
}

export interface Provider_State {
    providers: Provider_Config[];
    base_urls: Record<string, string>; // provider_id -> plaintext url (cached for UI speed)
    deleting_ids: Set<string>;

    // Actions
    add_provider: (name: string, icon: string) => Promise<void>;
    edit_provider: (id: string, name: string, icon: string) => void;
    delete_provider: (id: string) => Promise<void>;
    set_provider_config: (id: string, api_key: string, base_url?: string, external_id?: string, protocol?: Provider_Config['protocol'], custom_headers?: Record<string, string>, audio_model?: string, persist_to_engine?: boolean, metadata?: Record<string, unknown>) => Promise<void>;
    sync_with_backend: () => Promise<void>;
    
    // Legacy Bridge / Coordination
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

/**
 * use_provider_store
 * Configration management for AI providers.
 * Handles credential encryption via use_vault_store and sync with the Rust engine.
 * Refactored for strict snake_case compliance for backend parity.
 */
export const use_provider_store = create<Provider_State>()(
    persist(
        (set, get) => ({
            providers: DEFAULT_PROVIDERS,
            base_urls: {},
            deleting_ids: new Set(),

            add_provider: async (name, icon) => {
                const { providers } = get();
                if (providers.length >= 25) {
                    throw new Error('Neural Infrastructure Capacity Reached: Maximum of 25 nodes allowed.');
                }
                const base_id = (name || '').toLowerCase().replace(/\s+/g, '-');
                let id = base_id;
                let counter = 1;

                while (providers.some(p => p.id === id)) {
                    id = `${base_id}-${counter++}`;
                }

                const new_provider: Provider_Config = { id, name, icon };

                set(state => ({
                    providers: [...state.providers, new_provider]
                }));

                try {
                    await tadpole_os_service.update_provider(id, {
                        id,
                        name,
                        icon,
                        api_key: null,
                        base_url: null,
                        protocol: 'openai',
                        external_id: null
                    });
                } catch (error) {
                    console.error('[NeuralVault] Provider initialization sync failed:', error);
                }
            },

            edit_provider: (id, name, icon) => set(state => ({
                providers: state.providers.map(p => p.id === id ? { ...p, name, icon } : p)
            })),

            delete_provider: async (id) => {
                const model_store = use_model_store.getState();
                const associated_model_ids = model_store.models.filter(m => m.provider === id).map(m => m.id);

                set(state => {
                    const new_deleting = new Set(state.deleting_ids);
                    new_deleting.add(id);
                    associated_model_ids.forEach(mid => new_deleting.add(mid));

                    return {
                        providers: state.providers.filter(p => p.id !== id),
                        deleting_ids: new_deleting
                    };
                });
                
                // Also trigger deletion in model store
                associated_model_ids.forEach(mid => model_store.delete_model(mid));

                try {
                    await tadpole_os_service.delete_provider(id);
                    setTimeout(() => {
                        set(state => {
                            const new_deleting = new Set(state.deleting_ids);
                            new_deleting.delete(id);
                            return { deleting_ids: new_deleting };
                        });
                    }, 10000);
                } catch (error) {
                    console.error('[ProviderStore] Provider deletion sync failed:', error);
                    set(state => {
                        const new_deleting = new Set(state.deleting_ids);
                        new_deleting.delete(id);
                        return { deleting_ids: new_deleting };
                    });
                }
            },

            set_provider_config: async (id, api_key, base_url, external_id, protocol, custom_headers, audio_model, persist_to_engine, metadata) => {
                const vault_store = use_vault_store.getState();
                const { providers, base_urls } = get();

                if (api_key) {
                    await vault_store.set_encrypted_config(id, api_key);
                }

                set({
                    base_urls: { ...base_urls, [id]: base_url || '' },
                    providers: providers.map(p => p.id === id ? {
                        ...p,
                        base_url: base_url || '',
                        external_id,
                        protocol,
                        custom_headers,
                        audio_model,
                        persist_to_engine,
                        metadata
                    } : p)
                });

                try {
                    // Privacy Guard: Only send api_key to backend if user explicitly requested engine persistence.
                    // This allows autonomous scheduled jobs to function without the developer's local vault.
                    const key_to_persist = (persist_to_engine && api_key) ? api_key : null;
                    
                    await tadpole_os_service.update_provider(id, {
                        id,
                        name: providers.find(p => p.id === id)?.name || id,
                        icon: providers.find(p => p.id === id)?.icon,
                        api_key: key_to_persist,
                        base_url: base_url || null,
                        protocol,
                        external_id: external_id || null,
                        custom_headers,
                        audio_model: audio_model || null
                    });
                } catch (error) {
                    console.error('[ProviderStore] Backend sync failed:', error);
                }
            },

            sync_defaults: () => {
                const { providers } = get();
                const model_store = use_model_store.getState();
                
                if (providers.length === 0 && model_store.models.length === 0) {
                    set({ providers: DEFAULT_PROVIDERS });
                }
                
                model_store.sync_defaults(providers.length);
            },

            sync_with_backend: async () => {
                try {
                    console.debug('[ProviderStore] Initiating coordination sync...');
                    const raw_providers = await tadpole_os_service.get_providers();
                    const backend_providers: Provider_Config[] = (raw_providers as Record<string, unknown>[]).map(bp => ({
                        ...bp,
                        id: String(bp.id),
                        name: String(bp.name || bp.id),
                        api_key: bp.api_key as string | undefined, // Might be null from backend if not persisted
                        base_url: (bp.base_url || bp.baseUrl) as string | undefined,
                        external_id: (bp.external_id || bp.externalId) as string | undefined,
                        custom_headers: (bp.custom_headers || bp.customHeaders) as Record<string, string> | undefined,
                        audio_model: (bp.audio_model || bp.audioModel) as string | undefined,
                        persist_to_engine: (bp.persist_to_engine ?? bp.persistToEngine) as boolean | undefined
                    } as Provider_Config));

                    const { providers, deleting_ids } = get();
                    const model_store = use_model_store.getState();

                    // Sync Providers
                    const filtered_providers = providers.filter(p => 
                        backend_providers.some(bp => bp.id === p.id) || deleting_ids.has(p.id)
                    );

                    const final_providers = [...filtered_providers];
                    let providers_changed = filtered_providers.length !== providers.length;

                    backend_providers.forEach((bp) => {
                        const existing = final_providers.find(p => p.id === bp.id);
                        if (!existing && !deleting_ids.has(bp.id)) {
                            final_providers.push(bp);
                            providers_changed = true;
                        } else if (existing) {
                            if (bp.base_url && existing.base_url !== bp.base_url) {
                                existing.base_url = bp.base_url;
                                existing.protocol = bp.protocol;
                                providers_changed = true;
                            }
                        }
                    });

                    if (providers_changed) {
                        set({ providers: final_providers });
                    }

                    // Coordinate model sync
                    await model_store.sync_models();
                    get().sync_defaults();

                } catch (error) {
                    console.error('[ProviderStore] Coordination sync failed:', error);
                }
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
                    providers: providers.map((p: Record<string, unknown>) => ({
                        ...p,
                        api_key: p.api_key ?? p.apiKey,
                        base_url: p.base_url ?? p.baseUrl,
                        external_id: p.external_id ?? p.externalId,
                        custom_headers: p.custom_headers ?? p.customHeaders,
                        audio_model: p.audio_model ?? p.audioModel,
                        persist_to_engine: p.persist_to_engine ?? p.persistToEngine
                    })),
                    base_urls: state.base_urls ?? (state as Record<string, unknown>).baseUrls ?? {}
                } as unknown as Record<string, unknown>;
            }
        }
    )
);

// ── Cross-Tab Synchronization ────────────────────────────────
if (sync_channel) {
    // Initial fingerprint
    let last_broadcast = JSON.stringify({ 
        providers: use_provider_store.getState().providers, 
        base_urls: use_provider_store.getState().base_urls 
    });

    sync_channel.onmessage = (event) => {
        const { type, payload } = event.data;
        if (type === 'providers:sync') {
            // Update fingerprint BEFORE setting state to prevent local subscribe from echoing back
            last_broadcast = JSON.stringify({
                providers: payload.providers,
                base_urls: payload.base_urls
            });
            use_provider_store.setState({ 
                providers: payload.providers,
                base_urls: payload.base_urls
            });
        }
    };

    use_provider_store.subscribe((state) => {
        const current = JSON.stringify({ 
            providers: state.providers, 
            base_urls: state.base_urls 
        });
        
        if (current !== last_broadcast) {
            last_broadcast = current;
            sync_channel.postMessage({ 
                type: 'providers:sync', 
                payload: {
                    providers: state.providers,
                    base_urls: state.base_urls
                }
            });
        }
    });
}

