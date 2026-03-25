import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { CryptoService } from '../services/cryptoService';
import { MODEL_OPTIONS } from '../data/models';
import { PROVIDERS } from '../constants';
import { TadpoleOSService } from '../services/tadpoleosService';

export interface ModelEntry {
    id: string;
    name: string;
    provider: string; // Master provider ID used in frontend
    providerId?: string; // Mapped from backend JSON if exists
    modality: 'llm' | 'vision' | 'voice' | 'reasoning' | string;
    rpm?: number;
    rpd?: number;
    tpm?: number;
    tpd?: number;
}

interface ModelState {
    models: ModelEntry[];
    deletingIds: Set<string>;

    // Actions
    addModel: (name: string, provider: string, modality?: ModelEntry['modality'], limits?: Partial<Pick<ModelEntry, 'rpm' | 'rpd' | 'tpm' | 'tpd'>>) => Promise<void>;
    editModel: (id: string, name: string, provider: string, modality: ModelEntry['modality'], limits?: Partial<Pick<ModelEntry, 'rpm' | 'rpd' | 'tpm' | 'tpd'>>) => Promise<void>;
    deleteModel: (id: string) => Promise<void>;
    syncModels: () => Promise<void>;
    syncDefaults: (providersLength: number) => void;
}

// Initial models from static list
const INITIAL_MODELS: ModelEntry[] = MODEL_OPTIONS.map(m => {
    let provider: string = PROVIDERS.LOCAL;
    let modality: ModelEntry['modality'] = 'llm';
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
    if (lower.includes('coder') || lower.includes('reasoning') || lower.includes('o1') || lower.includes('o3') || lower.includes('o4') || lower.includes('r1') || lower.includes('thought')) modality = 'reasoning';

    return { id: CryptoService.generateId(), name: m, provider, modality };
});

export const useModelStore = create<ModelState>()(
    persist(
        (set, get) => ({
            models: INITIAL_MODELS,
            deletingIds: new Set(),

            addModel: async (name, provider, modality = 'llm', limits) => {
                const id = CryptoService.generateId();
                const newModel = {
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
                    models: [...state.models, newModel]
                }));

                try {
                    await TadpoleOSService.updateModel(id, {
                        ...newModel,
                        providerId: provider
                    });
                } catch (error) {
                    console.error('[ModelStore] Model sync failed:', error);
                }
            },

            editModel: async (id, name, provider, modality, limits) => {
                set(state => ({
                    models: state.models.map(m => m.id === id ? { ...m, name, provider, modality, ...limits } : m)
                }));

                try {
                    const model = get().models.find(m => m.id === id);
                    if (model) {
                        await TadpoleOSService.updateModel(id, {
                            ...model,
                            providerId: provider
                        });
                    }
                } catch (error) {
                    console.error('[ModelStore] Model update sync failed:', error);
                }
            },

            deleteModel: async (id) => {
                set(state => {
                    const newDeleting = new Set(state.deletingIds);
                    newDeleting.add(id);
                    return {
                        models: state.models.filter(m => m.id !== id),
                        deletingIds: newDeleting
                    };
                });

                try {
                    await TadpoleOSService.deleteModel(id);
                    setTimeout(() => {
                        set(state => {
                            const newDeleting = new Set(state.deletingIds);
                            newDeleting.delete(id);
                            return { deletingIds: newDeleting };
                        });
                    }, 5000);
                } catch (error) {
                    console.error('[ModelStore] Model deletion sync failed:', error);
                    set(state => {
                        const newDeleting = new Set(state.deletingIds);
                        newDeleting.delete(id);
                        return { deletingIds: newDeleting };
                    });
                }
            },

            syncModels: async () => {
                try {
                    const rawModels = await TadpoleOSService.getModels();
                    const backendModels: ModelEntry[] = (rawModels as Record<string, unknown>[]).map(bm => ({
                        ...bm,
                        id: String(bm.id),
                        name: String(bm.name || bm.id),
                        provider: String(bm.provider || bm.providerId || 'local'),
                        modality: String(bm.modality || 'llm')
                    } as ModelEntry));
                    
                    const { models, deletingIds } = get();
                    const filteredModels = models.filter(m => 
                        backendModels.some(bm => bm.id === m.id) || deletingIds.has(m.id)
                    );

                    const finalModels = [...filteredModels];
                    let changed = filteredModels.length !== models.length;

                    backendModels.forEach((bm) => {
                        const existing = finalModels.find(m => m.id === bm.id);
                        if (!existing && !deletingIds.has(bm.id)) {
                            finalModels.push(bm);
                            changed = true;
                        } else if (existing && (existing.provider !== bm.provider || existing.name !== bm.name)) {
                            // Update existing if provider or name changed
                            Object.assign(existing, bm);
                            changed = true;
                        }
                    });

                    if (changed) {
                        set({ models: finalModels });
                    }
                } catch (error) {
                    console.error('[ModelStore] Backend sync failed:', error);
                }
            },

            syncDefaults: (providersLength) => {
                const { models } = get();
                if (providersLength === 0 && models.length === 0) {
                    set({ models: INITIAL_MODELS });
                    return;
                }

                const updatedModels = models.map(m => {
                    const lower = m.name.toLowerCase();
                    let newProvider = m.provider;
                    if (lower.includes('claude')) newProvider = PROVIDERS.ANTHROPIC;
                    else if (lower.includes('gemini')) newProvider = PROVIDERS.GOOGLE;
                    else if (lower.includes('llama') && (lower.includes('groq') || lower.includes('versatile'))) newProvider = PROVIDERS.GROQ;

                    if (newProvider !== m.provider) {
                        return { ...m, provider: newProvider };
                    }
                    return m;
                });

                if (JSON.stringify(updatedModels) !== JSON.stringify(models)) {
                    set({ models: updatedModels });
                }
            }
        }),
        {
            name: 'tadpole-model-inventory',
            partialize: (state) => ({
                models: state.models,
            }),
        }
    )
);
