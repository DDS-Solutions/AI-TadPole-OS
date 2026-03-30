/**
 * @module agentService
 * Dedicated service for agent data loading and management.
 * Extracts the loading logic from mockAgents.ts to create a proper
 * separation between data definitions and service behavior.
 */

import { agents as mockAgents } from '../data/mockAgents';
import { AgentApiService } from './AgentApiService';
import { SystemApiService } from './SystemApiService';
import type { Agent, AgentStatus } from '../types';
import { useWorkspaceStore } from '../stores/workspaceStore';
import type { TadpoleOSModelConfig } from '../types/tadpoleos';

// Local storage key for agent overrides is DEPRECATED as of v1.1.0 in favor of server-side parity.
// const STORAGE_KEY = 'tadpole-agent-overrides';

export type AgentOverride = Partial<Agent> & {
    provider?: string;
    provider2?: string;
    provider3?: string;
    skills?: string[];
    workflows?: string[];
    mcpTools?: string[];
};

export interface RawAgent {
    id: string;
    name: string;
    role?: string;
    department?: string;
    status?: string | AgentStatus;
    model?: string;
    modelConfig?: TadpoleOSModelConfig;
    workspace?: string;
    skills?: string[];
    workflows?: string[];
    themeColor?: string;
    budgetUsd?: number;
    costUsd?: number;
    tokensUsed?: number;
    tokens_used?: number; // Legacy snake_case fallback
    model2?: string;
    model3?: string;
    modelConfig2?: TadpoleOSModelConfig;
    modelConfig3?: TadpoleOSModelConfig;
    activeModelSlot?: number;
    metadata?: {
        role?: string;
        department?: string;
    };
    mcp_tools?: string[];
    mcpTools?: string[];
    failureCount?: number;
    failure_count?: number;
    lastFailureAt?: string;
    last_failure_at?: string;
    category?: string;
    created_at?: string;
    createdAt?: string;
    tokenUsage?: {
        inputTokens?: number;
        outputTokens?: number;
        totalTokens?: number;
    };
    inputTokens?: number;
    outputTokens?: number;
    totalTokens?: number;
}

/**
 * Normalizes a raw agent object from the backend (or WebSocket) into the frontend Agent type.
 * Ensures schema parity.
 */
export const normalizeAgent = (raw: RawAgent, workspacePath?: string, existingAgent?: Agent): Agent => {
    const rawDept = raw.department || raw.metadata?.department || 'Operations';
    const dept = rawDept === 'QA' ? 'Quality Assurance' : rawDept;

    const status = (raw.status === 'working' ? 'active' : raw.status) || 'idle';

    return {
        id: String(raw.id),
        name: raw.name || 'Unnamed Agent',
        role: raw.role || raw.metadata?.role || 'AI Agent',
        department: dept as Agent['department'],
        status: status as AgentStatus,
        tokensUsed: raw.tokensUsed || raw.tokens_used || 0,
        // Match the logic from TadpoleOSService transformation
        model: (typeof raw.model === 'string' ? raw.model : raw.modelConfig?.modelId) || 'Unknown',
        modelConfig: raw.modelConfig,
        workspacePath: workspacePath || raw.workspace,
        currentTask: raw.status === 'active' || raw.status === 'working' ? (existingAgent?.currentTask) : undefined,
        skills: raw.skills || [],
        workflows: raw.workflows || [],
        mcpTools: raw.mcpTools || raw.mcp_tools || [],
        themeColor: raw.themeColor,
        budgetUsd: raw.budgetUsd || 0,
        costUsd: raw.costUsd || 0,
        model2: raw.model2,
        model3: raw.model3,
        modelConfig2: raw.modelConfig2,
        modelConfig3: raw.modelConfig3,
        activeModelSlot: raw.activeModelSlot as 1 | 2 | 3,
        failureCount: raw.failureCount ?? raw.failure_count ?? 0,
        lastFailureAt: raw.lastFailureAt ?? raw.last_failure_at,
        createdAt: raw.createdAt ?? raw.created_at,
        inputTokens: raw.tokenUsage?.inputTokens,
        outputTokens: raw.tokenUsage?.outputTokens,
        category: raw.category || 'user',
    };
};

/**
 * Loads agents from the Rust engine if available, falling back to mock data only if offline.
 * Trusting the server is the key to cross-device parity.
 */
export const loadAgents = async (): Promise<Agent[]> => {
    let rawAgents: RawAgent[] = [];
    let isBackendOnline = false;
    try {
        const isConnected = await SystemApiService.checkHealth();

        if (isConnected) {
            isBackendOnline = true;
            const liveAgents = await AgentApiService.getAgents();
            if (liveAgents.length > 0) {
                rawAgents = [...liveAgents];
            }
        }
    } catch {
        // Offline mode — fall back to local mock agents silently
    }

    if (isBackendOnline && rawAgents.length > 0) {
        // Only use agents from the backend. No backfilling with mocks.
        // This ensures that what you see is what is actually in the DB.
    } else if (rawAgents.length === 0) {
        // If no agents are found, return an empty list or show a warning.
        // Mocks are now restricted to tests to avoid persistence confusion.
        console.warn('⚠️ [AgentService] No live agents detected. Database may be empty or unreachable.');
    }

    const workspacePathFn = useWorkspaceStore.getState().getAgentPath;
    return rawAgents.map(raw => {
        const workspacePath = workspacePathFn(raw.id);
        return normalizeAgent(raw, workspacePath);
    });
};

/**
 * Persists an agent update. 
 * PROACTIVE PARITY: Emits an update to the backend registry immediately.
 */
export const persistAgentUpdate = async (agentId: string, updates: AgentOverride) => {
    // 1. Sync to Backend for Global Parity (Local overrides removed for security/integrity)
    try {
        const config: Record<string, unknown> = {};
        if (updates.role !== undefined) config.role = updates.role;
        if (updates.name !== undefined) config.name = updates.name;
        if (updates.model !== undefined) {
            config.modelId = updates.model;
            // Robustness: If model is updated via pick-list (where modelConfig might be omitted),
            // try to resolve the provider automatically if not provided.
            if (updates.modelConfig?.provider) {
                config.provider = updates.modelConfig.provider;
            } else {
                // Heuristic for common models if provider is missing
                const m = updates.model.toLowerCase();
                if (m.includes('gpt')) config.provider = 'openai';
                else if (m.includes('claude')) config.provider = 'anthropic';
                else if (m.includes('gemini')) config.provider = 'google';
                else if (m.includes('llama') || m.includes('mixtral')) config.provider = 'groq';
            }
        }
        if (updates.modelConfig !== undefined) {
            config.modelId = updates.modelConfig.modelId;
            config.provider = updates.modelConfig.provider;
            config.temperature = updates.modelConfig.temperature;
            config.systemPrompt = updates.modelConfig.systemPrompt;
            config.apiKey = updates.modelConfig.apiKey;
            config.baseUrl = updates.modelConfig.baseUrl;
        }
        if (updates.themeColor !== undefined) config.themeColor = updates.themeColor;
        if (updates.activeModelSlot !== undefined) config.activeModelSlot = updates.activeModelSlot;
        if (updates.modelConfig2 !== undefined) config.modelConfig2 = updates.modelConfig2;
        if (updates.modelConfig3 !== undefined) config.modelConfig3 = updates.modelConfig3;
        if (updates.budgetUsd !== undefined) config.budgetUsd = updates.budgetUsd;
        if (updates.skills !== undefined) config.skills = updates.skills;
        if (updates.workflows !== undefined) config.workflows = updates.workflows;
        if (updates.mcpTools !== undefined) config.mcpTools = updates.mcpTools;
        if (updates.category !== undefined) config.category = updates.category;
        if (updates.createdAt !== undefined) config.createdAt = updates.createdAt;
        if (updates.lastPulse !== undefined) config.lastPulse = updates.lastPulse;
        if (updates.inputTokens !== undefined) config.inputTokens = updates.inputTokens;
        if (updates.outputTokens !== undefined) config.outputTokens = updates.outputTokens;
        if (updates.tokensUsed !== undefined) config.tokensUsed = updates.tokensUsed;
        if (updates.failureCount !== undefined) config.failureCount = updates.failureCount;

        if (Object.keys(config).length > 0) {
            await AgentApiService.updateAgent(agentId, config);
        }
    } catch (e) {
        console.error('⚠️ [AgentService] Backend sync failed:', e);
        // We don't re-throw here to prevent UI lockup if the backend is flaky, 
        // as local overrides still work.
    }
};

/**
 * Returns the static mock agents synchronously (for initial render).
 * Use `loadAgents()` for the async version that checks TadpoleOS first.
 */
export const getMockAgents = (): Agent[] => mockAgents;


