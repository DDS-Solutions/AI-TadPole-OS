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

export interface SymbolNode {
    name: string;
    path: string;
    kind: string;
    signature: string;
    start_line: number;
    end_line: number;
}

export interface SymbolEdge {
    source: string;
    target: string;
}

export interface CodeGraphResponse {
    nodes: SymbolNode[];
    links: SymbolEdge[];
    anomalies: string[];
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
     */
    async get_blast_radius(name: string, path: string): Promise<SymbolNode[]> {
        const query_params = new URLSearchParams({ name, path }).toString();
        return api_request<SymbolNode[]>(`/v1/intelligence/blast-radius?${query_params}`, {
            method: 'GET',
        });
    }
};

// Metadata: [intelligence_api_service]
