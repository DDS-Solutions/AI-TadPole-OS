/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[workspace_api]` in observability traces.
 */

import { api_request } from '../base_api_service';
import type { Workspace_Status } from '../system_api_types';

export const workspace_api = {
    get_workspaces_status: async (): Promise<Workspace_Status[]> => {
        return api_request<Workspace_Status[]>('/v1/system/workspaces/status', { method: 'GET' });
    },

    get_workspace_files: async (): Promise<string[]> => {
        return api_request<string[]>('/v1/system/workspaces/files', { method: 'GET' });
    }
};

// Metadata: [workspace_api]
