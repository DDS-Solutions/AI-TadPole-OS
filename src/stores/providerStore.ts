import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { PROVIDERS } from '../constants';
import { TadpoleOSService } from '../services/tadpoleosService';
import { useVaultStore } from './vaultStore';
import { useModelStore, type ModelEntry } from './modelStore';

export type { ModelEntry };

export interface ProviderConfig {
    id: string; // 'openai', 'anthropic', etc.
    name: string;
    icon?: string;
    apiKey?: string; // Encrypted string in legacy, now handled by vaultStore
    baseUrl?: string;
    externalId?: string; // Provider identity for ToS/Tracking
    protocol?: 'openai' | 'anthropic' | 'google' | 'ollama' | 'deepseek';
    customHeaders?: Record<string, string>;
    audioModel?: string;
    persistToEngine?: boolean;
    metadata?: Record<string, unknown>;
}

interface ProviderState {
    providers: ProviderConfig[];
    baseUrls: Record<string, string>; // providerId -> plaintext url (cached for UI speed)
    deletingIds: Set<string>;

    // Actions
    addProvider: (name: string, icon: string) => void;
    editProvider: (id: string, name: string, icon: string) => void;
    deleteProvider: (id: string) => Promise<void>;
    setProviderConfig: (id: string, apiKey: string, baseUrl?: string, externalId?: string, protocol?: ProviderConfig['protocol'], customHeaders?: Record<string, string>, audioModel?: string, persistToEngine?: boolean, metadata?: Record<string, unknown>) => Promise<void>;
    syncWithBackend: () => Promise<void>;
    
    // Legacy Bridge / Coordination
    syncDefaults: () => void;
}

const DEFAULT_PROVIDERS: ProviderConfig[] = [
    { id: PROVIDERS.OPENAI, name: 'OpenAI', icon: '⚡' },
    { id: PROVIDERS.ANTHROPIC, name: 'Anthropic', icon: '🏺' },
    { id: PROVIDERS.GOOGLE, name: 'Google Vertex', icon: '☁️' },
    { id: PROVIDERS.GROQ, name: 'Groq', icon: '⚡' },
    { id: PROVIDERS.OLLAMA, name: 'Ollama', icon: '🦙' },
    { id: 'meta', name: 'Meta / Llama', icon: '🦙' },
    { id: PROVIDERS.INCEPTION, name: 'Inception AI', icon: '🧠' },
    { id: PROVIDERS.LOCAL, name: 'Local Infrastructure', icon: '🏠' },
];

export const useProviderStore = create<ProviderState>()(
    persist(
        (set, get) => ({
            providers: DEFAULT_PROVIDERS,
            baseUrls: {},
            deletingIds: new Set(),

            addProvider: async (name, icon) => {
                const { providers } = get();
                if (providers.length >= 25) {
                    throw new Error('Neural Infrastructure Capacity Reached: Maximum of 25 nodes allowed.');
                }
                const baseId = (name || '').toLowerCase().replace(/\s+/g, '-');
                let id = baseId;
                let counter = 1;

                while (providers.some(p => p.id === id)) {
                    id = `${baseId}-${counter++}`;
                }

                const newProvider: ProviderConfig = { id, name, icon };

                set(state => ({
                    providers: [...state.providers, newProvider]
                }));

                try {
                    await TadpoleOSService.updateProvider(id, {
                        ...newProvider,
                        apiKey: null,
                        baseUrl: null,
                        protocol: 'openai',
                        externalId: null
                    });
                } catch (error) {
                    console.error('[NeuralVault] Provider initialization sync failed:', error);
                }
            },

            editProvider: (id, name, icon) => set(state => ({
                providers: state.providers.map(p => p.id === id ? { ...p, name, icon } : p)
            })),

            deleteProvider: async (id) => {
                const modelStore = useModelStore.getState();
                const associatedModelIds = modelStore.models.filter(m => m.provider === id).map(m => m.id);

                set(state => {
                    const newDeleting = new Set(state.deletingIds);
                    newDeleting.add(id);
                    associatedModelIds.forEach(mid => newDeleting.add(mid));

                    return {
                        providers: state.providers.filter(p => p.id !== id),
                        deletingIds: newDeleting
                    };
                });
                
                // Also trigger deletion in model store
                associatedModelIds.forEach(mid => modelStore.deleteModel(mid));

                try {
                    await TadpoleOSService.deleteProvider(id);
                    setTimeout(() => {
                        set(state => {
                            const newDeleting = new Set(state.deletingIds);
                            newDeleting.delete(id);
                            return { deletingIds: newDeleting };
                        });
                    }, 10000);
                } catch (error) {
                    console.error('[ProviderStore] Provider deletion sync failed:', error);
                    set(state => {
                        const newDeleting = new Set(state.deletingIds);
                        newDeleting.delete(id);
                        return { deletingIds: newDeleting };
                    });
                }
            },

            setProviderConfig: async (id, apiKey, baseUrl, externalId, protocol, customHeaders, audioModel, persistToEngine, metadata) => {
                const vaultStore = useVaultStore.getState();
                const { providers, baseUrls } = get();

                if (apiKey) {
                    await vaultStore.setEncryptedConfig(id, apiKey);
                }

                set({
                    baseUrls: { ...baseUrls, [id]: baseUrl || '' },
                    providers: providers.map(p => p.id === id ? {
                        ...p,
                        baseUrl: baseUrl || '',
                        externalId,
                        protocol,
                        customHeaders,
                        audioModel,
                        persistToEngine,
                        metadata
                    } : p)
                });

                try {
                    // Privacy Guard: Only send apiKey to backend if user explicitly requested engine persistence.
                    // This allows autonomous scheduled jobs to function without the developer's local vault.
                    const keyToPersist = (persistToEngine && apiKey) ? apiKey : null;
                    
                    await TadpoleOSService.updateProvider(id, {
                        id,
                        name: providers.find(p => p.id === id)?.name || id,
                        icon: providers.find(p => p.id === id)?.icon,
                        apiKey: keyToPersist,
                        baseUrl: baseUrl || null,
                        protocol,
                        externalId: externalId || null,
                        customHeaders,
                        audioModel: audioModel || null
                    });
                } catch (error) {
                    console.error('[ProviderStore] Backend sync failed:', error);
                }
            },

            syncDefaults: () => {
                const { providers } = get();
                const modelStore = useModelStore.getState();
                
                if (providers.length === 0 && modelStore.models.length === 0) {
                    set({ providers: DEFAULT_PROVIDERS });
                }
                
                modelStore.syncDefaults(providers.length);
            },

            syncWithBackend: async () => {
                try {
                    console.debug('[ProviderStore] Initiating coordination sync...');
                    const backendProviders = await TadpoleOSService.getProviders() as unknown as ProviderConfig[];
                    const { providers, deletingIds } = get();
                    const modelStore = useModelStore.getState();

                    // Sync Providers
                    const filteredProviders = providers.filter(p => 
                        backendProviders.some(bp => bp.id === p.id) || deletingIds.has(p.id)
                    );

                    const finalProviders = [...filteredProviders];
                    let providersChanged = filteredProviders.length !== providers.length;

                    backendProviders.forEach((bp) => {
                        const existing = finalProviders.find(p => p.id === bp.id);
                        if (!existing && !deletingIds.has(bp.id)) {
                            finalProviders.push(bp);
                            providersChanged = true;
                        } else if (existing) {
                            if (bp.baseUrl && existing.baseUrl !== bp.baseUrl) {
                                existing.baseUrl = bp.baseUrl;
                                existing.protocol = bp.protocol;
                                providersChanged = true;
                            }
                        }
                    });

                    if (providersChanged) {
                        set({ providers: finalProviders });
                    }

                    // Coordinate model sync
                    await modelStore.syncModels();
                    get().syncDefaults();

                } catch (error) {
                    console.error('[ProviderStore] Coordination sync failed:', error);
                }
            }
        }),
        {
            name: 'tadpole-infrastructure-v3',
            partialize: (state) => ({
                providers: state.providers,
                baseUrls: state.baseUrls,
            }),
        }
    )
);

