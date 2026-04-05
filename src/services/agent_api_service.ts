/**
 * @docs ARCHITECTURE:Services
 * @docs API_REFERENCE:Endpoints
 * 
 * ### AI Assist Note
 * **Agent Domain Service**: Dedicated interface for agent lifecycle management, task dispatching, and vector memory operations. 
 * Implements Maturity Level 3 HATEOAS envelopes for paginated agent lists and secure API key injection via `NeuralVault`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: 429 Rate Limit (governed by `model_store` RPM/TPM), Vault lock-out (prevents task dispatch), or memory fragmentation during semantic search.
 * - **Telemetry Link**: Look for `X-Request-Id` in backend logs or search `[AgentAPI]` in backend tracing.
 */

import type { Agent, Agent_Config, Task_Payload } from '../types/index';
import type { Tadpole_OS_Agent } from '../types/tadpoleos';
import { api_request } from './base_api_service';
import type { Skill_Definition, Workflow_Definition, Hook_Definition } from '../stores/skill_store';
import { PROVIDERS } from '../constants';
import { use_provider_store } from '../stores/provider_store';
import { use_vault_store } from '../stores/vault_store';
import { use_model_store, type Model_Entry } from '../stores/model_store';
import { event_bus } from './event_bus';

export interface Agent_Memory_Entry {
    id: string;
    text: string;
    mission_id: string;
    timestamp: number;
    metadata?: Record<string, unknown>;
}

interface Raw_Agent_Memory_Entry {
    id: string;
    text?: string;
    content?: string;
    mission_id?: string;
    timestamp?: number | string;
    metadata?: Record<string, unknown>;
}

function normalize_agent_memory_entry(entry: Raw_Agent_Memory_Entry): Agent_Memory_Entry {
    const raw_timestamp = entry.timestamp;
    let parsed_timestamp = Math.floor(Date.now() / 1000);
    if (typeof raw_timestamp === 'number') {
        parsed_timestamp = raw_timestamp;
    } else if (typeof raw_timestamp === 'string') {
        const numeric = Number(raw_timestamp);
        if (Number.isFinite(numeric)) {
            parsed_timestamp = numeric;
        } else {
            const iso_millis = Date.parse(raw_timestamp);
            if (Number.isFinite(iso_millis)) {
                parsed_timestamp = Math.floor(iso_millis / 1000);
            }
        }
    }

    return {
        id: entry.id,
        text: entry.text ?? entry.content ?? '',
        mission_id: entry.mission_id ?? 'manual',
        timestamp: Number.isFinite(parsed_timestamp) ? parsed_timestamp : Math.floor(Date.now() / 1000),
        metadata: entry.metadata,
    };
}

export const agent_api_service = {
    /**
     * Fetches agents from TadpoleOS. Supporting Maturity Level 3 HATEOAS envelopes.
     * Handles both direct arrays and object-wrapped 'data' envelopes to maintain
     * compatibility with paginated backend responses.
     */
    get_agents: async (): Promise<Tadpole_OS_Agent[]> => {
        type Agent_List_Envelope = { data?: Tadpole_OS_Agent[] } | Tadpole_OS_Agent[];
        const result = await api_request<Agent_List_Envelope>('/v1/agents?per_page=500', { method: 'GET' });

        // Maturity Level 3: Handle the 'data' field in the paginated envelope if present.
        if (result && typeof result === 'object' && !Array.isArray(result) && 'data' in result) {
            return result.data ?? [];
        }

        return Array.isArray(result) ? result : [];
    },

    /**
     * Updates an agent's configuration in the global Rust registry.
     */
    update_agent: async (agent_id: string, config: Agent_Config): Promise<boolean> => {
        const body: Record<string, unknown> = {};
        if (config.name !== undefined) body.name = config.name;
        if (config.role !== undefined) body.role = config.role;
        if (config.model_id !== undefined) body.model_id = config.model_id;
        if (config.provider !== undefined) body.provider = config.provider;
        if (config.temperature !== undefined) body.temperature = config.temperature;
        if (config.theme_color !== undefined) body.theme_color = config.theme_color;
        if (config.budget_usd !== undefined) body.budget_usd = config.budget_usd;
        if (config.requires_oversight !== undefined) body.requires_oversight = config.requires_oversight;
        if (config.skills !== undefined) body.skills = config.skills;
        if (config.workflows !== undefined) body.workflows = config.workflows;
        if (config.mcp_tools !== undefined) body.mcp_tools = config.mcp_tools;
        if (config.system_prompt !== undefined) body.system_prompt = config.system_prompt;
        if (config.api_key !== undefined) body.api_key = config.api_key;
        if (config.base_url !== undefined) body.base_url = config.base_url;
        if (config.active_model_slot !== undefined) body.active_model_slot = config.active_model_slot;
        if (config.connector_configs !== undefined) body.connector_configs = config.connector_configs;
        if (config.failure_count !== undefined) body.failure_count = config.failure_count;
        if (config.category !== undefined) body.category = config.category;
        if (config.created_at !== undefined) body.created_at = config.created_at;
        if (config.last_pulse !== undefined) body.last_pulse = config.last_pulse;
        if (config.current_task !== undefined) body.current_task = config.current_task;
        if (config.input_tokens !== undefined) body.input_tokens = config.input_tokens;
        if (config.output_tokens !== undefined) body.output_tokens = config.output_tokens;
        if (config.tokens_used !== undefined) body.tokens_used = config.tokens_used;

        if (config.model_config2 !== undefined) {
            body.model2 = config.model_config2.model_id;
            body.model_config2 = config.model_config2;
        }
        if (config.model_config3 !== undefined) {
            body.model3 = config.model_config3.model_id;
            body.model_config3 = config.model_config3;
        }

        await api_request(`/v1/agents/${agent_id}`, {
            method: 'PUT',
            body: JSON.stringify(body)
        });
        return true;
    },

    /**
     * Creates a new agent in the global Rust registry.
     */
    create_agent: async (agent: Agent): Promise<boolean> => {
        const body = {
            id: agent.id,
            name: agent.name,
            role: agent.role,
            department: agent.department,
            description: "New Agent Node",
            model: agent.model,
            model_config: agent.model_config,
            model2: agent.model2,
            model3: agent.model3,
            model_config2: agent.model_config2,
            model_config3: agent.model_config3,
            active_model_slot: agent.active_model_slot,
            status: agent.status || "idle",
            current_task: agent.current_task,
            tokens_used: agent.tokens_used ?? 0,
            token_usage: {
                input_tokens: agent.input_tokens ?? 0,
                output_tokens: agent.output_tokens ?? 0,
                total_tokens: (agent.input_tokens ?? 0) + (agent.output_tokens ?? 0),
            },
            metadata: { role: agent.role, department: agent.department },
            skills: agent.skills || [],
            workflows: agent.workflows || [],
            theme_color: agent.theme_color,
            budget_usd: agent.budget_usd || 0,
            requires_oversight: agent.requires_oversight || false,
            voice_id: agent.voice_id,
            voice_engine: agent.voice_engine,
            connector_configs: agent.connector_configs || [],
            mcp_tools: agent.mcp_tools || [],
            cost_usd: agent.cost_usd ?? 0,
            created_at: agent.created_at || new Date().toISOString(),
            category: agent.category || 'ai',
            input_tokens: agent.input_tokens ?? 0,
            output_tokens: agent.output_tokens ?? 0,
        };

        await api_request('/v1/agents', {
            method: 'POST',
            body: JSON.stringify(body)
        });
        return true;
    },

    /**
     * Pauses a running agent.
     */
    pause_agent: async (agent_id: string): Promise<boolean> => {
        await api_request(`/v1/agents/${agent_id}/pause`, { method: 'POST' });
        return true;
    },

    /**
     * Resumes a paused agent.
     */
    resume_agent: async (agent_id: string): Promise<boolean> => {
        await api_request(`/v1/agents/${agent_id}/resume`, { method: 'POST' });
        return true;
    },

    /**
     * Dispatches a command task to a specific agent node.
     * Integrates with NeuralVault for secure API key injection.
     * 
     * SECURITY NOTE: If a local key is available in the vault, it is injected into the payload.
     * The Rust backend is responsible for redacting this key from systemic logs.
     */
    send_command: async (agent_id: string, message: string, model_id: string, provider: string, cluster_id?: string, department?: string, budget_usd?: number, external_id?: string, safe_mode?: boolean, analysis?: boolean, request_id?: string): Promise<boolean> => {

        const vault_store = use_vault_store.getState();
        const model_store = use_model_store.getState();
        const body: Task_Payload = { message, cluster_id, department, provider, model_id, budget_usd, external_id, safe_mode, analysis };

        const provider_api_key = await vault_store.get_api_key(provider);
        const is_actually_locked = vault_store.is_locked && !sessionStorage.getItem('tadpole-vault-master-key');
        const is_local = provider === PROVIDERS.OLLAMA || provider === PROVIDERS.LOCAL;

        if (provider_api_key) {
            // NeuralVault Override: Use the local key if present for immediate inference.
            body.api_key = provider_api_key;
            const inventory_model = model_store.models.find((m: Model_Entry) => m.name === model_id);
            if (inventory_model) {
                // Attach rate limits (RPM/TPM) to the payload for backend governance
                if (inventory_model.rpm) body.rpm = inventory_model.rpm;
                if (inventory_model.tpm) body.tpm = inventory_model.tpm;
                if (inventory_model.rpd) body.rpd = inventory_model.rpd;
                if (inventory_model.tpd) body.tpd = inventory_model.tpd;
            }
        } else if (!is_local) {
            // Alert user if inference is attempted without a valid credentials link.
            const reason = is_actually_locked ? 'Vault is Locked' : `No Key for ${provider.toUpperCase()}`;
            event_bus.emit_log({
                source: 'System',
                text: `🔒 Neural Security: ${reason} for ${agent_id.toUpperCase()}.`,
                severity: 'warning'
            });
        }

        if (use_provider_store.getState().base_urls[provider]) {
            body.base_url = use_provider_store.getState().base_urls[provider];
        }

        await api_request(`/v1/agents/${agent_id}/tasks`, {
            method: 'POST',
            body: JSON.stringify(body),
            headers: request_id ? { 'X-Request-Id': request_id } : undefined
        });

        return true;
    },

    /**
     * Fetches the long-term vector memory for a given agent.
     */
    get_agent_memory: async (agent_id: string): Promise<{ status: string; entries: Agent_Memory_Entry[] }> => {
        const result = await api_request<{ status: string; entries: Raw_Agent_Memory_Entry[] }>(`/v1/agents/${agent_id}/memories`, { method: 'GET' });
        return {
            ...result,
            entries: (result.entries ?? []).map(normalize_agent_memory_entry),
        };
    },

    /**
     * Deletes a specific long-term vector memory row for a given agent.
     */
    delete_agent_memory: async (agent_id: string, row_id: string): Promise<{ status: string }> => {
        return api_request<{ status: string }>(`/v1/agents/${agent_id}/memories/${row_id}`, { method: 'DELETE' });
    },

    /**
     * Persists a new text entry into the agent's long-term vector memory.
     */
    save_agent_memory: async (agent_id: string, text: string): Promise<{ status: string; id: string }> => {
        return api_request<{ status: string; id: string }>(`/v1/agents/${agent_id}/memories`, {
            method: 'POST',
            body: JSON.stringify({ text })
        });
    },

    /**
     * Resets an agent's failure count and status.
     */
    reset_agent: async (agent_id: string): Promise<{ status: string; message: string }> => {
        return api_request<{ status: string; message: string }>(`/v1/agents/${agent_id}/reset`, {
            method: 'POST'
        });
    },

    /**
     * Imports a capability from a file. Returns a structured preview.
     */
    import_capability: async (file: File): Promise<{ type: string; data: Skill_Definition | Workflow_Definition | Hook_Definition; preview: string }> => {
        const form_data = new FormData();
        form_data.append('file', file);
        return api_request('/v1/skills/import', {
            method: 'POST',
            body: form_data,
        });
    },

    /**
     * Finalizes registration of a parsed capability.
     */
    register_capability: async (type: string, data: Skill_Definition | Workflow_Definition | Hook_Definition, category: string): Promise<{ status: string; name: string }> => {
        return api_request('/v1/skills/register', {
            method: 'POST',
            body: JSON.stringify({ type, data, category })
        });
    },

    /**
     * Performs a global semantic search across agent memories and mission logs.
     */
    search_memory: async (query: string, agent_id?: string): Promise<{ status: string; entries: Agent_Memory_Entry[] }> => {
        const url = new URL('/v1/search/memory', window.location.origin);
        url.searchParams.append('query', query);
        if (agent_id) url.searchParams.append('agent_id', agent_id);

        const result = await api_request<{ status: string; entries: Raw_Agent_Memory_Entry[] }>(url.pathname + url.search, {
            method: 'GET'
        });
        return {
            ...result,
            entries: (result.entries ?? []).map(normalize_agent_memory_entry),
        };
    }
};

