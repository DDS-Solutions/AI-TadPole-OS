/**
 * @docs ARCHITECTURE:Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / mission_history_service
 * - **Primary Entrypoints**: `mission_history_service`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { api_request } from './base_api_service';
import type { 
    CatalogResponse, 
    MissionDossier 
} from '../types/mission_dossier';

export interface CatalogQueryParams {
    page?: number;
    per_page?: number;
    status?: string;
    search?: string;
}

export const mission_history_service = {
    /**
     * Fetches paginated catalog of all missions with aggregated counts.
     */
    fetch_catalog: async (params?: CatalogQueryParams): Promise<CatalogResponse> => {
        const search_params = new URLSearchParams();
        if (params?.page) search_params.set('page', params.page.toString());
        if (params?.per_page) search_params.set('per_page', params.per_page.toString());
        if (params?.status && params.status !== 'all') search_params.set('status', params.status);
        if (params?.search) search_params.set('search', params.search);

        const query_str = search_params.toString();
        const endpoint = `/v1/missions${query_str ? `?${query_str}` : ''}`;
        return api_request<CatalogResponse>(endpoint, { method: 'GET' });
    },

    /**
     * Fetches complete composite flight dossier for a specific mission.
     */
    fetch_dossier: async (mission_id: string): Promise<MissionDossier> => {
        const clean_id = encodeURIComponent(mission_id.trim());
        return api_request<MissionDossier>(`/v1/missions/${clean_id}/dossier`, { method: 'GET' });
    },

    /**
     * Toggles the retention pin status of a mission to protect it from Reaper cycles.
     */
    toggle_pin: async (mission_id: string): Promise<{ success: boolean; is_pinned: boolean; mission_id: string }> => {
        const clean_id = encodeURIComponent(mission_id.trim());
        return api_request<{ success: boolean; is_pinned: boolean; mission_id: string }>(
            `/v1/missions/${clean_id}/pin`, 
            { method: 'POST' }
        );
    }
};
