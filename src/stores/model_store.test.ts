/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Test Suite**: Validates the AI model configurations and availability. 
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
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { use_model_store } from './model_store';
import { tadpole_os_service } from '../services/tadpoleos_service';

vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        update_model: vi.fn(),
        get_models: vi.fn(),
        delete_model: vi.fn(),
    }
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

        it('edits a model', async () => {
            const store = use_model_store.getState();
            await store.edit_model('m1', 'gpt-4o', 'openai', 'vision');

            const state = use_model_store.getState();
            expect(state.models[0].name).toBe('gpt-4o');
            expect(state.models[0].modality).toBe('vision');
        });

        it('deletes a model', async () => {
            const store = use_model_store.getState();
            await store.delete_model('m1');

            const state = use_model_store.getState();
            expect(state.models).toHaveLength(0);
            expect(tadpole_os_service.delete_model).toHaveBeenCalledWith('m1');
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

