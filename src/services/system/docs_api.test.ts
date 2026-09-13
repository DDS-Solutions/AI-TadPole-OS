/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / docs_api.test
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
import { docs_api } from './docs_api';
import { api_request } from '../base_api_service';

vi.mock('../base_api_service', () => ({
    api_request: vi.fn()
}));

describe('docs_api', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('get_knowledge_docs', () => {
        it('should call api_request with GET and correct url', async () => {
            const mock_data = [{ category: 'core', name: 'intro', title: 'Introduction' }];
            vi.mocked(api_request).mockResolvedValueOnce(mock_data);

            const result = await docs_api.get_knowledge_docs();
            expect(result).toBe(mock_data);
            expect(api_request).toHaveBeenCalledWith('/v1/docs/knowledge', expect.objectContaining({
                method: 'GET'
            }));
        });

        it('should propagate AbortSignal and custom timeout', async () => {
            const controller = new AbortController();
            vi.mocked(api_request).mockResolvedValueOnce([]);

            await docs_api.get_knowledge_docs({ signal: controller.signal, timeout: 5000 });
            expect(api_request).toHaveBeenCalledWith('/v1/docs/knowledge', expect.objectContaining({
                signal: controller.signal,
                timeout: 5000
            }));
        });
    });

    describe('get_knowledge_doc', () => {
        it('should successfully fetch doc when inputs are safe', async () => {
            vi.mocked(api_request).mockResolvedValueOnce('# Content');

            const result = await docs_api.get_knowledge_doc('core', 'intro-doc.v1');
            expect(result).toBe('# Content');
            expect(api_request).toHaveBeenCalledWith('/v1/docs/knowledge/core/intro-doc.v1', expect.objectContaining({
                method: 'GET',
                headers: { 'Accept': 'text/markdown' },
                response_type: 'text'
            }));
        });

        it('should block directory traversal attempt in category', async () => {
            await expect(docs_api.get_knowledge_doc('../etc', 'passwd'))
                .rejects.toThrow('Invalid path segment: ../etc');
            expect(api_request).not.toHaveBeenCalled();
        });

        it('should block directory traversal attempt in name', async () => {
            await expect(docs_api.get_knowledge_doc('core', 'subdir/../../file'))
                .rejects.toThrow('Invalid path segment: subdir/../../file');
            expect(api_request).not.toHaveBeenCalled();
        });

        it('should block zero-width characters and spaces in parameters', async () => {
            await expect(docs_api.get_knowledge_doc('core\u200b', 'intro'))
                .rejects.toThrow('Invalid path segment: core\u200b');
            await expect(docs_api.get_knowledge_doc('core', 'intro name'))
                .rejects.toThrow('Invalid path segment: intro name');
            expect(api_request).not.toHaveBeenCalled();
        });

        it('should block non-ASCII or malicious symbols', async () => {
            await expect(docs_api.get_knowledge_doc('core', 'intro"quote'))
                .rejects.toThrow('Invalid path segment: intro"quote');
            await expect(docs_api.get_knowledge_doc('core', 'intro★'))
                .rejects.toThrow('Invalid path segment: intro★');
            expect(api_request).not.toHaveBeenCalled();
        });

        it('should block parameters exceeding 64 characters', async () => {
            const long_param = 'a'.repeat(65);
            await expect(docs_api.get_knowledge_doc('core', long_param))
                .rejects.toThrow(`Invalid path segment: ${long_param}`);
            expect(api_request).not.toHaveBeenCalled();
        });

        it('should block empty/falsy parameters', async () => {
            await expect(docs_api.get_knowledge_doc('', 'intro'))
                .rejects.toThrow('Invalid path segment: ');
            expect(api_request).not.toHaveBeenCalled();
        });

        it('should allow and properly route safe parameters containing dashes, dots, and underscores', async () => {
            vi.mocked(api_request).mockResolvedValueOnce('# OK');

            const result = await docs_api.get_knowledge_doc('neural-link_v1', 'doc.v1');
            expect(result).toBe('# OK');
            expect(api_request).toHaveBeenCalledWith('/v1/docs/knowledge/neural-link_v1/doc.v1', expect.objectContaining({
                method: 'GET'
            }));
        });

        it('should propagate signal and timeout options', async () => {
            const controller = new AbortController();
            vi.mocked(api_request).mockResolvedValueOnce('# OK');

            await docs_api.get_knowledge_doc('core', 'intro', { signal: controller.signal, timeout: 2000 });
            expect(api_request).toHaveBeenCalledWith('/v1/docs/knowledge/core/intro', expect.objectContaining({
                signal: controller.signal,
                timeout: 2000
            }));
        });
    });

    describe('get_operations_manual', () => {
        it('should call api_request with correct url and headers', async () => {
            vi.mocked(api_request).mockResolvedValueOnce('# Ops Manual');

            const result = await docs_api.get_operations_manual();
            expect(result).toBe('# Ops Manual');
            expect(api_request).toHaveBeenCalledWith('/v1/docs/operations-manual', expect.objectContaining({
                method: 'GET',
                headers: { 'Accept': 'text/markdown' },
                response_type: 'text'
            }));
        });

        it('should propagate signal and timeout options', async () => {
            const controller = new AbortController();
            vi.mocked(api_request).mockResolvedValueOnce('# Ops Manual');

            await docs_api.get_operations_manual({ signal: controller.signal, timeout: 10000 });
            expect(api_request).toHaveBeenCalledWith('/v1/docs/operations-manual', expect.objectContaining({
                signal: controller.signal,
                timeout: 10000
            }));
        });
    });
});
