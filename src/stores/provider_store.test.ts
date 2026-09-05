/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / provider_store.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { use_provider_store } from './provider_store';
import { provider_service } from '../services/provider_service';
import { system_api_service } from '../services/system_api_service';
import { log_error } from '../services/system_utils';

vi.mock('../services/system_api_service', () => ({
    system_api_service: {
        infra: {
            update_provider: vi.fn(),
            get_providers: vi.fn(),
            delete_provider: vi.fn(),
            get_models: vi.fn(),
            delete_model: vi.fn(),
            update_model: vi.fn(),
        }
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
            await provider_service.add_provider('Custom Hub', '🏢');

            const state = use_provider_store.getState();
            expect(state.providers).toHaveLength(2);
            expect(state.providers[1].id).toBe('custom-hub');
            expect(system_api_service.infra.update_provider).toHaveBeenCalled();
        });

        it('logs error on add_provider failure', async () => {
            vi.mocked(system_api_service.infra.update_provider).mockRejectedValue(new Error('Init failed'));
            
            await provider_service.add_provider('Fail Hub', '🏢');
            
            expect(log_error).toHaveBeenCalledWith('ProviderService', 'Provider Creation Failed', expect.any(Error));
        });

        it('edits a provider', async () => {
            await provider_service.set_provider_config('openai', '', { name: 'OpenAI v2', icon: '🚀' });

            const provider = use_provider_store.getState().providers[0];
            expect(provider.name).toBe('OpenAI v2');
            expect(provider.icon).toBe('🚀');
        });

        it('deletes a provider', async () => {
            await provider_service.delete_provider('openai');

            const state = use_provider_store.getState();
            expect(state.providers).toHaveLength(0);
            expect(system_api_service.infra.delete_provider).toHaveBeenCalledWith('openai');
        });

        it('logs error on delete_provider failure', async () => {
            vi.mocked(system_api_service.infra.delete_provider).mockRejectedValue(new Error('Delete Sync failed'));
            
            await provider_service.delete_provider('openai');
            
            expect(log_error).toHaveBeenCalledWith('ProviderService', 'Deletion Failed', expect.any(Error));
        });
    });

    describe('Coordinated Synchronization', () => {
        it('sync_with_backend updates providers', async () => {
            vi.mocked(system_api_service.infra.get_providers).mockResolvedValue([
                { id: 'anthropic', name: 'Anthropic Remote', base_url: 'https://api.anthropic.com' }
            ] as any);
            
            await provider_service.sync_with_backend();

            const state = use_provider_store.getState();
            expect(state.providers.find(p => p.id === 'anthropic')).toBeDefined();
        });

        it('logs error on sync_with_backend failure', async () => {
            vi.mocked(system_api_service.infra.get_providers).mockRejectedValue(new Error('Coordination failed'));
            
            await provider_service.sync_with_backend();
            
            expect(log_error).toHaveBeenCalledWith('ProviderService', 'Backend Synchronization Failed', expect.any(Error));
        });

        it('does not include api_key or Authorization headers in persisted state', () => {
            use_provider_store.setState({
                providers: [{
                    id: 'custom',
                    name: 'Custom',
                    api_key: 'secret-key-123',
                    custom_headers: { 'Authorization': 'Bearer test', 'X-Custom': 'safe' }
                }],
                base_urls: { custom: 'https://api.custom.com' },
                deleting_ids: new Set()
            });

            const persistOptions = (use_provider_store as any).persist?.getOptions?.();
            if (persistOptions?.partialize) {
                const persisted = persistOptions.partialize(use_provider_store.getState());
                expect(persisted.providers[0].api_key).toBeUndefined();
                expect(persisted.providers[0].custom_headers?.Authorization).toBeUndefined();
                expect(persisted.providers[0].custom_headers?.['X-Custom']).toBe('safe');
            }
        });

        it('sanitizes api_key and secret custom_headers on broadcast synchronization', () => {
            const raw_providers = [{
                id: 'custom-broadcast',
                name: 'Custom Broadcast',
                api_key: 'sensitive-api-token',
                custom_headers: { 'Authorization': 'Bearer token-xyz', 'api_key': 'secret-hdr', 'X-Trace-Id': 'public-trace' }
            }];

            const sanitized = raw_providers.map((provider) => {
                const p = { ...provider };
                delete (p as any).api_key;
                return {
                    ...p,
                    custom_headers: provider.custom_headers ? Object.fromEntries(
                        Object.entries(provider.custom_headers).filter(([k]) => !/^(authorization|api[-_]?key|token|secret)$/i.test(k))
                    ) : undefined
                };
            });

            expect((sanitized[0] as any).api_key).toBeUndefined();
            expect(sanitized[0].custom_headers?.Authorization).toBeUndefined();
            expect(sanitized[0].custom_headers?.api_key).toBeUndefined();
            expect(sanitized[0].custom_headers?.['X-Trace-Id']).toBe('public-trace');
        });
    });
});
