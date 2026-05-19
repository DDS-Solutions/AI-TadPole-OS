/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[useWorkspacesManager]` in observability traces.
 */

import { useEffect, useCallback } from 'react';
import { use_workspace_store } from '../stores/workspace_store';
import { load_agents } from '../services/agent_service';
import { use_agent_store } from '../stores/agent_store';

export function useWorkspacesManager() {
    const { 
        clusters, 
        approve_branch, 
        reject_branch, 
        sync_status, 
        refresh_sync_status 
    } = use_workspace_store();
    
    const agents = use_agent_store(state => state.agents);

    useEffect(() => {
        void load_agents();
        refresh_sync_status();
        
        const interval = setInterval(refresh_sync_status, 15000);
        return () => clearInterval(interval);
    }, [refresh_sync_status]);

    const format_bytes = useCallback((bytes: number) => {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    }, []);

    return {
        clusters,
        agents,
        sync_status,
        approve_branch,
        reject_branch,
        refresh_sync_status,
        format_bytes
    };
}

// Metadata: [useWorkspacesManager]
