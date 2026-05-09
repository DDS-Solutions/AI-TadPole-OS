/**
 * @docs ARCHITECTURE:Services
 * @docs OPERATIONS_MANUAL:Agents
 * @docs API_REFERENCE:Endpoints
 * 
 * ### AI Assist Note
 * **Service Bridge**: Orchestrates agent data normalization and backend synchronization. 
 * Decouples mock data fallbacks from production API logic.
 * 
 * ### @aiContext
 * - **Dependencies**: `agent_api_service` (REST), `system_api_service` (Health), `use_workspace_store` (Pathing).
 * - **Side Effects**: State mutation in `agent_store` (via consumer of `load_agents`), Backend persistence via REST PUT.
 * - **Mocking**: Mock `system_api_service.check_health` to `true` to bypass mock data fallback in tests.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Backend offline (falls back to mocks), data normalization schema mismatch, or failing to persist updates to the Rust sidecar.
 * - **Failure Path**: Agent status "ghosting" (stuck in 'busy' without a task), or desync between local `agent_store` and global Rust registry.
 * - **Telemetry Link**: Search for `[AgentService]` or `update_agent_status` in UI logs.
 */


/**
 * Agent_Service
 * Dedicated service for agent data loading and management.
 * Extracts the loading logic from mock_agents.ts to create a proper
 * separation between data definitions and service behavior.
 * Refactored for strict snake_case compliance for backend parity.
 */

import { agents as mock_agents } from '../data/mock_agents';
import { agent_api_service } from './agent_api_service';
import { system_api_service } from './system_api_service';
import type { Agent, AgentPatch, AgentDto } from '../contracts/agent';
import { use_workspace_store } from '../stores/workspace_store';
import { normalize_agent_dto } from '../domain/agents/normalizers';
export type { AgentDto as Raw_Agent } from '../contracts/agent';

/**
 * load_agents
 * Loads agents from the Rust engine if available, falling back to mock data only if offline.
 */
export const load_agents = async (options: RequestInit = {}): Promise<Agent[]> => {
    let raw_agents: AgentDto[] = [];
    let is_backend_online = false;
    try {
        const is_connected = await system_api_service.check_health();

        if (is_connected) {
            is_backend_online = true;
            const live_agents = await agent_api_service.get_agents(options);
            if (live_agents.length > 0) {
                raw_agents = [...live_agents];
            }
        }
    } catch {
        // Offline mode fallback
    }

    if (!is_backend_online || raw_agents.length === 0) {
        console.warn('⚠️ [AgentService] No live agents detected. Registry may be empty or unreachable.');
    }

    const workspace_path_fn = use_workspace_store.getState().get_agent_path;
    return (raw_agents || []).map(raw => {
        const workspace_path = workspace_path_fn(raw.id);
        return normalize_agent_dto(raw, workspace_path);
    });
};

export const persist_agent_update = async (agent_id: string, updates: AgentPatch): Promise<void> => {
    try {
        await agent_api_service.update_agent(agent_id, updates);
    } catch (e) {
        console.error('⚠️ [AgentService] Backend sync failed:', e);
    }
};

export { normalize_agent_dto as normalize_agent };


/**
 * get_mock_agents
 * Returns the static mock agents synchronously (for initial render).
 */
export const get_mock_agents = (): Agent[] => mock_agents as unknown as Agent[];


// Metadata: [agent_service]

// Metadata: [agent_service]
