/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / workspace_api
 * - **Primary Entrypoints**: `workspace_api`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { api_request } from '../base_api_service';
import type { Workspace_Status, RequestOptions, WorkspaceFilesResponse, RevisionSummary } from '../system_api_types';

export type { WorkspaceFilesResponse, RevisionSummary };

export const workspace_api = {
    get_workspaces_status: async (options?: RequestOptions): Promise<Workspace_Status[]> => {
        return api_request<Workspace_Status[]>('/v1/system/workspaces/status', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    get_workspace_files: async (options?: RequestOptions): Promise<WorkspaceFilesResponse> => {
        return api_request<WorkspaceFilesResponse>('/v1/system/workspaces/files', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    get_file_history: async (filePath: string, workspaceRoot?: string, options?: RequestOptions): Promise<RevisionSummary[]> => {
        const params = new URLSearchParams({ file_path: filePath });
        if (workspaceRoot) params.append('workspace_root', workspaceRoot);
        const res = await api_request<{ success: boolean; data: RevisionSummary[] } | RevisionSummary[]>(`/v1/cas/history?${params.toString()}`, {
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
        if (!res) return [];
        return Array.isArray(res) ? res : (res.data || []);
    },

    restore_file_version: async (filePath: string, versionNum: number, workspaceRoot?: string, options?: RequestOptions): Promise<RevisionSummary> => {
        const body: Record<string, unknown> = {
            file_path: filePath,
            version_num: versionNum,
        };
        if (workspaceRoot) body.workspace_root = workspaceRoot;
        const res = await api_request<{ success: boolean; data: RevisionSummary } | RevisionSummary>('/v1/cas/restore', {
            method: 'POST',
            body: JSON.stringify(body),
            signal: options?.signal,
            timeout: options?.timeout
        });
        return res && 'data' in res ? res.data : res;
    }
};
