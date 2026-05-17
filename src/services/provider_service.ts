/**
 * @docs ARCHITECTURE:Services:Provider
 * 
 * ### AI Assist Note
 * **Neural Orchestrator**: Hardens the provider configuration lifecycle by extracting side-effects.
 * Manages cross-tab synchronization and backend parity for AI infrastructure nodes.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Encryption mismatch in vault, or provider sync timeout.
 * - **Telemetry Link**: Search `[ProviderService]` in UI traces.
 */

import { use_provider_store, type Provider_Config } from '../stores/provider_store';
import { use_vault_store } from '../stores/vault_store';
import { use_model_store } from '../stores/model_store';
import { tadpole_os_service } from './tadpoleos_service';
import { log_error } from './system_utils';

const SYNC_CHANNEL = 'tadpole-os-sync';
const sync_channel = typeof window !== 'undefined' ? new BroadcastChannel(SYNC_CHANNEL) : null;

class Provider_Service {
    private last_broadcast: string = '';
    private debounce_timer: ReturnType<typeof setTimeout> | null = null;

    /**
     * Initializes the service, setting up cross-tab synchronization.
     */
    public init(): () => void {
        if (!sync_channel) return () => {};

        const on_message = (event: MessageEvent) => {
            const { type, payload } = event.data;
            if (type === 'providers:sync') {
                this.last_broadcast = JSON.stringify({
                    providers: payload.providers,
                    base_urls: payload.base_urls
                });
                use_provider_store.setState({ 
                    providers: payload.providers,
                    base_urls: payload.base_urls
                });
            }
        };

        sync_channel.addEventListener('message', on_message);

        const unsubscribe = use_provider_store.subscribe((state) => {
            const current = JSON.stringify({ 
                providers: state.providers, 
                base_urls: state.base_urls 
            });
            
            if (current !== this.last_broadcast) {
                this.last_broadcast = current;
                
                if (this.debounce_timer) clearTimeout(this.debounce_timer);
                this.debounce_timer = setTimeout(() => {
                    sync_channel?.postMessage({ 
                        type: 'providers:sync', 
                        payload: {
                            providers: state.providers,
                            base_urls: state.base_urls
                        }
                    });
                }, 250);
            }
        });

        return () => {
            sync_channel.removeEventListener('message', on_message);
            unsubscribe();
        };
    }

    /**
     * Synchronizes provider registry with the backend.
     */
    public async sync_with_backend(): Promise<void> {
        try {
            const raw_providers = (await tadpole_os_service.get_providers()) || [];
            const backend_providers: Provider_Config[] = (raw_providers as Record<string, unknown>[]).map(bp => ({
                ...bp,
                id: bp.id as string,
                name: (bp.name || bp.id) as string,
                api_key: bp.api_key as string | undefined,
                base_url: (bp.base_url || bp.baseUrl) as string | undefined,
                external_id: (bp.external_id || bp.externalId) as string | undefined,
                custom_headers: (bp.custom_headers || bp.customHeaders) as Record<string, string> | undefined,
                audio_model: (bp.audio_model || bp.audioModel) as string | undefined,
                persist_to_engine: (bp.persist_to_engine ?? bp.persistToEngine) as boolean | undefined,
                supports_steering_vectors: (bp.supports_steering_vectors ?? bp.supportsSteeringVectors) as boolean | undefined
            } as Provider_Config));

            const store = use_provider_store.getState();
            const model_store = use_model_store.getState();

            // Sync Logic
            const filtered_providers = store.providers.filter(p => 
                backend_providers.some(bp => bp.id === p.id) || store.deleting_ids.has(p.id)
            );

            const final_providers = [...filtered_providers];
            let providers_changed = filtered_providers.length !== store.providers.length;
            const new_base_urls = { ...store.base_urls };
            let urls_changed = false;

            backend_providers.forEach((bp) => {
                const existing = final_providers.find(p => p.id === bp.id);
                if (!existing && !store.deleting_ids.has(bp.id)) {
                    final_providers.push(bp);
                    providers_changed = true;
                } else if (existing) {
                    if (bp.base_url && existing.base_url !== bp.base_url) {
                        existing.base_url = bp.base_url;
                        existing.protocol = bp.protocol;
                        providers_changed = true;
                    }
                }
                
                if (bp.base_url && new_base_urls[bp.id] !== bp.base_url) {
                    new_base_urls[bp.id] = bp.base_url;
                    urls_changed = true;
                }
            });

            if (providers_changed || urls_changed) {
                use_provider_store.setState({ 
                    providers: final_providers,
                    base_urls: new_base_urls
                });
            }

            await model_store.sync_models();
            store.sync_defaults();

        } catch (error) {
            log_error('ProviderService', 'Backend Synchronization Failed', error);
        }
    }

    /**
     * Updates provider configuration and synchronizes with vault and engine.
     */
    public async set_provider_config(
        id: string, 
        api_key: string, 
        config: Partial<Provider_Config>
    ): Promise<void> {
        const vault_store = use_vault_store.getState();

        if (api_key) {
            await vault_store.set_encrypted_config(id, api_key);
        }

        use_provider_store.setState(state => ({
            base_urls: { ...state.base_urls, [id]: config.base_url || '' },
            providers: state.providers.map(p => p.id === id ? { ...p, ...config } : p)
        }));

        try {
            const key_to_persist = (config.persist_to_engine && api_key) ? api_key : null;
            const provider = use_provider_store.getState().providers.find(p => p.id === id);

            await tadpole_os_service.update_provider(id, {
                id,
                name: provider?.name || id,
                icon: provider?.icon,
                api_key: key_to_persist,
                base_url: config.base_url || null,
                protocol: config.protocol,
                external_id: config.external_id || null,
                custom_headers: config.custom_headers,
                audio_model: config.audio_model || null
            });
        } catch (error) {
            log_error('ProviderService', 'Config Update Failed', error);
        }
    }

    /**
     * Registers a new provider.
     */
    public async add_provider(name: string, icon: string): Promise<void> {
        const store = use_provider_store.getState();
        if (store.providers.length >= 25) {
            throw new Error('Capacity Limit: Maximum of 25 nodes reached.');
        }

        const base_id = name.toLowerCase().replace(/\s+/g, '-');
        let id = base_id;
        let counter = 1;

        while (store.providers.some(p => p.id === id)) {
            id = `${base_id}-${counter++}`;
        }

        use_provider_store.setState(state => ({
            providers: [...state.providers, { id, name, icon }]
        }));

        try {
            await tadpole_os_service.update_provider(id, {
                id, name, icon,
                api_key: null, base_url: null, protocol: 'openai', external_id: null
            });
        } catch (error) {
            log_error('ProviderService', 'Provider Creation Failed', error);
        }
    }

    /**
     * Deletes a provider and its associated models.
     */
    public async delete_provider(id: string): Promise<void> {
        const model_store = use_model_store.getState();
        const associated_model_ids = model_store.models.filter(m => m.provider === id).map(m => m.id);

        use_provider_store.setState(state => {
            const new_deleting = new Set(state.deleting_ids);
            new_deleting.add(id);
            associated_model_ids.forEach(mid => new_deleting.add(mid));

            return {
                providers: state.providers.filter(p => p.id !== id),
                deleting_ids: new_deleting
            };
        });

        associated_model_ids.forEach(mid => model_store.delete_model(mid));

        try {
            await tadpole_os_service.delete_provider(id);
            setTimeout(() => {
                use_provider_store.setState(state => {
                    const new_deleting = new Set(state.deleting_ids);
                    new_deleting.delete(id);
                    return { deleting_ids: new_deleting };
                });
            }, 10000);
        } catch (error) {
            log_error('ProviderService', 'Deletion Failed', error);
        }
    }
}

export const provider_service = new Provider_Service();
