import type { Agent, AgentConfig, TaskPayload } from '../types/index';
import type { TadpoleOSAgent } from '../types/tadpoleos';
import { apiRequest } from './BaseApiService';
import type { SkillDefinition, WorkflowDefinition, HookDefinition } from '../stores/skillStore';
import { PROVIDERS } from '../constants';
import { useProviderStore } from '../stores/providerStore';
import { useVaultStore } from '../stores/vaultStore';
import { useModelStore, type ModelEntry } from '../stores/modelStore';
import { EventBus } from './eventBus';

export interface AgentMemoryEntry {
    id: string;
    text: string;
    mission_id: string;
    timestamp: number;
    metadata?: Record<string, unknown>;
}

interface RawAgentMemoryEntry {
    id: string;
    text?: string;
    content?: string;
    mission_id?: string;
    timestamp?: number | string;
    metadata?: Record<string, unknown>;
}

function normalizeAgentMemoryEntry(entry: RawAgentMemoryEntry): AgentMemoryEntry {
    const rawTimestamp = entry.timestamp;
    let parsedTimestamp = Math.floor(Date.now() / 1000);
    if (typeof rawTimestamp === 'number') {
        parsedTimestamp = rawTimestamp;
    } else if (typeof rawTimestamp === 'string') {
        const numeric = Number(rawTimestamp);
        if (Number.isFinite(numeric)) {
            parsedTimestamp = numeric;
        } else {
            const isoMillis = Date.parse(rawTimestamp);
            if (Number.isFinite(isoMillis)) {
                parsedTimestamp = Math.floor(isoMillis / 1000);
            }
        }
    }

    return {
        id: entry.id,
        text: entry.text ?? entry.content ?? '',
        mission_id: entry.mission_id ?? 'manual',
        timestamp: Number.isFinite(parsedTimestamp) ? parsedTimestamp : Math.floor(Date.now() / 1000),
        metadata: entry.metadata,
    };
}

export const AgentApiService = {
    /**
     * Fetches agents from TadpoleOS. Supporting Maturity Level 3 HATEOAS envelopes.
     * Handles both direct arrays and object-wrapped 'data' envelopes to maintain
     * compatibility with paginated backend responses.
     */
    getAgents: async (): Promise<TadpoleOSAgent[]> => {
        type AgentListEnvelope = { data?: TadpoleOSAgent[] } | TadpoleOSAgent[];
        const result = await apiRequest<AgentListEnvelope>('/v1/agents?per_page=500', { method: 'GET' });

        // Maturity Level 3: Handle the 'data' field in the paginated envelope if present.
        if (result && typeof result === 'object' && !Array.isArray(result) && 'data' in result) {
            return result.data ?? [];
        }

        return Array.isArray(result) ? result : [];
    },

    /**
     * Updates an agent's configuration in the global Rust registry.
     */
    updateAgent: async (agentId: string, config: AgentConfig): Promise<boolean> => {
        const body: Record<string, unknown> = {};
        if (config.name !== undefined) body.name = config.name;
        if (config.role !== undefined) body.role = config.role;
        if (config.modelId !== undefined) body.modelId = config.modelId;
        if (config.provider !== undefined) body.provider = config.provider;
        if (config.temperature !== undefined) body.temperature = config.temperature;
        if (config.themeColor !== undefined) body.themeColor = config.themeColor;
        if (config.budgetUsd !== undefined) body.budgetUsd = config.budgetUsd;
        if (config.requiresOversight !== undefined) body.requiresOversight = config.requiresOversight;
        if (config.skills !== undefined) body.skills = config.skills;
        if (config.workflows !== undefined) body.workflows = config.workflows;
        if (config.mcpTools !== undefined) body.mcpTools = config.mcpTools;
        if (config.systemPrompt !== undefined) body.systemPrompt = config.systemPrompt;
        if (config.apiKey !== undefined) body.apiKey = config.apiKey;
        if (config.baseUrl !== undefined) body.baseUrl = config.baseUrl;
        if (config.activeModelSlot !== undefined) body.activeModelSlot = config.activeModelSlot;
        if (config.connectorConfigs !== undefined) body.connectorConfigs = config.connectorConfigs;
        if (config.failureCount !== undefined) body.failureCount = config.failureCount;
        if (config.category !== undefined) body.category = config.category;
        if (config.createdAt !== undefined) body.createdAt = config.createdAt;
        if (config.lastPulse !== undefined) body.lastPulse = config.lastPulse;
        if (config.inputTokens !== undefined) body.inputTokens = config.inputTokens;
        if (config.outputTokens !== undefined) body.outputTokens = config.outputTokens;

        if (config.modelConfig2 !== undefined) {
            body.model2 = config.modelConfig2.modelId;
            body.modelConfig2 = config.modelConfig2;
        }
        if (config.modelConfig3 !== undefined) {
            body.model3 = config.modelConfig3.modelId;
            body.modelConfig3 = config.modelConfig3;
        }

        await apiRequest(`/v1/agents/${agentId}`, {
            method: 'PUT',
            body: JSON.stringify(body)
        });
        return true;
    },

    /**
     * Creates a new agent in the global Rust registry.
     */
    createAgent: async (agent: Agent): Promise<boolean> => {
        const body = {
            id: agent.id,
            name: agent.name,
            role: agent.role,
            department: agent.department,
            description: "New Agent Node",
            model: agent.model,
            modelConfig: agent.modelConfig,
            model2: agent.model2,
            model3: agent.model3,
            modelConfig2: agent.modelConfig2,
            modelConfig3: agent.modelConfig3,
            status: "idle",
            tokensUsed: 0,
            tokenUsage: { inputTokens: 0, outputTokens: 0, totalTokens: 0 },
            metadata: { role: agent.role, department: agent.department },
            themeColor: agent.themeColor,
            budgetUsd: agent.budgetUsd || 0,
            requiresOversight: agent.requiresOversight || false,
            connectorConfigs: agent.connectorConfigs || [],
            mcpTools: agent.mcpTools || [],
            costUsd: 0,
            createdAt: agent.createdAt || new Date().toISOString(),
            category: agent.category || 'ai',
            inputTokens: 0,
            outputTokens: 0,
        };

        await apiRequest('/v1/agents', {
            method: 'POST',
            body: JSON.stringify(body)
        });
        return true;
    },

    /**
     * Pauses a running agent.
     */
    pauseAgent: async (agentId: string): Promise<boolean> => {
        await apiRequest(`/v1/agents/${agentId}/pause`, { method: 'POST' });
        return true;
    },

    /**
     * Resumes a paused agent.
     */
    resumeAgent: async (agentId: string): Promise<boolean> => {
        await apiRequest(`/v1/agents/${agentId}/resume`, { method: 'POST' });
        return true;
    },


    /**
     * Dispatches a command task to a specific agent node.
     * Integrates with NeuralVault for secure API key injection.
     * 
     * SECURITY NOTE: If a local key is available in the vault, it is injected into the payload.
     * The Rust backend is responsible for redacting this key from systemic logs.
     */
    sendCommand: async (agentId: string, message: string, modelId: string, provider: string, clusterId?: string, department?: string, budgetUsd?: number, externalId?: string, safeMode?: boolean, analysis?: boolean, requestId?: string): Promise<boolean> => {

        const vaultStore = useVaultStore.getState();
        const modelStore = useModelStore.getState();
        const providerStore = useProviderStore.getState();
        const body: TaskPayload = { message, clusterId, department, provider, modelId, budgetUsd, externalId, safeMode, analysis };

        const providerApiKey = await vaultStore.getApiKey(provider);
        const isActuallyLocked = vaultStore.isLocked && !sessionStorage.getItem('tadpole-vault-master-key');
        const isLocal = provider === PROVIDERS.OLLAMA || provider === PROVIDERS.LOCAL;

        if (providerApiKey) {
            // NeuralVault Override: Use the local key if present for immediate inference.
            body.apiKey = providerApiKey;
            const inventoryModel = modelStore.models.find((m: ModelEntry) => m.name === modelId);
            if (inventoryModel) {
                // Attach rate limits (RPM/TPM) to the payload for backend governance
                if (inventoryModel.rpm) body.rpm = inventoryModel.rpm;
                if (inventoryModel.tpm) body.tpm = inventoryModel.tpm;
                if (inventoryModel.rpd) body.rpd = inventoryModel.rpd;
                if (inventoryModel.tpd) body.tpd = inventoryModel.tpd;
            }
        } else if (!isLocal) {
            // Alert user if inference is attempted without a valid credentials link.
            const reason = isActuallyLocked ? 'Vault is Locked' : `No Key for ${provider.toUpperCase()}`;
            EventBus.emit({
                source: 'System',
                text: `🔒 Neural Security: ${reason} for ${agentId.toUpperCase()}.`,
                severity: 'warning'
            });
        }

        if (providerStore.baseUrls[provider]) {
            body.baseUrl = providerStore.baseUrls[provider];
        }

        await apiRequest(`/v1/agents/${agentId}/tasks`, {
            method: 'POST',
            body: JSON.stringify(body),
            headers: requestId ? { 'X-Request-Id': requestId } : undefined
        });

        return true;
    },

    /**
     * Fetches the long-term vector memory for a given agent.
     */
    getAgentMemory: async (agentId: string): Promise<{ status: string; entries: AgentMemoryEntry[] }> => {
        const result = await apiRequest<{ status: string; entries: RawAgentMemoryEntry[] }>(`/v1/agents/${agentId}/memories`, { method: 'GET' });
        return {
            ...result,
            entries: (result.entries ?? []).map(normalizeAgentMemoryEntry),
        };
    },

    /**
     * Deletes a specific long-term vector memory row for a given agent.
     */
    deleteAgentMemory: async (agentId: string, rowId: string): Promise<{ status: string }> => {
        return apiRequest<{ status: string }>(`/v1/agents/${agentId}/memories/${rowId}`, { method: 'DELETE' });
    },

    /**
     * Persists a new text entry into the agent's long-term vector memory.
     */
    saveAgentMemory: async (agentId: string, text: string): Promise<{ status: string; id: string }> => {
        return apiRequest<{ status: string; id: string }>(`/v1/agents/${agentId}/memories`, {
            method: 'POST',
            body: JSON.stringify({ text })
        });
    },

    /**
     * Resets an agent's failure count and status.
     */
    resetAgent: async (agentId: string): Promise<{ status: string; message: string }> => {
        return apiRequest<{ status: string; message: string }>(`/v1/agents/${agentId}/reset`, {
            method: 'POST'
        });
    },

    /**
     * Imports a capability from a file. Returns a structured preview.
     */
    importCapability: async (file: File): Promise<{ type: string; data: SkillDefinition | WorkflowDefinition | HookDefinition; preview: string }> => {
        const formData = new FormData();
        formData.append('file', file);
        return apiRequest('/v1/skills/import', {
            method: 'POST',
            body: formData,
        });
    },

    /**
     * Finalizes registration of a parsed capability.
     */
    registerCapability: async (type: string, data: SkillDefinition | WorkflowDefinition | HookDefinition, category: string): Promise<{ status: string; name: string }> => {
        return apiRequest('/v1/skills/register', {
            method: 'POST',
            body: JSON.stringify({ type, data, category })
        });
    },

    /**
     * Performs a global semantic search across agent memories and mission logs.
     */
    searchMemory: async (query: string, agentId?: string): Promise<{ status: string; entries: AgentMemoryEntry[] }> => {
        const url = new URL('/v1/search/memory', window.location.origin);
        url.searchParams.append('query', query);
        if (agentId) url.searchParams.append('agent_id', agentId);

        const result = await apiRequest<{ status: string; entries: RawAgentMemoryEntry[] }>(url.pathname + url.search, {
            method: 'GET'
        });
        return {
            ...result,
            entries: (result.entries ?? []).map(normalizeAgentMemoryEntry),
        };
    }
};
