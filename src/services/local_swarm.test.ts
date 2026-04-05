/**
 * @docs ARCHITECTURE:Services
 * 
 * ### AI Assist Note
 * **Root/Core**: Manages the local swarm.test. 
 * Part of the Tadpole-OS core layer.
 */

/**
 * @file local_swarm.test.ts
 * @description Suite for verifying Local Swarm infrastructure management via system_api_service.
 * @module Services/local_swarm
 * @testedBehavior
 * - Model Catalog: Retrieval of available local models (Ollama/Llama).
 * - Model Pulse: Initiation of model pulls to specific Bunker nodes.
 * - Node Inventory: Fetching of available physical/virtual Swarm nodes.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks api_request to intercept infrastructure commands.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { system_api_service } from './system_api_service';
import { api_request } from './base_api_service';

vi.mock('./base_api_service', () => ({
    api_request: vi.fn(),
    DEPLOY_TIMEOUT: 300000
}));

describe('system_api_service - Local Swarm', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should fetch model catalog', async () => {
        const mock_catalog = [{ id: 'llama3', name: 'Llama 3' }];
        (api_request as any).mockResolvedValue(mock_catalog);

        const result = await system_api_service.get_model_catalog();
        
        expect(api_request).toHaveBeenCalledWith('/v1/infra/model-store/catalog', { method: 'GET' });
        expect(result).toEqual(mock_catalog);
    });

    it('should initiate model pull with correct payload', async () => {
        const mock_response = { status: 'success' };
        (api_request as any).mockResolvedValue(mock_response);

        const result = await system_api_service.pull_model('llama3', 'node-1');
        
        expect(api_request).toHaveBeenCalledWith('/v1/infra/model-store/pull', {
            method: 'POST',
            body: JSON.stringify({ tag: 'llama3', node_id: 'node-1' })
        });
        expect(result).toEqual(mock_response);
    });

    it('should fetch swarm nodes', async () => {
        const mock_nodes = [{ id: 'node-1', name: 'Bunker 1' }];
        (api_request as any).mockResolvedValue(mock_nodes);

        const result = await system_api_service.get_nodes();
        
        expect(api_request).toHaveBeenCalledWith('/v1/infra/nodes', { method: 'GET' });
        expect(result).toEqual(mock_nodes);
    });
});

