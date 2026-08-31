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
 * - **Witness Tests**: none declared
 */

import { api_request } from '../base_api_service';
import type { Workspace_Status, RequestOptions } from '../system_api_types';

export const workspace_api = {
    get_workspaces_status: async (options?: RequestOptions): Promise<Workspace_Status[]> => {
        return api_request<Workspace_Status[]>('/v1/system/workspaces/status', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    get_workspace_files: async (options?: RequestOptions): Promise<string[]> => {
        return api_request<string[]>('/v1/system/workspaces/files', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    }
};
