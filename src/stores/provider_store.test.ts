import { describe, it, expect, vi, beforeEach } from 'vitest';
import { use_provider_store } from './provider_store';
import { tadpole_os_service } from '../services/tadpoleos_service';

vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        update_provider: vi.fn(),
        get_providers: vi.fn(),
        delete_provider: vi.fn(),
        get_models: vi.fn(),
    }
}));

describe('use_provider_store', () => {
    beforeEach(() => {
        use_provider_store.setState({
            providers: [{ id: 'openai', name: 'OpenAI', icon: '⚡' }],
            baseUrls: {},
            deletingIds: new Set(),
        });
        vi.clearAllMocks();
    });

    describe('Provider CRUD', () => {
        it('adds a provider', async () => {
            const store = use_provider_store.getState();
            await store.addProvider('Custom Hub', '🏢');

            const state = use_provider_store.getState();
            expect(state.providers).toHaveLength(2);
            expect(state.providers[1].id).toBe('custom-hub');
        });

        it('edits a provider', () => {
            const store = use_provider_store.getState();
            store.editProvider('openai', 'OpenAI v2', '🚀');

            const provider = use_provider_store.getState().providers[0];
            expect(provider.name).toBe('OpenAI v2');
        });

        it('deletes a provider', async () => {
            const store = use_provider_store.getState();
            await store.delete_provider('openai');

            const state = use_provider_store.getState();
            expect(state.providers).toHaveLength(0);
            expect(tadpole_os_service.delete_provider).toHaveBeenCalledWith('openai');
        });
    });

    describe('Coordinated Synchronization', () => {
        it('syncWithBackend updates providers', async () => {
            vi.mocked(tadpole_os_service.get_providers).mockResolvedValue([
                { id: 'anthropic', name: 'Anthropic Remote', baseUrl: 'https://api.anthropic.com' }
            ] as any);
            
            const store = use_provider_store.getState();
            await store.syncWithBackend();

            const state = use_provider_store.getState();
            expect(state.providers.find(p => p.id === 'anthropic')).toBeDefined();
        });
    });
});

