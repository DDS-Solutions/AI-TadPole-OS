/**
 * @docs ARCHITECTURE:Services:Provider
 * ### AI Assist Note
 * Blank credential input means keep; removal requires explicit operator intent.
 * ### 🔍 Debugging & Observability
 * Draft validation runs before provider, vault, or engine mutations.
 */
import type { Provider_Config } from '../../stores/provider_store';

export type Credential_Change = { kind: 'keep' } | { kind: 'set'; value: string } | { kind: 'remove' };
export type Provider_Draft = Omit<Provider_Config, 'id' | 'api_key' | 'protocol'> & {
    protocol: NonNullable<Provider_Config['protocol']>;
};
export type Raw_Provider_Draft = Omit<Partial<Provider_Config>, 'custom_headers' | 'protocol'> & {
    protocol?: string; custom_headers?: unknown;
};
const protocols: Provider_Draft['protocol'][] = [
    'openai', 'anthropic', 'google', 'ollama', 'deepseek', 'groq', 'local', 'xai',
    'together', 'mistral', 'openrouter', 'custom'
];

export function credential_change(value: string, remove = false): Credential_Change {
    if (remove) return { kind: 'remove' };
    return value.trim() ? { kind: 'set', value: value.trim() } : { kind: 'keep' };
}

export function validate_provider_draft(raw: Raw_Provider_Draft): Provider_Draft {
    const protocol = raw.protocol || 'openai';
    if (!protocols.includes(protocol as Provider_Draft['protocol'])) throw new Error('Unsupported communication protocol selected');
    let headers: unknown = raw.custom_headers || {};
    if (typeof headers === 'string') {
        try { headers = JSON.parse(headers); } catch { throw new Error('Invalid JSON in custom headers'); }
    }
    if (!headers || typeof headers !== 'object' || Array.isArray(headers)
        || Object.values(headers).some(value => typeof value !== 'string')) {
        throw new Error('Custom headers must be an object of string values');
    }
    const base_url = raw.base_url?.trim() || '';
    if (base_url) {
        const url = new URL(base_url);
        if (!['http:', 'https:'].includes(url.protocol)) throw new Error('Provider endpoint must use HTTP or HTTPS');
    }
    return {
        name: raw.name || '', icon: raw.icon, protocol: protocol as Provider_Draft['protocol'], base_url,
        external_id: raw.external_id?.trim(), custom_headers: headers as Record<string, string>,
        audio_model: raw.audio_model, persist_to_engine: raw.persist_to_engine ?? false,
        supports_steering_vectors: raw.supports_steering_vectors ?? false, metadata: raw.metadata
    };
}
