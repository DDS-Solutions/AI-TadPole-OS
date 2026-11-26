/**
 * @docs ARCHITECTURE:Services:Provider
 * ### AI Assist Note
 * Exercise credential intent and recovery through the real command boundary.
 * ### 🔍 Debugging & Observability
 * Network and crypto ports are replaced with deterministic fixtures.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { provider_configuration } from './provider_configuration';
import { use_provider_store } from '../stores/provider_store';
import { use_vault_store } from '../stores/vault_store';
import { system_api_service } from './system_api_service';
import { credential_change } from '../domain/providers/configuration';

vi.mock('./system_api_service', () => ({ system_api_service: { infra: {
    update_provider: vi.fn(), test_provider: vi.fn(), sync_provider_models: vi.fn()
} } }));

describe('provider configuration commands', () => {
    beforeEach(() => {
        vi.resetAllMocks();
        use_provider_store.setState({ providers: [{ id: 'p', name: 'Before', protocol: 'openai' }], base_urls: {} });
        use_vault_store.setState({
            encrypted_configs: { p: 'old-ciphertext' },
            is_unlocked: () => true,
            get_api_key: vi.fn().mockResolvedValue('saved-secret'),
            set_encrypted_config: vi.fn(async (id, key) => {
                use_vault_store.setState(state => {
                    const encrypted_configs = { ...state.encrypted_configs };
                    if (key) encrypted_configs[id] = 'new-ciphertext';
                    else delete encrypted_configs[id];
                    return { encrypted_configs };
                });
            })
        });
    });

    it('uses the saved key for blank input in both Test and Save', async () => {
        const intent = credential_change('  ');
        await provider_configuration.test('p', { base_url: ' https://example.test/v1 ' }, intent);
        await provider_configuration.save('p', { persist_to_engine: true, base_url: 'https://example.test/v1' }, intent);
        expect(system_api_service.infra.test_provider).toHaveBeenCalledWith(expect.objectContaining({ api_key: 'saved-secret', base_url: 'https://example.test/v1' }));
        expect(system_api_service.infra.update_provider).toHaveBeenCalledWith('p', expect.objectContaining({ api_key: 'saved-secret' }));
        expect(use_vault_store.getState().set_encrypted_config).not.toHaveBeenCalled();
    });

    it('removes a key explicitly using the engine empty-string contract', async () => {
        await provider_configuration.save('p', { persist_to_engine: false }, { kind: 'remove' });
        expect(system_api_service.infra.update_provider).toHaveBeenCalledWith('p', expect.objectContaining({ api_key: '' }));
        expect(use_vault_store.getState().encrypted_configs.p).toBeUndefined();
    });

    it('validates before changing identity, credentials or engine state', async () => {
        await expect(provider_configuration.save('p', { name: 'After', custom_headers: '{bad' }, { kind: 'set', value: 'new' })).rejects.toThrow('Invalid JSON');
        expect(use_provider_store.getState().providers[0].name).toBe('Before');
        expect(use_vault_store.getState().set_encrypted_config).not.toHaveBeenCalled();
        expect(system_api_service.infra.update_provider).not.toHaveBeenCalled();
    });

    it('blocks key changes while locked before sending an engine update', async () => {
        use_vault_store.setState({ is_unlocked: () => false });
        await expect(provider_configuration.save('p', {}, { kind: 'remove' })).rejects.toThrow('locked');
        expect(system_api_service.infra.update_provider).not.toHaveBeenCalled();
    });

    it('recovers only its fields and ciphertext after a failed write', async () => {
        let reject!: (error: Error) => void;
        vi.mocked(system_api_service.infra.update_provider).mockImplementationOnce(() => new Promise((_, fail) => { reject = fail; }));
        const saving = provider_configuration.save('p', { name: 'Optimistic', base_url: 'https://example.test' }, { kind: 'set', value: 'new' });
        const rejected = expect(saving).rejects.toThrow('offline');
        await vi.waitFor(() => expect(reject).toBeTypeOf('function'));
        use_provider_store.setState(state => ({ providers: [...state.providers.map(p => ({ ...p, name: 'Newer edit' })), { id: 'other', name: 'Other' }] }));
        use_vault_store.setState(state => ({ encrypted_configs: { ...state.encrypted_configs, other: 'other-ciphertext' } }));
        reject(new Error('offline'));
        await rejected;
        expect(use_provider_store.getState().providers.map(p => p.name)).toEqual(['Newer edit', 'Other']);
        expect(use_provider_store.getState().base_urls.p).toBe('');
        expect(use_vault_store.getState().encrypted_configs).toEqual({ p: 'old-ciphertext', other: 'other-ciphertext' });
    });

    it('permits local providers without keys and preserves engine-only credentials on keep', async () => {
        use_vault_store.setState({ get_api_key: vi.fn().mockResolvedValue(null), is_unlocked: () => false });
        await provider_configuration.test('p', { protocol: 'ollama' }, { kind: 'keep' });
        await provider_configuration.save('p', { protocol: 'ollama', persist_to_engine: true }, { kind: 'keep' });
        expect(system_api_service.infra.update_provider).toHaveBeenCalledWith('p', expect.objectContaining({ api_key: null }));
    });

    it('rejects unsaved discovery settings instead of testing a different endpoint', async () => {
        await expect(provider_configuration.discover('p', { base_url: 'https://new.example.test' }, { kind: 'keep' })).rejects.toThrow('Save');
        expect(system_api_service.infra.sync_provider_models).not.toHaveBeenCalled();
    });
});
