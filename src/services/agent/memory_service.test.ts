/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / memory_service.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AgentMemoryService } from './memory_service';

describe('AgentMemoryService', () => {
    let mock_api_request: any;
    let service: AgentMemoryService;

    beforeEach(() => {
        mock_api_request = vi.fn();
        service = new AgentMemoryService(mock_api_request);
    });

    it('get_agent_memory returns normalized entries on success', async () => {
        const raw_entries = [
            {
                rowid: 1,
                agent_id: 'agent-1',
                text: 'entry 1',
                created_at: '2026-07-10T12:00:00Z',
                embedding: []
            }
        ];
        mock_api_request.mockResolvedValue({ status: 'ok', entries: raw_entries });

        const result = await service.get_agent_memory('agent-1');

        expect(mock_api_request).toHaveBeenCalledWith('/v1/agents/agent-1/memories', { method: 'GET' });
        expect(result.status).toBe('ok');
        expect(result.entries).toHaveLength(1);
        expect(result.entries[0].text).toBe('entry 1');
    });

    it('get_agent_memory throws mapped API error on failure', async () => {
        mock_api_request.mockRejectedValue(new Error('Network failure'));

        await expect(service.get_agent_memory('agent-1')).rejects.toThrow();
    });

    it('delete_agent_memory calls delete endpoint', async () => {
        mock_api_request.mockResolvedValue({ status: 'deleted' });

        const result = await service.delete_agent_memory('agent-1', 'row-123');

        expect(mock_api_request).toHaveBeenCalledWith('/v1/agents/agent-1/memories/row-123', { method: 'DELETE' });
        expect(result.status).toBe('deleted');
    });

    it('delete_agent_memory throws on failure', async () => {
        mock_api_request.mockRejectedValue(new Error('Failed to delete'));

        await expect(service.delete_agent_memory('agent-1', 'row-123')).rejects.toThrow();
    });

    it('save_agent_memory calls post endpoint with JSON body', async () => {
        mock_api_request.mockResolvedValue({ status: 'ok', id: 'new-id' });

        const result = await service.save_agent_memory('agent-1', 'fresh memory');

        expect(mock_api_request).toHaveBeenCalledWith('/v1/agents/agent-1/memories', {
            method: 'POST',
            body: JSON.stringify({ text: 'fresh memory' })
        });
        expect(result.status).toBe('ok');
        expect(result.id).toBe('new-id');
    });

    it('save_agent_memory throws on failure', async () => {
        mock_api_request.mockRejectedValue(new Error('Save failed'));

        await expect(service.save_agent_memory('agent-1', 'fresh memory')).rejects.toThrow();
    });

    it('search_memory queries endpoint with query parameter', async () => {
        mock_api_request.mockResolvedValue({ status: 'ok', entries: [] });

        const result = await service.search_memory('needle', 'agent-1');

        expect(mock_api_request).toHaveBeenCalledWith('/v1/search/memory?query=needle&agent_id=agent-1', {
            method: 'GET'
        });
        expect(result.status).toBe('ok');
    });

    it('search_memory queries endpoint without agent_id parameter', async () => {
        mock_api_request.mockResolvedValue({ status: 'ok', entries: [] });

        await service.search_memory('needle');

        expect(mock_api_request).toHaveBeenCalledWith('/v1/search/memory?query=needle', {
            method: 'GET'
        });
    });

    it('search_memory throws on failure', async () => {
        mock_api_request.mockRejectedValue(new Error('Search failed'));

        await expect(service.search_memory('needle')).rejects.toThrow();
    });
});
