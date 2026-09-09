/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / local_swarm.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { vi, describe, it, expect, beforeEach } from 'vitest';
import { system_api_service } from './system_api_service';
import { api_request } from './base_api_service';

vi.mock('./base_api_service', async (importOriginal) => {
    const actual = await importOriginal<any>();
    return {
        ...actual,
        api_request: vi.fn(),
        DEPLOY_TIMEOUT: 300000
    };
});

describe('system_api_service - Local Swarm', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should fetch model catalog', async () => {
        const mock_catalog = [{ id: 'llama3', name: 'Llama 3' }];
        (api_request as any).mockResolvedValue(mock_catalog);

        const result = await system_api_service.infra.get_model_catalog();
        
        expect(api_request).toHaveBeenCalledWith('/v1/infra/model-store/catalog', { method: 'GET' });
        expect(result).toEqual(mock_catalog);
    });

    it('should initiate model pull with correct payload', async () => {
        const mock_response = { status: 'success' };
        (api_request as any).mockResolvedValue(mock_response);

        const result = await system_api_service.infra.pull_model('llama3', 'node-1');
        
        expect(api_request).toHaveBeenCalledWith('/v1/infra/model-store/pull', {
            method: 'POST',
            body: JSON.stringify({ tag: 'llama3', node_id: 'node-1' })
        });
        expect(result).toEqual(mock_response);
    });

    it('should fetch swarm nodes', async () => {
        const mock_nodes = [{ id: 'node-1', name: 'Bunker 1' }];
        (api_request as any).mockResolvedValue(mock_nodes);

        const result = await system_api_service.infra.get_nodes();
        
        expect(api_request).toHaveBeenCalledWith('/v1/infra/nodes', { method: 'GET' });
        expect(result).toEqual(mock_nodes);
    });
});
