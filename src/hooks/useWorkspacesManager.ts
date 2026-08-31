/**
 * @docs ARCHITECTURE:UI-Hooks
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useWorkspacesManager
 * - **Primary Entrypoints**: `useWorkspacesManager`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useEffect } from 'react';
import { use_workspace_store } from '../stores/workspace_store';
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
    const fetch_agents = use_agent_store(state => state.fetch_agents);

    useEffect(() => {
        void fetch_agents();
        refresh_sync_status();
        
        const interval = setInterval(refresh_sync_status, 15000);
        return () => clearInterval(interval);
    }, [refresh_sync_status, fetch_agents]);

    const format_bytes = (bytes: number) => {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    };

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
