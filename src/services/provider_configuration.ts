/**
 * @docs ARCHITECTURE:Services:Provider
 * ### AI Assist Note
 * Provider commands own credentials, engine payloads, and entity-scoped recovery.
 * ### 🔍 Debugging & Observability
 * Failed writes reject to the caller; metadata rollback preserves unrelated edits.
 */
import { use_provider_store, type Provider_Config } from '../stores/provider_store';
import { use_vault_store } from '../stores/vault_store';
import { use_model_store } from '../stores/model_store';
import { system_api_service } from './system_api_service';
import { validate_provider_draft, type Credential_Change, type Raw_Provider_Draft, type Provider_Draft } from '../domain/providers/configuration';

const pending = new Map<string, symbol>();

async function effective_key(id: string, change: Credential_Change): Promise<string | null> {
    if (change.kind === 'remove') return null;
    if (change.kind === 'set') return change.value;
    return use_vault_store.getState().get_api_key(id);
}

function engine_payload(id: string, draft: Provider_Draft, key: string | null) {
    return { id, ...draft, api_key: key, base_url: draft.base_url || null, external_id: draft.external_id || null };
}

function validated(id: string, raw: Raw_Provider_Draft): Provider_Draft {
    const existing = use_provider_store.getState().providers.find(p => p.id === id);
    return validate_provider_draft({ ...existing, ...raw });
}

export const provider_configuration = {
    async save(id: string, raw: Raw_Provider_Draft, credential: Credential_Change): Promise<void> {
        const draft = validated(id, raw);
        if (pending.has(id)) throw new Error('A provider update is already in progress');
        const operation = Symbol(id);
        pending.set(id, operation);
        const before = use_provider_store.getState();
        const previous = before.providers.find(p => p.id === id);
        const previous_url = before.base_urls[id];
        let optimistic: Provider_Config | undefined;
        const vault_before = use_vault_store.getState().encrypted_configs;
        let written_configs: typeof vault_before | undefined;
        try {
            const vault = use_vault_store.getState();
            if (credential.kind !== 'keep' && !vault.is_unlocked()) throw new Error('Store is locked');
            const key = await effective_key(id, credential);
            if (!previous) throw new Error('Provider no longer exists');
            if (credential.kind !== 'keep') {
                await vault.set_encrypted_config(id, credential.kind === 'set' ? credential.value : '');
                written_configs = use_vault_store.getState().encrypted_configs;
            }
            optimistic = { ...previous, ...draft };
            use_provider_store.setState(state => ({
                providers: state.providers.map(p => p.id === id ? optimistic! : p),
                base_urls: { ...state.base_urls, [id]: draft.base_url || '' }
            }));
            // The engine preserves an existing key for null and removes it for ''.
            const engine_key = credential.kind === 'remove' ? '' : draft.persist_to_engine ? key : null;
            await system_api_service.infra.update_provider(id, engine_payload(id, draft, engine_key));
        } catch (error) {
            if (written_configs) use_vault_store.setState(state => {
                const encrypted_configs = { ...state.encrypted_configs };
                for (const key of new Set([id, id.toLowerCase()])) {
                    if (encrypted_configs[key] !== written_configs?.[key]) continue;
                    if (Object.hasOwn(vault_before, key)) encrypted_configs[key] = vault_before[key];
                    else delete encrypted_configs[key];
                }
                return { encrypted_configs };
            });
            if (optimistic && previous) use_provider_store.setState(state => ({
                providers: state.providers.map(p => {
                    if (p.id !== id) return p;
                    const restored = { ...p };
                    for (const field of Object.keys(draft) as (keyof Provider_Draft)[]) {
                        if (Object.is(p[field], optimistic?.[field])) Object.assign(restored, { [field]: previous[field] });
                    }
                    return restored;
                }),
                base_urls: state.base_urls[id] === (draft.base_url || '')
                    ? { ...state.base_urls, [id]: previous_url || '' } : state.base_urls
            }));
            throw error;
        } finally {
            if (pending.get(id) === operation) pending.delete(id);
        }
    },

    async test(id: string, raw: Raw_Provider_Draft, credential: Credential_Change) {
        const draft = validated(id, raw);
        if (credential.kind === 'remove') throw new Error('Save credential removal before testing the connection');
        const key = await effective_key(id, credential);
        // A blank key also allows the engine to use its saved credential.
        return system_api_service.infra.test_provider({ ...draft, id, api_key: key || '' });
    },

    async discover(id: string, raw: Raw_Provider_Draft, credential: Credential_Change) {
        const draft = validated(id, raw);
        const key = await effective_key(id, credential);
        if (credential.kind === 'remove') throw new Error('Save credential removal before discovering models');
        const stored = use_provider_store.getState().providers.find(p => p.id === id);
        if (draft.base_url !== (stored?.base_url || '') || draft.protocol !== (stored?.protocol || 'openai')
            || JSON.stringify(draft.custom_headers || {}) !== JSON.stringify(stored?.custom_headers || {})) {
            throw new Error('Save the provider connection settings before discovering models');
        }
        const result = await system_api_service.infra.sync_provider_models(id, key || undefined);
        await use_model_store.getState().sync_models();
        return result;
    }
};
