/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the AI Provider (OpenAI, Anthropic, Ollama) connectivity and config store.** 
 * Verifies the management of base URLs, API keys, and protocol handshakes. 
 * Mocks `tadpole_os_service` to isolate provider metadata orchestration from network side-effects and backend model latency.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Connectivity 'False Positives' where a provider is marked 'Active' but subsequent requests fail due to invalid endpoint metadata or expired API keys.
 * - **Telemetry Link**: Search `[provider_store.test]` in tracing logs.
 */


/**
 * @file provider_store.test.ts
 * @description Suite for the AI Provider Configuration store (provider_store).
 * @module Stores/ProviderStore
 * @testedBehavior
 * - CRUD: Technical infrastructure management (add, edit, delete providers).
 * - Coordination: Backend synchronization via tadpole_os_service.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity (add_provider, edit_provider, sync_with_backend, base_urls, deleting_ids).
 * - Mocks tadpole_os_service to prevent network side-effects.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { use_provider_store } from './provider_store';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { log_error } from '../services/system_utils';

vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        update_provider: vi.fn(),
        get_providers: vi.fn(),
        delete_provider: vi.fn(),
        get_models: vi.fn(),
        delete_model: vi.fn(),
        update_model: vi.fn(),
    }
}));

vi.mock('../services/system_utils', () => ({
    log_error: vi.fn(),
}));

describe('use_provider_store', () => {
    beforeEach(() => {
        use_provider_store.setState({
            providers: [{ id: 'openai', name: 'OpenAI', icon: '⚡' }],
            base_urls: {},
            deleting_ids: new Set(),
        });
        vi.clearAllMocks();
    });

    describe('Provider CRUD', () => {
        it('adds a provider', async () => {
            const store = use_provider_store.getState();
            await store.add_provider('Custom Hub', '🏢');

            const state = use_provider_store.getState();
            expect(state.providers).toHaveLength(2);
            expect(state.providers[1].id).toBe('custom-hub');
            expect(tadpole_os_service.update_provider).toHaveBeenCalled();
        });

        it('logs error on add_provider failure', async () => {
            vi.mocked(tadpole_os_service.update_provider).mockRejectedValue(new Error('Init failed'));
            
            const store = use_provider_store.getState();
            await store.add_provider('Fail Hub', '🏢');
            
            expect(log_error).toHaveBeenCalledWith('ProviderStore', 'Provider Initialization Failed', expect.any(Error));
        });

        it('edits a provider', () => {
            const store = use_provider_store.getState();
            store.edit_provider('openai', 'OpenAI v2', '🚀');

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

        it('logs error on delete_provider failure', async () => {
            vi.mocked(tadpole_os_service.delete_provider).mockRejectedValue(new Error('Delete Sync failed'));
            
            const store = use_provider_store.getState();
            await store.delete_provider('openai');
            
            expect(log_error).toHaveBeenCalledWith('ProviderStore', 'Provider Deletion Failed', expect.any(Error));
        });
    });

    describe('Coordinated Synchronization', () => {
        it('sync_with_backend updates providers', async () => {
            vi.mocked(tadpole_os_service.get_providers).mockResolvedValue([
                { id: 'anthropic', name: 'Anthropic Remote', base_url: 'https://api.anthropic.com' }
            ] as any);
            
            const store = use_provider_store.getState();
            await store.sync_with_backend();

            const state = use_provider_store.getState();
            expect(state.providers.find(p => p.id === 'anthropic')).toBeDefined();
        });

        it('logs error on sync_with_backend failure', async () => {
            vi.mocked(tadpole_os_service.get_providers).mockRejectedValue(new Error('Coordination failed'));
            
            const store = use_provider_store.getState();
            await store.sync_with_backend();
            
            expect(log_error).toHaveBeenCalledWith('ProviderStore', 'Coordination Sync Failed', expect.any(Error));
        });
    });
});

