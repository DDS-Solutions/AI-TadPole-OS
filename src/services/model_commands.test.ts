/**
 * @docs ARCHITECTURE:Stores
 * ### AI Assist Note
 * Verify command recovery with overlapping state changes through the public store.
 * ### 🔍 Debugging & Observability
 * Deferred API fixtures expose optimistic failure ordering without network traffic.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { use_model_store } from '../stores/model_store';
import { system_api_service } from './system_api_service';

vi.mock('./system_api_service', () => ({ system_api_service: { infra: {
    update_model: vi.fn(), delete_model: vi.fn(), get_models: vi.fn()
} } }));
vi.mock('./system_utils', () => ({ log_error: vi.fn() }));

describe('model command recovery', () => {
    beforeEach(() => {
        vi.resetAllMocks();
        use_model_store.setState({ models: [{ id: 'm', name: 'Before', provider: 'local', modality: 'llm', rpm: 1 }], deleting_ids: new Set() });
    });

    it('rolls back failed fields while keeping newer edits and unrelated models', async () => {
        let reject!: (error: Error) => void;
        vi.mocked(system_api_service.infra.update_model).mockImplementationOnce(() => new Promise((_, fail) => { reject = fail; }));
        const saving = use_model_store.getState().edit_model('m', 'Optimistic', 'openai', 'llm', { rpm: 5 });
        use_model_store.setState(state => ({ models: [
            ...state.models.map(model => ({ ...model, name: 'Newer' })),
            { id: 'other', name: 'Other', provider: 'local', modality: 'llm' }
        ] }));
        reject(new Error('offline'));
        await saving;
        expect(use_model_store.getState().models).toEqual([
            expect.objectContaining({ id: 'm', name: 'Newer', provider: 'local', rpm: 1 }),
            expect.objectContaining({ id: 'other' })
        ]);
    });

    it('restores only a failed deletion and preserves models received while awaiting it', async () => {
        let reject!: (error: Error) => void;
        vi.mocked(system_api_service.infra.delete_model).mockImplementationOnce(() => new Promise((_, fail) => { reject = fail; }));
        const deleting = use_model_store.getState().delete_model('m');
        use_model_store.setState({ models: [{ id: 'other', name: 'Other', provider: 'local', modality: 'llm' }] });
        reject(new Error('offline'));
        await deleting;
        expect(use_model_store.getState().models.map(model => model.id).sort()).toEqual(['m', 'other']);
        expect(use_model_store.getState().deleting_ids.size).toBe(0);
    });

    it('accepts authoritative pricing updates even when routing is unchanged', async () => {
        vi.mocked(system_api_service.infra.get_models).mockResolvedValue([
            { id: 'm', name: 'Before', provider_id: 'local', modality: 'llm', input_cost_per_m: 7 }
        ]);
        await use_model_store.getState().sync_models();
        expect(use_model_store.getState().models[0].input_cost_per_m).toBe(7);
    });
});
