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
            const mock_files = { files: ['file1.ts', 'file2.ts'], total: 2, truncated: false };
            vi.mocked(api_request).mockResolvedValueOnce(mock_files);

            const result = await workspace_api.get_workspace_files();
            expect(result).toEqual(mock_files);
            expect(api_request).toHaveBeenCalledWith('/v1/system/workspaces/files', expect.objectContaining({
                method: 'GET'
            }));
        });

        it('propagates abort signal and timeout option', async () => {
            const controller = new AbortController();
            vi.mocked(api_request).mockResolvedValueOnce({ files: [], total: 0, truncated: false });

            await workspace_api.get_workspace_files({ signal: controller.signal, timeout: 10000 });
            expect(api_request).toHaveBeenCalledWith('/v1/system/workspaces/files', expect.objectContaining({
                signal: controller.signal,
                timeout: 10000
            }));
        });
    });

    describe('get_file_history', () => {
        it('calls GET /v1/cas/history with query params', async () => {
            const mock_history = [{
                id: 1,
                workspace_id: 'ws-1',
                file_path: 'test.rs',
                hash: 'abc123hash',
                size_bytes: 42,
                version_num: 1,
                created_at: '2026-10-06T12:00:00Z'
            }];
            vi.mocked(api_request).mockResolvedValueOnce({ success: true, data: mock_history });

            const res = await workspace_api.get_file_history('test.rs', '/root');
            expect(res).toEqual(mock_history);
            expect(api_request).toHaveBeenCalledWith('/v1/cas/history?file_path=test.rs&workspace_root=%2Froot', expect.objectContaining({
                method: 'GET'
            }));
        });
    });

    describe('restore_file_version', () => {
        it('calls POST /v1/cas/restore with payload', async () => {
            const mock_revision = {
                id: 1,
                workspace_id: 'ws-1',
                file_path: 'test.rs',
                hash: 'abc123hash',
                size_bytes: 42,
                version_num: 1,
                created_at: '2026-10-06T12:00:00Z'
            };
            vi.mocked(api_request).mockResolvedValueOnce({ success: true, data: mock_revision });

            const res = await workspace_api.restore_file_version('test.rs', 1, '/root');
            expect(res).toEqual(mock_revision);
            expect(api_request).toHaveBeenCalledWith('/v1/cas/restore', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({ file_path: 'test.rs', version_num: 1, workspace_root: '/root' })
            }));
        });
    });
});
