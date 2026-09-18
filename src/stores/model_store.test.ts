/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / model_store.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { use_model_store } from './model_store';
import { system_api_service } from '../services/system_api_service';
import { log_error } from '../services/system_utils';

vi.mock('../services/system_api_service', () => ({
    system_api_service: {
        infra: {
            update_model: vi.fn(),
            get_models: vi.fn(),
            delete_model: vi.fn(),
        }
    }
}));

vi.mock('../services/system_utils', () => ({
    log_error: vi.fn(),
}));

describe('use_model_store', () => {
    beforeEach(() => {
        use_model_store.setState({
            models: [{ id: 'm1', name: 'gpt-4', provider: 'openai', modality: 'llm' }],
            deleting_ids: new Set(),
        });
        vi.clearAllMocks();
    });

    describe('Model CRUD', () => {
        it('adds a model', async () => {
            const store = use_model_store.getState();
            await store.add_model('llama-3', 'meta', 'llm');

            const state = use_model_store.getState();
            expect(state.models).toHaveLength(2);
            expect(state.models[1].name).toBe('llama-3');
            expect(state.models[1].provider).toBe('meta');
            
            expect(system_api_service.infra.update_model).toHaveBeenCalledWith(
                state.models[1].id,
                expect.objectContaining({ provider_id: 'meta' })
            );
        });

        it('logs error on add_model sync failure', async () => {
            vi.mocked(system_api_service.infra.update_model).mockRejectedValue(new Error('Sync failed'));
            
            const store = use_model_store.getState();
            await store.add_model('llama-3', 'meta', 'llm');
            
            expect(log_error).toHaveBeenCalledWith('ModelStore', 'Model Sync Failed', expect.any(Error));
        });

        it('edits a model', async () => {
            const store = use_model_store.getState();
            await store.edit_model('m1', 'gpt-4o', 'openai', 'vision');

            const state = use_model_store.getState();
            expect(state.models[0].name).toBe('gpt-4o');
            expect(state.models[0].modality).toBe('vision');
        });

        it('logs error on edit_model sync failure', async () => {
            vi.mocked(system_api_service.infra.update_model).mockRejectedValue(new Error('Update failed'));
            
            const store = use_model_store.getState();
            await store.edit_model('m1', 'gpt-4o', 'openai', 'vision');
            
            expect(log_error).toHaveBeenCalledWith('ModelStore', 'Model Update Sync Failed', expect.any(Error));
        });

        it('deletes a model', async () => {
            const store = use_model_store.getState();
            await store.delete_model('m1');

            const state = use_model_store.getState();
            expect(state.models).toHaveLength(0);
            expect(system_api_service.infra.delete_model).toHaveBeenCalledWith('m1');
        });

        it('logs error and rolls back state on delete_model sync failure', async () => {
            vi.mocked(system_api_service.infra.delete_model).mockRejectedValue(new Error('Delete failed'));
            
            const store = use_model_store.getState();
            await store.delete_model('m1');
            
            expect(log_error).toHaveBeenCalledWith('ModelStore', 'Model Deletion Sync Failed', expect.any(Error));
            expect(use_model_store.getState().models.some(m => m.id === 'm1')).toBe(true);
        });
    });

    describe('Synchronization', () => {
        it('sync_models updates state from backend', async () => {
            vi.mocked(system_api_service.infra.get_models).mockResolvedValue([
                { id: 'remote_m1', name: 'claude-3', provider: 'anthropic' }
            ] as any);

            const store = use_model_store.getState();
            await store.sync_models();

            const state = use_model_store.getState();
            expect(state.models.find(m => m.id === 'remote_m1')).toBeDefined();
        });

        it('sync_defaults maps known model names to their proper providers', () => {
            use_model_store.setState({
                models: [
                    { id: 'm1', name: 'claude-3.5-sonnet', provider: 'local', modality: 'llm' },
                    { id: 'm2', name: 'gemini-1.5-pro', provider: 'local', modality: 'llm' }
                ]
            });

            use_model_store.getState().sync_defaults(2);

            const state = use_model_store.getState();
            expect(state.models.find(m => m.id === 'm1')?.provider).toBe('anthropic');
            expect(state.models.find(m => m.id === 'm2')?.provider).toBe('google');
        });

        it('migrates persisted state with legacy camelCase token properties', () => {
            const migrate = (use_model_store as any).persist?.getOptions?.()?.migrate;
            if (typeof migrate === 'function') {
                const legacy_state = {
                    models: [
                        { id: 'm-old', name: 'old-model', providerId: 'openai', inputTokens: 4096, outputTokens: 2048 }
                    ]
                };
                const migrated = migrate(legacy_state, 0);
                expect(migrated.models[0].provider_id).toBe('openai');
                expect(migrated.models[0].input_tokens).toBe(4096);
                expect(migrated.models[0].output_tokens).toBe(2048);
            }
        });
    });
});
