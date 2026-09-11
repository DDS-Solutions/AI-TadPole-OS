/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / provider_service.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { provider_service } from './provider_service';
import { use_provider_store } from '../stores/provider_store';
import { use_vault_store } from '../stores/vault_store';
import { use_model_store } from '../stores/model_store';
import { system_api_service } from './system_api_service';

vi.mock('./system_api_service', () => ({
    system_api_service: {
        infra: {
            get_providers: vi.fn(),
            update_provider: vi.fn(),
            delete_provider: vi.fn()
        }
    }
}));

describe('provider_service', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        provider_service.destroy();
        use_provider_store.setState({
            providers: [{ id: 'p1', name: 'Provider 1', icon: 'default' }],
            base_urls: { p1: 'http://localhost:11434' },
            deleting_ids: new Set()
        });
        use_vault_store.setState({
            set_encrypted_config: vi.fn().mockResolvedValue(undefined)
        } as any);
        use_model_store.setState({
            models: [{ id: 'm1', name: 'Model 1', provider: 'p1' } as any],
            sync_models: vi.fn().mockResolvedValue(undefined),
            delete_model: vi.fn()
        } as any);
    });

    afterEach(() => {
        provider_service.destroy();
    });

    describe('sync_with_backend', () => {
        it('syncs providers from backend and updates store', async () => {
            const backend_providers = [
                { id: 'p1', name: 'Provider 1 Updated', base_url: 'http://localhost:8000', protocol: 'openai' },
                { id: 'p2', name: 'Provider 2', base_url: 'https://api.openai.com/v1', protocol: 'openai' }
            ];
            vi.mocked(system_api_service.infra.get_providers).mockResolvedValueOnce(backend_providers);

            await provider_service.sync_with_backend();

            const state = use_provider_store.getState();
            expect(state.providers).toHaveLength(2);
            expect(state.base_urls.p1).toBe('http://localhost:8000');
            expect(state.base_urls.p2).toBe('https://api.openai.com/v1');
            expect(use_model_store.getState().sync_models).toHaveBeenCalled();
        });

        it('handles backend sync failure gracefully', async () => {
            vi.mocked(system_api_service.infra.get_providers).mockRejectedValueOnce(new Error('Network error'));
            await provider_service.sync_with_backend();
            // Should not crash and state remains unchanged
            expect(use_provider_store.getState().providers).toHaveLength(1);
        });
    });

    describe('set_provider_config', () => {
        it('persists key to vault and updates backend', async () => {
            vi.mocked(system_api_service.infra.update_provider).mockResolvedValueOnce(undefined as any);

            await provider_service.set_provider_config('p1', 'secret-key', {
                base_url: 'http://new-url:8000',
                protocol: 'openai',
                persist_to_engine: true
            });

            expect(use_vault_store.getState().set_encrypted_config).toHaveBeenCalledWith('p1', 'secret-key');
            expect(system_api_service.infra.update_provider).toHaveBeenCalledWith('p1', expect.objectContaining({
                id: 'p1',
                api_key: 'secret-key',
                base_url: 'http://new-url:8000'
            }));
            expect(use_provider_store.getState().base_urls.p1).toBe('http://new-url:8000');
        });

        it('reverts optimistic state update when backend update fails', async () => {
            vi.mocked(system_api_service.infra.update_provider).mockRejectedValueOnce(new Error('Backend error'));

            await provider_service.set_provider_config('p1', '', {
                base_url: 'http://failed-url:8000'
            });

            // Rolled back to initial base_url
            expect(use_provider_store.getState().base_urls.p1).toBe('http://localhost:11434');
        });
    });

    describe('add_provider', () => {
        it('generates unique slug id and adds provider to store and backend', async () => {
            vi.mocked(system_api_service.infra.update_provider).mockResolvedValueOnce(undefined as any);

            await provider_service.add_provider('Custom Node', 'custom-icon');

            const state = use_provider_store.getState();
            const added = state.providers.find(p => p.id === 'custom-node');
            expect(added).toBeDefined();
            expect(added?.name).toBe('Custom Node');
            expect(system_api_service.infra.update_provider).toHaveBeenCalledWith('custom-node', expect.objectContaining({
                id: 'custom-node',
                name: 'Custom Node',
                icon: 'custom-icon'
            }));
        });

        it('reverts optimistic addition if backend fails', async () => {
            vi.mocked(system_api_service.infra.update_provider).mockRejectedValueOnce(new Error('Failure'));

            await provider_service.add_provider('Fail Node', 'icon');

            expect(use_provider_store.getState().providers.find(p => p.id === 'fail-node')).toBeUndefined();
        });

        it('throws when capacity limit of 25 nodes is reached', async () => {
            const full_providers = Array.from({ length: 25 }, (_, i) => ({ id: `p-${i}`, name: `P${i}`, icon: 'icon' }));
            use_provider_store.setState({ providers: full_providers });

            await expect(provider_service.add_provider('Node 26', 'icon')).rejects.toThrow(/Capacity Limit/);
        });
    });

    describe('delete_provider', () => {
        it('removes provider, marks deleting_ids, and deletes associated models', async () => {
            vi.mocked(system_api_service.infra.delete_provider).mockResolvedValueOnce(undefined as any);

            await provider_service.delete_provider('p1');

            const state = use_provider_store.getState();
            expect(state.providers.find(p => p.id === 'p1')).toBeUndefined();
            expect(state.deleting_ids.has('p1')).toBe(true);
            expect(state.deleting_ids.has('m1')).toBe(true);
            expect(use_model_store.getState().delete_model).toHaveBeenCalledWith('m1');
            expect(system_api_service.infra.delete_provider).toHaveBeenCalledWith('p1');
        });

        it('reverts optimistic state and clears deleting_ids on deletion failure', async () => {
            vi.mocked(system_api_service.infra.delete_provider).mockRejectedValueOnce(new Error('Delete error'));

            await provider_service.delete_provider('p1');

            const state = use_provider_store.getState();
            expect(state.providers.find(p => p.id === 'p1')).toBeDefined();
            expect(state.deleting_ids.has('p1')).toBe(false);
            expect(state.deleting_ids.has('m1')).toBe(false);
        });
    });

    describe('init', () => {
        it('returns an unsubscribe cleanup function', () => {
            const cleanup = provider_service.init();
            expect(typeof cleanup).toBe('function');
            cleanup();
        });
    });
});
