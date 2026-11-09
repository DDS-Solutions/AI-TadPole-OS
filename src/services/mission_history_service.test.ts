/**
 * @docs ARCHITECTURE:Services
 *
 * ### AI Assist Note
 * Unit tests for mission_history_service query parameter serialization and error handling.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mission_history_service } from './mission_history_service';
import { api_request } from './base_api_service';

vi.mock('./base_api_service', () => ({
    api_request: vi.fn(),
}));

describe('mission_history_service', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should call GET /v1/missions with default query when no params provided', async () => {
        const mock_catalog = {
            success: true,
            data: [],
            total: 0,
            page: 1,
            per_page: 25,
        };
        vi.mocked(api_request).mockResolvedValueOnce(mock_catalog);

        const res = await mission_history_service.fetch_catalog();

        expect(api_request).toHaveBeenCalledWith('/v1/missions', { method: 'GET' });
        expect(res).toEqual(mock_catalog);
    });

    it('should serialize query params correctly for status, page, and search', async () => {
        const mock_catalog = {
            success: true,
            data: [],
            total: 5,
            page: 2,
            per_page: 10,
        };
        vi.mocked(api_request).mockResolvedValueOnce(mock_catalog);

        const res = await mission_history_service.fetch_catalog({
            page: 2,
            per_page: 10,
            status: 'completed',
            search: 'refactor',
        });

        expect(api_request).toHaveBeenCalledWith(
            '/v1/missions?page=2&per_page=10&status=completed&search=refactor',
            { method: 'GET' }
        );
        expect(res.page).toBe(2);
    });

    it('should call GET /v1/missions/:id/dossier for specific mission', async () => {
        const mock_dossier = {
            mission: { id: 'm-123', title: 'Test Mission' },
            logs: [],
            diffs: [],
            memory_palace: [],
            trust_graph: { entities: [], relations: [] },
            economics: { total_cost_usd: 0, budget_usd: 1, transactions: [] },
            debrief: { agent99_notes: null, drift_score: null, verdict: 'VERIFIED_SUCCESS', recommendations: [] },
        };
        vi.mocked(api_request).mockResolvedValueOnce(mock_dossier);

        const res = await mission_history_service.fetch_dossier('m-123');

        expect(api_request).toHaveBeenCalledWith('/v1/missions/m-123/dossier', { method: 'GET' });
        expect(res.mission.id).toBe('m-123');
    });

    it('should call POST /v1/missions/:id/pin to toggle retention pin', async () => {
        const mock_pin_res = {
            success: true,
            is_pinned: true,
            mission_id: 'm-123',
        };
        vi.mocked(api_request).mockResolvedValueOnce(mock_pin_res);

        const res = await mission_history_service.toggle_pin('m-123');

        expect(api_request).toHaveBeenCalledWith('/v1/missions/m-123/pin', { method: 'POST' });
        expect(res.is_pinned).toBe(true);
    });
});
