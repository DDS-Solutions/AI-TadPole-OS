/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[infra_api_test]` in observability traces.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { infra_api } from './infra_api';
import { api_request } from '../base_api_service';

vi.mock('../base_api_service', () => ({
    api_request: vi.fn()
}));

describe('infra_api', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('test_provider', () => {
        it('calls POST /v1/infra/providers/{id}/test with config', async () => {
            const config = { id: 'prov-1', name: 'OpenAI', protocol: 'openai_chat' };
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok', latency: 150 });

            const result = await infra_api.test_provider(config);
            expect(result).toEqual({ status: 'ok', latency: 150 });
            expect(api_request).toHaveBeenCalledWith('/v1/infra/providers/prov-1/test', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify(config)
            }));
        });

        it('blocks directory traversal attempts in test_provider', async () => {
            const result = await infra_api.test_provider({ id: 'prov/../../bad', name: 'Bad', protocol: 'test' });
            expect(result.status).toBe('error');
            expect(result.message).toContain('Invalid identifier');
            expect(api_request).not.toHaveBeenCalled();
        });

        it('returns standardized error message when connection times out', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('timed out'));
            const result = await infra_api.test_provider({ id: 'p1', name: 'P', protocol: 'test' });
            expect(result).toEqual({ status: 'error', message: 'Handshake timeout: The provider endpoint is unresponsive.' });
        });
    });

    describe('get_nodes', () => {
        it('calls GET /v1/infra/nodes', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await infra_api.get_nodes();
            expect(api_request).toHaveBeenCalledWith('/v1/infra/nodes', expect.objectContaining({ method: 'GET' }));
        });
    });

    describe('discover_nodes', () => {
        it('calls POST /v1/infra/nodes/discover', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok', discovered: [] });
            await infra_api.discover_nodes();
            expect(api_request).toHaveBeenCalledWith('/v1/infra/nodes/discover', expect.objectContaining({ method: 'POST' }));
        });
    });

    describe('get_providers', () => {
        it('calls GET /v1/infra/providers', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await infra_api.get_providers();
            expect(api_request).toHaveBeenCalledWith('/v1/infra/providers', expect.objectContaining({ method: 'GET' }));
        });
    });

    describe('update_provider', () => {
        it('calls PUT on safe id', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });
            await infra_api.update_provider('prov-2', { name: 'updated' });
            expect(api_request).toHaveBeenCalledWith('/v1/infra/providers/prov-2', expect.objectContaining({
                method: 'PUT',
                body: JSON.stringify({ name: 'updated' })
            }));
        });

        it('blocks traversal ids', async () => {
            await expect(infra_api.update_provider('../bad', {}))
                .rejects.toThrow('Invalid identifier');
        });
    });

    describe('delete_provider', () => {
        it('calls DELETE on safe id', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await infra_api.delete_provider('prov-3');
            expect(api_request).toHaveBeenCalledWith('/v1/infra/providers/prov-3', expect.objectContaining({
                method: 'DELETE'
            }));
        });

        it('blocks invalid characters', async () => {
            await expect(infra_api.delete_provider('p*'))
                .rejects.toThrow('Invalid identifier');
        });
    });

    describe('sync_provider_models', () => {
        it('calls POST on safe id', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'success' });
            await infra_api.sync_provider_models('prov-4', 'sk-key');
            expect(api_request).toHaveBeenCalledWith('/v1/infra/providers/prov-4/sync', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({ api_key: 'sk-key' })
            }));
        });

        it('blocks traversal attempt', async () => {
            await expect(infra_api.sync_provider_models('p/../s', 'sk'))
                .rejects.toThrow('Invalid identifier');
        });
    });

    describe('update_model', () => {
        it('calls PUT on safe id', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });
            await infra_api.update_model('m-1', { vram: '8GB' });
            expect(api_request).toHaveBeenCalledWith('/v1/infra/models/m-1', expect.objectContaining({
                method: 'PUT',
                body: JSON.stringify({ vram: '8GB' })
            }));
        });
    });

    describe('delete_model', () => {
        it('calls DELETE on safe id', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await infra_api.delete_model('m-2');
            expect(api_request).toHaveBeenCalledWith('/v1/infra/models/m-2', expect.objectContaining({
                method: 'DELETE'
            }));
        });
    });

    describe('get_models', () => {
        it('calls GET /v1/infra/models', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await infra_api.get_models();
            expect(api_request).toHaveBeenCalledWith('/v1/infra/models', expect.objectContaining({ method: 'GET' }));
        });
    });

    describe('get_model_catalog', () => {
        it('calls GET /v1/infra/model-store/catalog', async () => {
            vi.mocked(api_request).mockResolvedValueOnce([]);
            await infra_api.get_model_catalog();
            expect(api_request).toHaveBeenCalledWith('/v1/infra/model-store/catalog', expect.objectContaining({ method: 'GET' }));
        });
    });

    describe('pull_model', () => {
        it('calls POST /v1/infra/model-store/pull with tag and node', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });
            await infra_api.pull_model('model-a', 'node-1');
            expect(api_request).toHaveBeenCalledWith('/v1/infra/model-store/pull', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({ tag: 'model-a', node_id: 'node-1' })
            }));
        });
    });
});

// Metadata: [infra_api_test]
