/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / intelligence_api_service.test
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
import { intelligence_api_service } from './intelligence_api_service';
import { api_request } from './base_api_service';

vi.mock('./base_api_service', () => ({
    api_request: vi.fn(),
}));

describe('intelligence_api_service', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('get_code_graph calls /v1/intelligence/graph via GET', async () => {
        const mock_graph = {
            nodes: [{ id: '1', name: 'App', kind: 'class', path: '/src/App.tsx' }],
            links: [{ source: '1', target: '2' }],
            anomalies: []
        };
        vi.mocked(api_request).mockResolvedValueOnce(mock_graph);

        const result = await intelligence_api_service.get_code_graph();
        expect(api_request).toHaveBeenCalledWith('/v1/intelligence/graph', {
            method: 'GET'
        });
        expect(result).toEqual(mock_graph);
    });

    it('get_blast_radius serializes name and path params and propagates AbortSignal', async () => {
        const mock_nodes = [{ id: 'node-1', name: 'AgentStore', kind: 'variable', path: 'src/stores/agent_store.ts' }];
        vi.mocked(api_request).mockResolvedValueOnce(mock_nodes);

        const controller = new AbortController();
        const result = await intelligence_api_service.get_blast_radius('AgentStore', 'src/stores/agent_store.ts', controller.signal);

        expect(api_request).toHaveBeenCalledWith(
            '/v1/intelligence/blast-radius?name=AgentStore&path=src%2Fstores%2Fagent_store.ts',
            {
                method: 'GET',
                signal: controller.signal
            }
        );
        expect(result).toEqual(mock_nodes);
    });

    it('get_knowledge formats all query parameters correctly', async () => {
        const mock_entries = [{
            id: 'k-1',
            text: 'Architecture pattern',
            topic: 'design',
            cluster_id: 'cl-1',
            source_node_id: null,
            source_agent_id: null,
            content_hash: 'abc',
            confidence: 0.95,
            human_confirmed: true,
            ttl: null,
            created_at: 1000,
            access_count: 5,
            concept_type: 'pattern',
            title: 'Pattern',
            description: null,
            resource_uri: null,
            tags: 'tag1'
        }];
        vi.mocked(api_request).mockResolvedValueOnce(mock_entries);

        const result = await intelligence_api_service.get_knowledge({
            topic: 'security',
            cluster_id: 'cluster-9',
            concept_type: 'law',
            limit: 20,
            offset: 40
        });

        expect(api_request).toHaveBeenCalledWith(
            '/v1/knowledge?topic=security&cluster_id=cluster-9&concept_type=law&limit=20&offset=40',
            {
                method: 'GET',
                signal: undefined
            }
        );
        expect(result).toEqual(mock_entries);
    });

    it('get_knowledge handles empty params', async () => {
        vi.mocked(api_request).mockResolvedValueOnce([]);
        const result = await intelligence_api_service.get_knowledge();
        expect(api_request).toHaveBeenCalledWith('/v1/knowledge?', {
            method: 'GET',
            signal: undefined
        });
        expect(result).toEqual([]);
    });

    it('get_knowledge_peers calls /v1/knowledge/:id/peers with optional limit', async () => {
        vi.mocked(api_request).mockResolvedValueOnce([]);
        const controller = new AbortController();
        const result = await intelligence_api_service.get_knowledge_peers('k-entry-1', 5, controller.signal);

        expect(api_request).toHaveBeenCalledWith(
            '/v1/knowledge/k-entry-1/peers?limit=5',
            {
                method: 'GET',
                signal: controller.signal
            }
        );
        expect(result).toEqual([]);
    });
});
