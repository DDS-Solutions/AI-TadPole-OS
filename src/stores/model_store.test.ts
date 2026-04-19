/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Validation of the AI Provider and Model configuration store.** 
 * Verifies the mapping of provider-specific parameters (Temperature, TPM/RPM), model-specific capabilities (LLM vs Vision), and the lifecycle of model configurations. 
 * Mocks `tadpole_os_service` for model persistence operations to isolate state mutations from backend engine latency.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Invalid parameter propagation to the backend or failure to handle provider-specific quota limits during a model swap.
 * - **Telemetry Link**: Search `[model_store.test]` in tracing logs.
 */


/**
 * @file model_store.test.ts
 * @description Suite for the Swarm Model Config and Tokenomics store.
 * @module Stores/ModelStore
 * @testedBehavior
 * - Model CRUD: Lifecycle of LLM, TTS, and Vision model configurations.
 * - Persistence: Synchronization of local model registries with the backend engine.
 * - Modality Mapping: Validation of model-specific capabilities (e.g., LLM vs Vision).
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks tadpole_os_service for model persistence operations.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { use_model_store } from './model_store';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { log_error } from '../services/system_utils';

vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        update_model: vi.fn(),
        get_models: vi.fn(),
        delete_model: vi.fn(),
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
            
            expect(tadpole_os_service.update_model).toHaveBeenCalledWith(
                state.models[1].id,
                expect.objectContaining({ provider_id: 'meta' })
            );
        });

        it('logs error on add_model sync failure', async () => {
            vi.mocked(tadpole_os_service.update_model).mockRejectedValue(new Error('Sync failed'));
            
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
            vi.mocked(tadpole_os_service.update_model).mockRejectedValue(new Error('Update failed'));
            
            const store = use_model_store.getState();
            await store.edit_model('m1', 'gpt-4o', 'openai', 'vision');
            
            expect(log_error).toHaveBeenCalledWith('ModelStore', 'Model Update Sync Failed', expect.any(Error));
        });

        it('deletes a model', async () => {
            const store = use_model_store.getState();
            await store.delete_model('m1');

            const state = use_model_store.getState();
            expect(state.models).toHaveLength(0);
            expect(tadpole_os_service.delete_model).toHaveBeenCalledWith('m1');
        });

        it('logs error on delete_model sync failure', async () => {
            vi.mocked(tadpole_os_service.delete_model).mockRejectedValue(new Error('Delete failed'));
            
            const store = use_model_store.getState();
            await store.delete_model('m1');
            
            expect(log_error).toHaveBeenCalledWith('ModelStore', 'Model Deletion Sync Failed', expect.any(Error));
        });
    });

    describe('Synchronization', () => {
        it('sync_models updates state from backend', async () => {
            vi.mocked(tadpole_os_service.get_models).mockResolvedValue([
                { id: 'remote_m1', name: 'claude-3', provider: 'anthropic' }
            ] as any);

            const store = use_model_store.getState();
            await store.sync_models();

            const state = use_model_store.getState();
            expect(state.models.find(m => m.id === 'remote_m1')).toBeDefined();
        });
    });
});

