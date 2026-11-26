/**
 * @docs ARCHITECTURE:Stores
 * ### AI Assist Note
 * Model commands own inventory requests and selective optimistic recovery.
 * ### 🔍 Debugging & Observability
 * Existing ModelStore failure telemetry remains the compatibility error channel.
 */
import type { StoreApi } from 'zustand';
import type { Model_State, Model_Entry } from '../stores/model_store';
import { Crypto_Service } from './crypto_service';
import { system_api_service } from './system_api_service';
import { log_error } from './system_utils';

type ModelCommands = Pick<Model_State, 'add_model' | 'edit_model' | 'delete_model' | 'sync_models'>;
export function create_model_commands({ set, get }: {
    set: StoreApi<Model_State>['setState']; get: () => Model_State;
}): ModelCommands {
    return {
        add_model: async (name, provider, modality = 'llm', limits) => {
            const id = Crypto_Service.generate_id();
            const new_model: Model_Entry = {
                id,
                name,
                provider,
                modality,
                rpm: limits?.rpm ?? 10,
                tpm: limits?.tpm ?? 100000,
                rpd: limits?.rpd ?? 1000,
                tpd: limits?.tpd ?? 10000000,
                context_window: limits?.context_window ?? 128000,
                input_cost_per_m: limits?.input_cost_per_m ?? 0,
                output_cost_per_m: limits?.output_cost_per_m ?? 0
            };

            set(state => ({
                models: [...state.models, new_model]
            }));

            try {
                await system_api_service.infra.update_model(id, {
                    ...new_model,
                    provider_id: provider
                });
            } catch (error) {
                log_error('ModelStore', 'Model Sync Failed', error);
                set(state => ({ models: state.models.filter(model => model !== new_model) }));
            }
        },

        edit_model: async (id, name, provider, modality, limits) => {
            const before = get().models.find(model => model.id === id);
            const patch = { name, provider, modality, ...limits };
            let optimistic: Model_Entry | undefined;
            set(state => ({
                models: (state.models || []).map(m => m.id === id ? (optimistic = { ...m, ...patch }) : m)
            }));

            try {
                const model = (get().models || []).find(m => m.id === id);
                if (model) {
                    await system_api_service.infra.update_model(id, {
                        ...model,
                        provider_id: provider
                    });
                }
            } catch (error) {
                log_error('ModelStore', 'Model Update Sync Failed', error);
                if (before) set(state => ({ models: state.models.map(model => {
                    if (model.id !== id) return model;
                    const restored = { ...model };
                    for (const field of Object.keys(patch) as (keyof typeof patch)[]) {
                        if (Object.is(model[field], optimistic?.[field])) Object.assign(restored, { [field]: before[field] });
                    }
                    return restored;
                }) }));
            }
        },

        delete_model: async (id) => {
            const prev_models = get().models || [];
            set(state => {
                const new_deleting = new Set(state.deleting_ids || []);
                new_deleting.add(id);
                return {
                    models: (state.models || []).filter(m => m.id !== id),
                    deleting_ids: new_deleting
                };
            });

            try {
                await system_api_service.infra.delete_model(id);
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
                    return {
                        models: state.models.some(model => model.id === id) ? state.models : [...state.models, ...prev_models.filter(model => model.id === id)],
                        deleting_ids: new_deleting
                    };
                });
            }
        },

        sync_models: async () => {
            try {
                const raw_models = (await system_api_service.infra.get_models()) || [];
                const backend_models: Model_Entry[] = (raw_models as Record<string, unknown>[]).map(bm => ({
                    ...bm,
                    id: bm.id as string,
                    name: (bm.name || bm.id) as string,
                    provider: (bm.provider || bm.provider_id || bm.providerId || 'local') as string,
                    provider_id: (bm.provider_id || bm.providerId || bm.provider || 'local') as string,
                    modality: (bm.modality || 'llm') as Model_Entry['modality'],
                    context_window: (bm.context_window as number) ?? (bm.capabilities as Model_Entry['capabilities'])?.context_window,
                    input_cost_per_m: bm.input_cost_per_m as number | undefined,
                    output_cost_per_m: bm.output_cost_per_m as number | undefined,
                    input_tokens: bm.input_tokens as number | undefined,
                    output_tokens: bm.output_tokens as number | undefined,
                    capabilities: bm.capabilities as Model_Entry['capabilities']
                } as Model_Entry));
            
                const models = get().models || [];
                const deleting_ids = get().deleting_ids || new Set();
                const filtered_models = models.filter(m => 
                    (backend_models || []).some(bm => bm.id === m.id) || deleting_ids.has(m.id)
                );

                let final_models = [...filtered_models];
                let changed = filtered_models.length !== models.length;

                backend_models.forEach((bm) => {
                    const existing_idx = final_models.findIndex(m => m.id === bm.id);
                    if (existing_idx === -1 && !deleting_ids.has(bm.id)) {
                        final_models.push(bm);
                        changed = true;
                    } else if (existing_idx !== -1) {
                        const existing = final_models[existing_idx];
                        // Authoritative Update: Backend always wins for routing critical fields
                        if (JSON.stringify(existing) !== JSON.stringify({ ...existing, ...bm })) {
                            final_models = final_models.map((m, idx) => idx === existing_idx ? { ...m, ...bm } : m);
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

    };
}
