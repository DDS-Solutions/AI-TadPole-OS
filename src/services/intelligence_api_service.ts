/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:Services**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[intelligence_api_service]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:Services
 * @docs API_REFERENCE:Endpoints
 * 
 * ### AI Assist Note
 * **Intelligence API Service**: Handles communication with the Rust intelligence backend endpoints.
 * Provides access to the codebase topology graph and blast radius analysis.
 */

import { api_request } from './base_api_service';
import type { SymbolNode } from '../contracts/generated';

export type { SymbolNode };

export interface SymbolEdge {
    source: string;
    target: string;
}

export interface CodeGraphResponse {
    nodes: SymbolNode[];
    links: SymbolEdge[];
    anomalies: string[];
}

export interface KnowledgeEntry {
    id: string;
    text: string;
    topic: string;
    cluster_id: string | null;
    source_node_id: string | null;
    source_agent_id: string | null;
    content_hash: string;
    confidence: number;
    human_confirmed: boolean;
    ttl: number | null;
    created_at: number;
    access_count: number;
    concept_type: string;
    title: string | null;
    description: string | null;
    resource_uri: string | null;
    tags: string | null;
}

export const intelligence_api_service = {
    /**
     * Fetches the full high-fidelity codebase symbol graph.
     */
    async get_code_graph(): Promise<CodeGraphResponse> {
        return api_request<CodeGraphResponse>('/v1/intelligence/graph', {
            method: 'GET',
        });
    },

    /**
     * Calculates the downstream impact (blast radius) of changing a specific symbol.
     * @param name The name of the symbol.
     * @param path The obfuscated path of the file containing the symbol.
     * @param signal Optional AbortSignal to cancel the request.
     */
    async get_blast_radius(name: string, path: string, signal?: AbortSignal): Promise<SymbolNode[]> {
        const query_params = new URLSearchParams({ name, path }).toString();
        return api_request<SymbolNode[]>(`/v1/intelligence/blast-radius?${query_params}`, {
            method: 'GET',
            signal,
        });
    },

    /**
     * Fetches paginated OKF knowledge entries from the IKS.
     */
    async get_knowledge(params?: {
        topic?: string;
        cluster_id?: string;
        concept_type?: string;
        limit?: number;
        offset?: number;
    }, signal?: AbortSignal): Promise<KnowledgeEntry[]> {
        const query_params = new URLSearchParams();
        if (params?.topic) query_params.append('topic', params.topic);
        if (params?.cluster_id) query_params.append('cluster_id', params.cluster_id);
        if (params?.concept_type) query_params.append('concept_type', params.concept_type);
        if (params?.limit !== undefined) query_params.append('limit', params.limit.toString());
        if (params?.offset !== undefined) query_params.append('offset', params.offset.toString());

        const query_str = query_params.toString();
        return api_request<KnowledgeEntry[]>(`/v1/knowledge?${query_str}`, {
            method: 'GET',
            signal,
        });
    },

    /**
     * Fetches semantic peer nodes for a specific OKF knowledge entry.
     */
    async get_knowledge_peers(id: string, limit?: number, signal?: AbortSignal): Promise<KnowledgeEntry[]> {
        const query_params = new URLSearchParams();
        if (limit !== undefined) query_params.append('limit', limit.toString());
        const query_str = query_params.toString();
        return api_request<KnowledgeEntry[]>(`/v1/knowledge/${id}/peers?${query_str}`, {
            method: 'GET',
            signal,
        });
    }
};

// Metadata: [intelligence_api_service]
