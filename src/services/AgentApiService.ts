import type { Agent, AgentConfig, TaskPayload } from '../types/index';
import type { TadpoleOSAgent } from '../types/tadpoleos';
import { apiRequest } from './BaseApiService';
import { PROVIDERS } from '../constants';
import { useProviderStore } from '../stores/providerStore';
import { useVaultStore } from '../stores/vaultStore';
import { useModelStore, type ModelEntry } from '../stores/modelStore';
import { EventBus } from './eventBus';

export const AgentApiService = {
    /**
     * Fetches agents from TadpoleOS. Supporting Maturity Level 3 HATEOAS envelopes.
     */
    getAgents: async (): Promise<TadpoleOSAgent[]> => {
        type AgentListEnvelope = { data?: TadpoleOSAgent[] } | TadpoleOSAgent[];
        const result = await apiRequest<AgentListEnvelope>('/v1/agents?per_page=500', { method: 'GET' });

        // Maturity Level 3: Handle the 'data' field in the paginated envelope
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
        if (config.skills !== undefined) body.skills = config.skills;
        if (config.workflows !== undefined) body.workflows = config.workflows;
        if (config.systemPrompt !== undefined) body.systemPrompt = config.systemPrompt;
        if (config.apiKey !== undefined) body.apiKey = config.apiKey;
        if (config.baseUrl !== undefined) body.baseUrl = config.baseUrl;
        if (config.activeModelSlot !== undefined) body.activeModelSlot = config.activeModelSlot;

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
            costUsd: 0
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
     */
    sendCommand: async (agentId: string, message: string, modelId: string, provider: string, clusterId?: string, department?: string, budgetUsd?: number, externalId?: string, safeMode?: boolean, analysis?: boolean, requestId?: string): Promise<boolean> => {

        const vaultStore = useVaultStore.getState();
        const modelStore = useModelStore.getState();
        const providerStore = useProviderStore.getState();
        const body: TaskPayload = { message, clusterId, department, provider, modelId, budgetUsd, externalId, safeMode, analysis };

        const providerApiKey = await vaultStore.getApiKey(provider);
        const isLocal = provider === PROVIDERS.OLLAMA || provider === PROVIDERS.LOCAL;

        if (providerApiKey) {
            // NeuralVault Override: Use the local key if present, but the backend will redact it from logs.
            body.apiKey = providerApiKey;
            const inventoryModel = modelStore.models.find((m: ModelEntry) => m.name === modelId);
            if (inventoryModel) {
                if (inventoryModel.rpm) body.rpm = inventoryModel.rpm;
                if (inventoryModel.tpm) body.tpm = inventoryModel.tpm;
                if (inventoryModel.rpd) body.rpd = inventoryModel.rpd;
                if (inventoryModel.tpd) body.tpd = inventoryModel.tpd;
            }
        } else if (!isLocal) {
            const reason = vaultStore.isLocked ? 'Vault is Locked' : 'Using Server Environment';
            EventBus.emit({
                source: 'System',
                text: `🔒 Neural Security: ${reason} for ${provider.toUpperCase()}.`,
                severity: 'info'
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
    getAgentMemory: async (agentId: string): Promise<{ status: string; entries: unknown[] }> => {
        return apiRequest<{ status: string; entries: unknown[] }>(`/v1/agents/${agentId}/memories`, { method: 'GET' });
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
    }
};
