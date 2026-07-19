/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[engine_api_test]` in observability traces.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { engine_api } from './engine_api';
import { api_request } from '../base_api_service';

vi.mock('../base_api_service', () => ({
    api_request: vi.fn(),
    DEPLOY_TIMEOUT: 7200000
}));

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
        it('returns true when engine is healthy', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });
            const result = await engine_api.check_health();
            expect(result).toBe(true);
        });

        it('returns false when engine status is null', async () => {
            vi.mocked(api_request).mockRejectedValueOnce(new Error('timeout'));
            const result = await engine_api.check_health();
            expect(result).toBe(false);
        });
    });

    describe('deploy_engine', () => {
        it('calls POST /v1/engine/deploy without target', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'deployed' });
            const result = await engine_api.deploy_engine();
            expect(result.status).toBe('deployed');
            expect(api_request).toHaveBeenCalledWith('/v1/engine/deploy', expect.objectContaining({
                method: 'POST',
                timeout: 7200000
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
    });

    describe('kill_agents', () => {
        it('calls POST /v1/engine/kill', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await engine_api.kill_agents();
            expect(api_request).toHaveBeenCalledWith('/v1/engine/kill', expect.objectContaining({
                method: 'POST'
            }));
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
    });

    describe('transcribe', () => {
        it('sends audio_blob in FormData to /v1/engine/transcribe', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ text: 'transcribed speech' });
            const mock_blob = new Blob(['audio'], { type: 'audio/wav' });

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
        });

        it('returns empty string if response has no text field', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            const result = await engine_api.transcribe(new Blob());
            expect(result).toBe('');
        });
    });

    describe('install_template', () => {
        it('calls POST /v1/engine/templates/install with template details', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});
            await engine_api.install_template('http://github.com/repo', '/src/templates');

            expect(api_request).toHaveBeenCalledWith('/v1/engine/templates/install', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({ repository_url: 'http://github.com/repo', path: '/src/templates' })
            }));
        });
    });
});

// Metadata: [engine_api_test]
