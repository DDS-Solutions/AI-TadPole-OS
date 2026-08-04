/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[infra_api]` in observability traces.
 */

import { api_request } from '../base_api_service';
import type { Provider_Test_Config, Store_Model, Swarm_Node, RequestOptions } from '../system_api_types';

const sanitize_id = (id: string): string => {
    // Colon is included to support Ollama model IDs (e.g. gemma4:e4b, llama3:8b).
    // encodeURIComponent percent-encodes the colon before it is placed in the URL,
    // so this does not expand the injection surface.
    if (!id || id.includes('..') || !/^[a-zA-Z0-9_.:/-]{1,128}$/.test(id)) {
        throw new Error(`Invalid identifier: ${id}`);
    }
    return encodeURIComponent(id);
};

export const infra_api = {
    test_provider: async (config: Provider_Test_Config, options?: RequestOptions): Promise<{ status: string; latency?: number; message?: string }> => {
        try {
            const clean_id = sanitize_id(config.id);
            return await api_request<{ status: string; latency?: number }>(`/v1/infra/providers/${clean_id}/test`, {
                method: 'POST',
                body: JSON.stringify(config),
                signal: options?.signal,
                timeout: options?.timeout
            });
        } catch (error) {
            const is_timeout = error instanceof Error && (error.message.includes('timed out') || error.message.includes('TIMEOUT'));
            const message = is_timeout
                ? 'Handshake timeout: The provider endpoint is unresponsive.'
                : (error instanceof Error ? error.message : 'Network connection refused.');
            return { status: 'error', message };
        }
    },

    get_nodes: async (options?: RequestOptions): Promise<Swarm_Node[]> => {
        return api_request<Swarm_Node[]>('/v1/infra/nodes', {
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    discover_nodes: async (options?: RequestOptions): Promise<{ status: string, discovered: string[] }> => {
        return api_request<{ status: string, discovered: string[] }>('/v1/infra/nodes/discover', { 
            method: 'POST',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    get_providers: async (options?: RequestOptions): Promise<Record<string, unknown>[]> => {
        return api_request<Record<string, unknown>[]>('/v1/infra/providers', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    update_provider: async (id: string, config: Record<string, unknown>, options?: RequestOptions): Promise<{ status: string }> => {
        const clean_id = sanitize_id(id);
        return api_request<{ status: string }>(`/v1/infra/providers/${clean_id}`, {
            method: 'PUT',
            body: JSON.stringify(config),
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    delete_provider: async (id: string, options?: RequestOptions): Promise<void> => {
        const clean_id = sanitize_id(id);
        await api_request<void>(`/v1/infra/providers/${clean_id}`, { 
            method: 'DELETE',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    sync_provider_models: async (id: string, api_key?: string, options?: RequestOptions): Promise<{ status: string; added: number; discovered: number; message: string }> => {
        const clean_id = sanitize_id(id);
        return api_request<{ status: string; added: number; discovered: number; message: string }>(`/v1/infra/providers/${clean_id}/sync`, {
            method: 'POST',
            body: api_key ? JSON.stringify({ api_key }) : undefined,
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    update_model: async (id: string, entry: Record<string, unknown>, options?: RequestOptions): Promise<{ status: string }> => {
        const clean_id = sanitize_id(id);
        return api_request<{ status: string }>(`/v1/infra/models/${clean_id}`, {
            method: 'PUT',
            body: JSON.stringify(entry),
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    delete_model: async (id: string, options?: RequestOptions): Promise<void> => {
        const clean_id = sanitize_id(id);
        await api_request<void>(`/v1/infra/models/${clean_id}`, { 
            method: 'DELETE',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    get_models: async (options?: RequestOptions): Promise<Record<string, unknown>[]> => {
        return api_request<Record<string, unknown>[]>('/v1/infra/models', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    get_model_catalog: async (options?: RequestOptions): Promise<Store_Model[]> => {
        return api_request<Store_Model[]>('/v1/infra/model-store/catalog', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    pull_model: async (model_id: string, node_id: string, options?: RequestOptions): Promise<{ status: string }> => {
        return api_request<{ status: string }>('/v1/infra/model-store/pull', {
            method: 'POST',
            body: JSON.stringify({ tag: model_id, node_id }),
            signal: options?.signal,
            timeout: options?.timeout
        });
    }
};

// Metadata: [infra_api]
