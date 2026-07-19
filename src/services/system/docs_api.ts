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
import type { RequestOptions } from '../system_api_types';

const sanitize_segment = (segment: string): string => {
    if (!segment || !/^[a-zA-Z0-9_.-]{1,64}$/.test(segment)) {
        throw new Error(`Invalid path segment: ${segment}`);
    }
    return encodeURIComponent(segment);
};

export const docs_api = {
    get_knowledge_docs: async (options?: RequestOptions): Promise<{ category: string; name: string; title: string; }[]> => {
        return api_request<{ category: string; name: string; title: string; }[]>('/v1/docs/knowledge', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    get_knowledge_doc: async (category: string, name: string, options?: RequestOptions): Promise<string> => {
        const clean_category = sanitize_segment(category);
        const clean_name = sanitize_segment(name);
        return api_request<string>(`/v1/docs/knowledge/${clean_category}/${clean_name}`, {
            method: 'GET',
            headers: { 'Accept': 'text/markdown' },
            response_type: 'text',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    get_operations_manual: async (options?: RequestOptions): Promise<string> => {
        return api_request<string>('/v1/docs/operations-manual', {
            method: 'GET',
            headers: { 'Accept': 'text/markdown' },
            response_type: 'text',
            signal: options?.signal,
            timeout: options?.timeout
        });
    }
};

// Metadata: [docs_api]
