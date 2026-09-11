/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / engine_api.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `engine_api.test.ts`
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { engine_api } from './engine_api';
import { api_request, DEPLOY_TIMEOUT } from '../base_api_service';

vi.mock('../base_api_service', async (importOriginal) => {
    const actual = await importOriginal<typeof import('../base_api_service')>();
    return {
        ...actual,
        api_request: vi.fn(),
        DEPLOY_TIMEOUT: 7200000
    };
});

describe('engine_api', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('get_engine_status', () => {
        it('calls GET /v1/engine/health with default timeout', async () => {
            const mock_status = { status: 'healthy', version: '1.0.0', heartbeat: 'ok', active_agents: 4, features: ['tts'] };
            vi.mocked(api_request).mockResolvedValueOnce(mock_status);

            const result = await engine_api.get_engine_status();
            expect(result).toEqual(mock_status);
            expect(api_request).toHaveBeenCalledWith('/v1/engine/health', expect.objectContaining({
                method: 'GET',
                timeout: 5000
            }));
        });

        it('returns null on request failure', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('connection failed'));
            const result = await engine_api.get_engine_status();
            expect(result).toBeNull();
        });

        it('propagates custom options like signal', async () => {
            const controller = new AbortController();
            vi.mocked(api_request).mockResolvedValueOnce({});

            await engine_api.get_engine_status({ signal: controller.signal, timeout: 3000 });
            expect(api_request).toHaveBeenCalledWith('/v1/engine/health', expect.objectContaining({
                signal: controller.signal,
                timeout: 3000
            }));
        });
    });

    describe('check_health', () => {
        it('returns true when engine returns ok or healthy status payload', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });
            const resultOk = await engine_api.check_health();
            expect(resultOk).toBe(true);

            vi.mocked(api_request).mockResolvedValueOnce({ status: 'healthy', version: '1.0' });
            const resultHealthy = await engine_api.check_health();
            expect(resultHealthy).toBe(true);
        });

        it('returns false when engine status is null or request throws', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('timeout'));
            const result = await engine_api.check_health();
            expect(result).toBe(false);
        });
    });

    describe('deploy_engine', () => {
        it('calls POST /v1/engine/deploy without target and uses DEPLOY_TIMEOUT', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'deployed' });
            const result = await engine_api.deploy_engine();
            expect(result.status).toBe('deployed');
            expect(api_request).toHaveBeenCalledWith('/v1/engine/deploy', expect.objectContaining({
                method: 'POST',
                timeout: DEPLOY_TIMEOUT
            }));
        });

        it('calls POST and properly encodes target parameter', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'deployed' });
            await engine_api.deploy_engine('prod v2 & cache');
            expect(api_request).toHaveBeenCalledWith('/v1/engine/deploy?target=prod%20v2%20%26%20cache', expect.objectContaining({
                method: 'POST'
            }));
        });

        it('propagates options parameters', async () => {
            const controller = new AbortController();
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });
            await engine_api.deploy_engine(undefined, { signal: controller.signal, timeout: 10000 });
            expect(api_request).toHaveBeenCalledWith('/v1/engine/deploy', expect.objectContaining({
                signal: controller.signal,
                timeout: 10000
            }));
        });

        it('propagates errors when deploy_engine rejects', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('Deploy runner crashed'));
            await expect(engine_api.deploy_engine()).rejects.toThrow('Deploy runner crashed');
        });
    });

    describe('speak', () => {
        it('calls POST /v1/engine/speak with body and response_type blob', async () => {
            const mock_blob = new Blob(['audio_data'], { type: 'audio/wav' });
            vi.mocked(api_request).mockResolvedValueOnce(mock_blob);

            const result = await engine_api.speak('hello', 'en-US', 'neural');
            expect(result).toBe(mock_blob);
            expect(api_request).toHaveBeenCalledWith('/v1/engine/speak', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({ text: 'hello', voice: 'en-US', engine: 'neural' }),
                response_type: 'blob'
            }));
        });

        it('propagates options parameters', async () => {
            const controller = new AbortController();
            vi.mocked(api_request).mockResolvedValueOnce(new Blob());
            await engine_api.speak('test', undefined, undefined, { signal: controller.signal, timeout: 1000 });
            expect(api_request).toHaveBeenCalledWith('/v1/engine/speak', expect.objectContaining({
                signal: controller.signal,
                timeout: 1000
            }));
        });

        it('propagates errors when speak rejects', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('TTS synthesis failed'));
            await expect(engine_api.speak('fail')).rejects.toThrow('TTS synthesis failed');
        });
    });

    describe('kill_agents', () => {
        it('calls POST /v1/engine/kill', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await engine_api.kill_agents();
            expect(api_request).toHaveBeenCalledWith('/v1/engine/kill', expect.objectContaining({
                method: 'POST'
            }));
        });

        it('propagates errors when kill_agents rejects', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('Kill switch failed'));
            await expect(engine_api.kill_agents()).rejects.toThrow('Kill switch failed');
        });
    });

    describe('shutdown_engine', () => {
        it('calls POST /v1/engine/shutdown', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await engine_api.shutdown_engine();
            expect(api_request).toHaveBeenCalledWith('/v1/engine/shutdown', expect.objectContaining({
                method: 'POST'
            }));
        });

        it('propagates errors when shutdown_engine rejects', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('Permission denied'));
            await expect(engine_api.shutdown_engine()).rejects.toThrow('Permission denied');
        });
    });

    describe('transcribe', () => {
        it('sends audio_blob in FormData to /v1/engine/transcribe and verifies blob attachment', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ text: 'transcribed speech' });
            const mock_blob = new Blob(['audio sample payload'], { type: 'audio/wav' });

            const result = await engine_api.transcribe(mock_blob);
            expect(result).toBe('transcribed speech');
            expect(api_request).toHaveBeenCalledWith('/v1/engine/transcribe', expect.objectContaining({
                method: 'POST',
                body: expect.any(FormData)
            }));

            const sent_body = vi.mocked(api_request).mock.calls[0][1]?.body as FormData;
            const file = sent_body.get('file') as any;
            expect(file).toBeDefined();
            expect(file.name).toBe('speech.wav');
            expect(file.size).toBe(mock_blob.size);
        });

        it('returns empty string if response has no text field', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            const result = await engine_api.transcribe(new Blob());
            expect(result).toBe('');
        });

        it('propagates errors when transcribe rejects', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('Audio decoding error'));
            await expect(engine_api.transcribe(new Blob())).rejects.toThrow('Audio decoding error');
        });
    });

    describe('install_template', () => {
        it('calls POST /v1/engine/templates/install with template details', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'success', message: 'ok' });
            await engine_api.install_template('http://github.com/repo', '/src/templates');

            expect(api_request).toHaveBeenCalledWith('/v1/engine/templates/install', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({
                    repository_url: 'http://github.com/repo',
                    path: '/src/templates',
                    model_override: undefined,
                    overwrite: false,
                    namespace: undefined
                })
            }));
        });

        it('calls POST /v1/engine/templates/install with model_override, overwrite, and namespace', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'success', message: 'ok' });
            const modelOverride = { provider: 'ollama', model_id: 'gemma4:e4b', base_url: 'http://127.0.0.1:11434' };
            await engine_api.install_template('http://github.com/repo', '/src/templates', modelOverride, true, 'field_services');

            expect(api_request).toHaveBeenCalledWith('/v1/engine/templates/install', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({
                    repository_url: 'http://github.com/repo',
                    path: '/src/templates',
                    model_override: modelOverride,
                    overwrite: true,
                    namespace: 'field_services'
                })
            }));
        });

        it('propagates errors when install_template rejects', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('Git clone failed: 404 Not Found'));
            await expect(engine_api.install_template('http://github.com/repo', 'missing')).rejects.toThrow('Git clone failed: 404 Not Found');
        });
    });

    describe('import_template', () => {
        it('calls POST /v1/engine/templates/import with local payload, namespace, and model_override', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'success', message: 'ok' });
            const payload = {
                swarm: { name: 'Test Swarm' },
                agents: [{ filename: 'agent-1.json', content: { id: 'agent-1' } }],
                workflows: [{ filename: 'test.md', content: '# Step 1' }],
                overwrite: true,
                model_override: { provider: 'ollama-cloud', model_id: 'gemma4:31b-cloud' },
                namespace: 'mkt'
            };
            const res = await engine_api.import_template(payload);

            expect(api_request).toHaveBeenCalledWith('/v1/engine/templates/import', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify(payload)
            }));
            expect(res.status).toBe('success');
        });

        it('propagates errors when import_template rejects', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('Invalid swarm schema'));
            await expect(engine_api.import_template({ swarm: {} })).rejects.toThrow('Invalid swarm schema');
        });
    });

    describe('get_installed_templates', () => {
        it('calls GET /v1/engine/templates/installed and returns installed swarms', async () => {
            const mockSwarms = [{
                id: 'marketing',
                name: 'Marketing Swarm',
                description: 'Automated marketing',
                industry: 'Marketing',
                installed_at: '2026-08-25T15:00:00Z',
                template_path: 'templates/marketing',
                agents: ['lead_gen', 'copywriter'],
                workflows: ['daily_campaign.md'],
                mcp_servers: ['hubspot']
            }];
            vi.mocked(api_request).mockResolvedValueOnce({ swarms: mockSwarms });

            const res = await engine_api.get_installed_templates();
            expect(api_request).toHaveBeenCalledWith('/v1/engine/templates/installed', expect.objectContaining({
                method: 'GET'
            }));
            expect(res.swarms).toEqual(mockSwarms);
        });
    });

    describe('uninstall_template', () => {
        it('calls POST /v1/engine/templates/uninstall with swarm_id and archive options', async () => {
            const mockResponse = {
                status: 'success',
                message: 'Uninstalled',
                uninstalled_agents: ['lead_gen'],
                uninstalled_workflows: ['daily_campaign.md'],
                uninstalled_mcp_servers: ['hubspot'],
                archived_path: 'data/swarm_config/archive/marketing/20260825'
            };
            vi.mocked(api_request).mockResolvedValueOnce(mockResponse);

            const res = await engine_api.uninstall_template('marketing', true);
            expect(api_request).toHaveBeenCalledWith('/v1/engine/templates/uninstall', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({ swarm_id: 'marketing', archive: true })
            }));
            expect(res).toEqual(mockResponse);
        });
    });

    describe('update_environment', () => {
        it('calls POST /v1/system/environment with variables payload', async () => {
            const mockResponse = {
                status: 'success',
                updated_keys: ['STRIPE_API_KEY', 'QUICKBOOKS_CLIENT_ID']
            };
            vi.mocked(api_request).mockResolvedValueOnce(mockResponse);

            const variables = {
                STRIPE_API_KEY: 'sk_test_12345',
                QUICKBOOKS_CLIENT_ID: 'qb_client_xyz'
            };
            const res = await engine_api.update_environment(variables);
            expect(api_request).toHaveBeenCalledWith('/v1/system/environment', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({ variables })
            }));
            expect(res).toEqual(mockResponse);
        });
    });
});
