/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[docs_api]` in observability traces.
 */

import { api_request } from '../base_api_service';

export const docs_api = {
    get_knowledge_docs: async (): Promise<{ category: string; name: string; title: string; }[]> => {
        return api_request<{ category: string; name: string; title: string; }[]>('/v1/docs/knowledge', { method: 'GET' });
    },

    get_knowledge_doc: async (category: string, name: string): Promise<string> => {
        return api_request<string>(`/v1/docs/knowledge/${category}/${name}`, {
            method: 'GET',
            headers: { 'Accept': 'text/markdown' },
            response_type: 'text'
        });
    },

    get_operations_manual: async (): Promise<string> => {
        return api_request<string>('/v1/docs/operations-manual', {
            method: 'GET',
            headers: { 'Accept': 'text/markdown' },
            response_type: 'text'
        });
    }
};

// Metadata: [docs_api]
