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
import type { Provider_Test_Config, Store_Model, Swarm_Node } from '../system_api_types';

export const infra_api = {
    test_provider: async (config: Provider_Test_Config): Promise<{ status: string; latency?: number; message?: string }> => {
        try {
            return await api_request<{ status: string; latency?: number }>(`/v1/infra/providers/${config.id}/test`, {
                method: 'POST',
                body: JSON.stringify(config)
            });
        } catch (error) {
            const is_timeout = error === 'TIMEOUT';
            const message = is_timeout
                ? 'Handshake timeout: The provider endpoint is unresponsive.'
                : (error instanceof Error ? error.message : 'Network connection refused.');
            return { status: 'error', message };
        }
    },

    get_nodes: async (options: RequestInit = {}): Promise<Swarm_Node[]> => {
        return api_request<Swarm_Node[]>('/v1/infra/nodes', {
            method: 'GET',
            ...options
        });
    },

    discover_nodes: async (): Promise<{ status: string, discovered: string[] }> => {
        return api_request<{ status: string, discovered: string[] }>('/v1/infra/nodes/discover', { method: 'POST' });
    },

    get_providers: async (): Promise<Record<string, unknown>[]> => {
        return api_request<Record<string, unknown>[]>('/v1/infra/providers', { method: 'GET' });
    },

    update_provider: async (id: string, config: Record<string, unknown>): Promise<{ status: string }> => {
        return api_request<{ status: string }>(`/v1/infra/providers/${id}`, {
            method: 'PUT',
            body: JSON.stringify(config)
        });
    },

    delete_provider: async (id: string): Promise<void> => {
        await api_request(`/v1/infra/providers/${id}`, { method: 'DELETE' });
    },

    sync_provider_models: async (id: string, api_key?: string): Promise<{ status: string; added: number; discovered: number; message: string }> => {
        return api_request<{ status: string; added: number; discovered: number; message: string }>(`/v1/infra/providers/${id}/sync`, {
            method: 'POST',
            body: api_key ? JSON.stringify({ api_key }) : undefined
        });
    },

    update_model: async (id: string, entry: Record<string, unknown>): Promise<{ status: string }> => {
        return api_request<{ status: string }>(`/v1/infra/models/${id}`, {
            method: 'PUT',
            body: JSON.stringify(entry)
        });
    },

    delete_model: async (id: string): Promise<void> => {
        await api_request(`/v1/infra/models/${id}`, { method: 'DELETE' });
    },

    get_models: async (): Promise<Record<string, unknown>[]> => {
        return api_request<Record<string, unknown>[]>('/v1/infra/models', { method: 'GET' });
    },

    get_model_catalog: async (): Promise<Store_Model[]> => {
        return api_request<Store_Model[]>('/v1/infra/model-store/catalog', { method: 'GET' });
    },

    pull_model: async (model_id: string, node_id: string): Promise<{ status: string }> => {
        return api_request<{ status: string }>('/v1/infra/model-store/pull', {
            method: 'POST',
            body: JSON.stringify({ tag: model_id, node_id })
        });
    }
};

// Metadata: [infra_api]
