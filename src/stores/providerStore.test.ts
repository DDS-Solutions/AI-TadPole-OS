import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useProviderStore } from './providerStore';
import { TadpoleOSService } from '../services/tadpoleosService';

vi.mock('../services/tadpoleosService', () => ({
    TadpoleOSService: {
        updateProvider: vi.fn(),
        getProviders: vi.fn(),
        deleteProvider: vi.fn(),
        getModels: vi.fn(),
    }
}));

describe('useProviderStore', () => {
    beforeEach(() => {
        useProviderStore.setState({
            providers: [{ id: 'openai', name: 'OpenAI', icon: '⚡' }],
            baseUrls: {},
            deletingIds: new Set(),
        });
        vi.clearAllMocks();
    });

    describe('Provider CRUD', () => {
        it('adds a provider', async () => {
            const store = useProviderStore.getState();
            await store.addProvider('Custom Hub', '🏢');

            const state = useProviderStore.getState();
            expect(state.providers).toHaveLength(2);
            expect(state.providers[1].id).toBe('custom-hub');
        });

        it('edits a provider', () => {
            const store = useProviderStore.getState();
            store.editProvider('openai', 'OpenAI v2', '🚀');

            const provider = useProviderStore.getState().providers[0];
            expect(provider.name).toBe('OpenAI v2');
        });

        it('deletes a provider', async () => {
            const store = useProviderStore.getState();
            await store.deleteProvider('openai');

            const state = useProviderStore.getState();
            expect(state.providers).toHaveLength(0);
            expect(TadpoleOSService.deleteProvider).toHaveBeenCalledWith('openai');
        });
    });

    describe('Coordinated Synchronization', () => {
        it('syncWithBackend updates providers', async () => {
            vi.mocked(TadpoleOSService.getProviders).mockResolvedValue([
                { id: 'anthropic', name: 'Anthropic Remote', baseUrl: 'https://api.anthropic.com' }
            ] as any);
            
            const store = useProviderStore.getState();
            await store.syncWithBackend();

            const state = useProviderStore.getState();
            expect(state.providers.find(p => p.id === 'anthropic')).toBeDefined();
        });
    });
});
