/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Test Suite**: Validates the AI provider configurations (OpenAI, etc.). 
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
    });
});

