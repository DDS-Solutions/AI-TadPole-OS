import { vi, describe, it, expect, beforeEach } from 'vitest';
import { system_api_service } from './system_api_service';
import { apiRequest } from './base_api_service';

vi.mock('./base_api_service', () => ({
    apiRequest: vi.fn(),
    DEPLOY_TIMEOUT: 300000
}));

describe('system_api_service - Local Swarm', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should fetch model catalog', async () => {
        const mockCatalog = [{ id: 'llama3', name: 'Llama 3' }];
        (apiRequest as any).mockResolvedValue(mockCatalog);

        const result = await system_api_service.get_model_catalog();
        
        expect(apiRequest).toHaveBeenCalledWith('/v1/infra/model-store/catalog', { method: 'GET' });
        expect(result).toEqual(mockCatalog);
    });

    it('should initiate model pull with correct payload', async () => {
        const mockResponse = { status: 'success' };
        (apiRequest as any).mockResolvedValue(mockResponse);

        const result = await system_api_service.pull_model('llama3', 'node-1');
        
        expect(apiRequest).toHaveBeenCalledWith('/v1/infra/model-store/pull', {
            method: 'POST',
            body: JSON.stringify({ tag: 'llama3', nodeId: 'node-1' })
        });
        expect(result).toEqual(mockResponse);
    });

    it('should fetch swarm nodes', async () => {
        const mockNodes = [{ id: 'node-1', name: 'Bunker 1' }];
        (apiRequest as any).mockResolvedValue(mockNodes);

        const result = await system_api_service.get_nodes();
        
        expect(apiRequest).toHaveBeenCalledWith('/v1/infra/nodes', { method: 'GET' });
        expect(result).toEqual(mockNodes);
    });
});
