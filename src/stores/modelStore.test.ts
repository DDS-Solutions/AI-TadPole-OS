import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useModelStore } from './modelStore';
import { TadpoleOSService } from '../services/tadpoleosService';

vi.mock('../services/tadpoleosService', () => ({
    TadpoleOSService: {
        updateModel: vi.fn(),
        getModels: vi.fn(),
        deleteModel: vi.fn(),
    }
}));

describe('useModelStore', () => {
    beforeEach(() => {
        useModelStore.setState({
            models: [{ id: 'm1', name: 'gpt-4', provider: 'openai', modality: 'llm' }],
            deletingIds: new Set(),
        });
        vi.clearAllMocks();
    });

    describe('Model CRUD', () => {
        it('adds a model', async () => {
            const store = useModelStore.getState();
            await store.addModel('llama-3', 'meta', 'llm');

            const state = useModelStore.getState();
            expect(state.models).toHaveLength(2);
            expect(state.models[1].name).toBe('llama-3');
            expect(state.models[1].provider).toBe('meta');
            
            expect(TadpoleOSService.updateModel).toHaveBeenCalledWith(
                state.models[1].id,
                expect.objectContaining({ providerId: 'meta' })
            );
        });

        it('edits a model', async () => {
            const store = useModelStore.getState();
            await store.editModel('m1', 'gpt-4o', 'openai', 'vision');

            const state = useModelStore.getState();
            expect(state.models[0].name).toBe('gpt-4o');
            expect(state.models[0].modality).toBe('vision');
        });

        it('deletes a model', async () => {
            const store = useModelStore.getState();
            await store.deleteModel('m1');

            const state = useModelStore.getState();
            expect(state.models).toHaveLength(0);
            expect(TadpoleOSService.deleteModel).toHaveBeenCalledWith('m1');
        });
    });

    describe('Synchronization', () => {
        it('syncModels updates state from backend', async () => {
            vi.mocked(TadpoleOSService.getModels).mockResolvedValue([
                { id: 'remote_m1', name: 'claude-3', provider: 'anthropic' }
            ] as any);

            const store = useModelStore.getState();
            await store.syncModels();

            const state = useModelStore.getState();
            expect(state.models.find(m => m.id === 'remote_m1')).toBeDefined();
        });
    });
});
