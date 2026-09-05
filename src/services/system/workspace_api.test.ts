/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / workspace_api.test
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
import { workspace_api } from './workspace_api';
import { api_request } from '../base_api_service';

vi.mock('../base_api_service', () => ({
    api_request: vi.fn()
}));

describe('workspace_api', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('get_workspaces_status', () => {
        it('calls GET /v1/system/workspaces/status', async () => {
            const mock_status = [{ id: 'w-1', agent_id: 'a1', source_type: 'local', source_uri: 'file:///t', status: 'synced', last_sync_at: '2026-06-07T12:00:00Z', file_count: 12, total_bytes: 45000 }];
            vi.mocked(api_request).mockResolvedValueOnce(mock_status);

            const result = await workspace_api.get_workspaces_status();
            expect(result).toEqual(mock_status);
            expect(api_request).toHaveBeenCalledWith('/v1/system/workspaces/status', expect.objectContaining({
                method: 'GET'
            }));
        });

        it('propagates abort signal and timeout option', async () => {
            const controller = new AbortController();
            vi.mocked(api_request).mockResolvedValueOnce([]);

            await workspace_api.get_workspaces_status({ signal: controller.signal, timeout: 5000 });
            expect(api_request).toHaveBeenCalledWith('/v1/system/workspaces/status', expect.objectContaining({
                signal: controller.signal,
                timeout: 5000
            }));
        });
    });

    describe('get_workspace_files', () => {
        it('calls GET /v1/system/workspaces/files', async () => {
            const mock_files = ['file1.ts', 'file2.ts'];
            vi.mocked(api_request).mockResolvedValueOnce(mock_files);

            const result = await workspace_api.get_workspace_files();
            expect(result).toEqual(mock_files);
            expect(api_request).toHaveBeenCalledWith('/v1/system/workspaces/files', expect.objectContaining({
                method: 'GET'
            }));
        });

        it('propagates abort signal and timeout option', async () => {
            const controller = new AbortController();
            vi.mocked(api_request).mockResolvedValueOnce([]);

            await workspace_api.get_workspace_files({ signal: controller.signal, timeout: 10000 });
            expect(api_request).toHaveBeenCalledWith('/v1/system/workspaces/files', expect.objectContaining({
                signal: controller.signal,
                timeout: 10000
            }));
        });
    });
});
