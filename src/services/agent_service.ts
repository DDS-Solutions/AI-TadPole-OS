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
import type { Agent, Agent_Status } from '../types';
import { use_workspace_store } from '../stores/workspace_store';
import type { Tadpole_OS_Model_Config } from '../types/tadpoleos';

export type Agent_Override = Partial<Agent> & {
    provider?: string;
    provider2?: string;
    provider3?: string;
    skills?: string[];
    workflows?: string[];
    mcp_tools?: string[];
};

export interface Raw_Agent {
    id: string;
    name: string;
    role?: string;
    department?: string;
    status?: string | Agent_Status;
    model?: string;
    model_config?: Tadpole_OS_Model_Config;
    workspace?: string;
    skills?: string[];
    workflows?: string[];
    theme_color?: string;
    budget_usd?: number;
    cost_usd?: number;
    tokens_used?: number;
    model2?: string;
    model3?: string;
    model_config2?: Tadpole_OS_Model_Config;
    model_config3?: Tadpole_OS_Model_Config;
    active_model_slot?: number;
    metadata?: {
        role?: string;
        department?: string;
    };
    mcp_tools?: string[];
    failure_count?: number;
    last_failure_at?: string;
    category?: string;
    created_at?: string;
    token_usage?: {
        input_tokens?: number;
        output_tokens?: number;
        total_tokens?: number;
    };
    input_tokens?: number;
    output_tokens?: number;
    total_tokens?: number;
}

/**
 * normalize_agent
 * Normalizes a raw agent object from the backend (or WebSocket) into the frontend Agent type.
 * Ensures schema parity.
 */
export const normalize_agent = (raw: Raw_Agent, workspace_path?: string, existing_agent?: Agent): Agent => {
    const raw_dept = raw.department || raw.metadata?.department || 'Operations';
    const dept = raw_dept === 'QA' ? 'Quality Assurance' : raw_dept;

    const status = (raw.status === 'working' ? 'active' : raw.status) || 'idle';

    return {
        id: String(raw.id),
        name: raw.name || 'Unnamed Agent',
        role: raw.role || raw.metadata?.role || 'AI Agent',
        department: dept as Agent['department'],
        status: status as Agent_Status,
        tokens_used: raw.tokens_used || 0,
        model: (typeof raw.model === 'string' ? raw.model : raw.model_config?.model_id) || 'Unknown',
        model_config: raw.model_config,
        workspace_path: workspace_path || raw.workspace,
        current_task: (raw.status === 'active' || raw.status === 'working') ? (existing_agent?.current_task) : undefined,
        skills: raw.skills || [],
        workflows: raw.workflows || [],
        mcp_tools: raw.mcp_tools || [],
        theme_color: raw.theme_color,
        budget_usd: raw.budget_usd || 0,
        cost_usd: raw.cost_usd || 0,
        model2: raw.model2,
        model3: raw.model3,
        model_config2: raw.model_config2,
        model_config3: raw.model_config3,
        active_model_slot: raw.active_model_slot as 1 | 2 | 3,
        failure_count: raw.failure_count ?? 0,
        last_failure_at: raw.last_failure_at,
        created_at: raw.created_at,
        input_tokens: raw.token_usage?.input_tokens ?? raw.input_tokens,
        output_tokens: raw.token_usage?.output_tokens ?? raw.output_tokens,
        category: raw.category || 'user',
    };
};

/**
 * load_agents
 * Loads agents from the Rust engine if available, falling back to mock data only if offline.
 */
export const load_agents = async (): Promise<Agent[]> => {
    let raw_agents: Raw_Agent[] = [];
    let is_backend_online = false;
    try {
        const is_connected = await system_api_service.check_health();

        if (is_connected) {
            is_backend_online = true;
            const live_agents = await agent_api_service.get_agents();
            if (live_agents.length > 0) {
                raw_agents = [...live_agents as unknown as Raw_Agent[]];
            }
        }
    } catch {
        // Offline mode fallback
    }

    if (!is_backend_online || raw_agents.length === 0) {
        console.warn('⚠️ [AgentService] No live agents detected. Registry may be empty or unreachable.');
    }

    const workspace_path_fn = use_workspace_store.getState().get_agent_path;
    return raw_agents.map(raw => {
        const workspace_path = workspace_path_fn(raw.id);
        return normalize_agent(raw, workspace_path);
    });
};

/**
 * persist_agent_update
 * Persists an agent update to the global Rust agent registry.
 */
export const persist_agent_update = async (agent_id: string, updates: Agent_Override) => {
    try {
        const config: Record<string, unknown> = {};
        if (updates.role !== undefined) config.role = updates.role;
        if (updates.name !== undefined) config.name = updates.name;
        if (updates.model !== undefined) {
            config.model_id = updates.model;
            if (updates.model_config?.provider) {
                config.provider = updates.model_config.provider;
            } else {
                const m = updates.model.toLowerCase();
                if (m.includes('gpt')) config.provider = 'openai';
                else if (m.includes('claude')) config.provider = 'anthropic';
                else if (m.includes('gemini')) config.provider = 'google';
                else if (m.includes('llama') || m.includes('mixtral')) config.provider = 'groq';
            }
        }
        if (updates.model_config !== undefined) {
            config.model_id = updates.model_config.model_id;
            config.provider = updates.model_config.provider;
            config.temperature = updates.model_config.temperature;
            config.system_prompt = updates.model_config.system_prompt;
            config.api_key = updates.model_config.api_key;
            config.base_url = updates.model_config.base_url;
        }
        if (updates.theme_color !== undefined) config.theme_color = updates.theme_color;
        if (updates.active_model_slot !== undefined) config.active_model_slot = updates.active_model_slot;
        if (updates.model_config2 !== undefined) config.model_config2 = updates.model_config2;
        if (updates.model_config3 !== undefined) config.model_config3 = updates.model_config3;
        if (updates.budget_usd !== undefined) config.budget_usd = updates.budget_usd;
        if (updates.skills !== undefined) config.skills = updates.skills;
        if (updates.workflows !== undefined) config.workflows = updates.workflows;
        if (updates.mcp_tools !== undefined) config.mcp_tools = updates.mcp_tools;
        if (updates.category !== undefined) config.category = updates.category;
        if (updates.created_at !== undefined) config.created_at = updates.created_at;
        if (updates.input_tokens !== undefined) config.input_tokens = updates.input_tokens;
        if (updates.output_tokens !== undefined) config.output_tokens = updates.output_tokens;
        if (updates.tokens_used !== undefined) config.tokens_used = updates.tokens_used;
        if (updates.failure_count !== undefined) config.failure_count = updates.failure_count;

        if (Object.keys(config).length > 0) {
            await agent_api_service.update_agent(agent_id, config);
        }
    } catch (e) {
        console.error('⚠️ [AgentService] Backend sync failed:', e);
    }
};

/**
 * get_mock_agents
 * Returns the static mock agents synchronously (for initial render).
 */
export const get_mock_agents = (): Agent[] => mock_agents as unknown as Agent[];
