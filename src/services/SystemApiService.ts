import type { SwarmNode } from '../types/index';
import { apiRequest, DEPLOY_TIMEOUT } from './BaseApiService';

/** Consumption metrics for budget governance. */
export interface QuotaDetails {
    entity_id: string;
    budget_usd: number;
    used_usd: number;
    reset_period: 'daily' | 'monthly' | 'never';
    last_reset_at: string;
    next_reset_at: string;
}

export interface SystemDefense {
    memoryPressure: number;
    cpuLoad: number;
    sandboxStatus: string;
    sandboxType: string;
    merkleIntegrity: number;
}

export interface Quotas {
    totalBudget: number;
    totalSpent: number;
    remaining: number;
    efficiency: number;
    agentQuotas: QuotaDetails[];
    systemDefense: SystemDefense;
}

/** Represents a single decision record in the immutable audit trail. */
export interface AuditEntry {
    id: string;
    agentId: string;
    skill: string | null;
    status: string;
    decision: string | null;
    decidedAt: string | null;
    createdAt: string;
    /** Whether the entry passed the cryptographic integrity check. */
    isVerified: boolean;
}

/** Real-time health metrics for an active agent. */
export interface AgentHealth {
    agentId: string;
    name: string;
    status: string;
    failureCount: number;
    lastFailureAt: string | null;
    /** True if failureCount is below the threshold and agent is not throttled. */
    isHealthy: boolean;
    /** True if the agent is currently paused due to repeated failure loops. */
    isThrottled: boolean;
}

export interface InfraNode { id: string; name: string; host: string; status: string;[key: string]: unknown; }

export interface ProviderTestConfig {
    id: string;
    name: string;
    protocol: string;
    apiKey?: string;
    baseUrl?: string;
    externalId?: string;
    customHeaders?: Record<string, string>;
    audioModel?: string;
    [key: string]: unknown;
}

export interface BenchmarkRecord {
    id: string;
    name: string;
    test_id: string;
    category: string;
    mean_ms: number;
    p95_ms?: number;
    p99_ms?: number;
    target_value?: string;
    status: string;
    metadata?: string;
    created_at: string;
    [key: string]: unknown;
}

export interface ScheduledJob {
    id: string;
    agent_id: string;
    workflow_id?: string | null;
    name: string;
    prompt: string;
    cron_expr: string;
    budget_usd: number;
    enabled: boolean;
    last_run_at: string | null;
    next_run_at: string;
    consecutive_failures: number;
    max_failures: number;
    created_at: string;
}

export interface WorkflowEntry {
    id: string;
    name: string;
    description: string | null;
    created_at: string;
}

export interface WorkflowStep {
    id: string;
    workflow_id: string;
    step_number: number;
    agent_id: string;
    prompt: string;
    budget_usd: number;
}

export interface ScheduledJobRun {
    id: string;
    job_id: string;
    mission_id: string | null;
    started_at: string;
    completed_at: string | null;
    status: string;
    cost_usd: number;
    output_summary: string | null;
}

export const SystemApiService = {
    /**
     * Checks if the TadpoleOS instance is reachable.
     */
    checkHealth: async (): Promise<boolean> => {
        try {
            await apiRequest('/v1/engine/health', { method: 'GET', timeout: 5000 });
            return true;
        } catch {
            return false;
        }
    },

    /**
     * Triggers a production deployment of the engine.
     */
    deployEngine: async (target?: string | number): Promise<{ status: string, output?: string }> => {
        const url = target ? `/v1/engine/deploy?target=${target}` : '/v1/engine/deploy';
        return apiRequest<{ status: string, output?: string }>(url, {
            method: 'POST',
            timeout: DEPLOY_TIMEOUT
        });
    },

    /**
     * Synthesizes text to audio using the backend TTS engine.
     */
    speak: async (text: string, voice?: string, engine?: string): Promise<Blob> => {
        return apiRequest<Blob>('/v1/engine/speak', {
            method: 'POST',
            body: JSON.stringify({ text, voice, engine }),
            responseType: 'blob'
        });
    },

    /**
     * Halts all running agents.
     */
    killAgents: async (): Promise<void> => {
        await apiRequest('/v1/engine/kill', { method: 'POST' });
    },

    /**
     * Shuts down the backend server.
     */
    shutdownEngine: async (): Promise<void> => {
        await apiRequest('/v1/engine/shutdown', { method: 'POST' });
    },

    /**
     * Transcribes audio using the backend's high-fidelity Whisper engine.
     */
    transcribe: async (audioBlob: Blob): Promise<string> => {
        const formData = new FormData();
        formData.append('file', audioBlob, 'speech.wav');

        const data = await apiRequest<{ text?: string }>('/v1/engine/transcribe', {
            method: 'POST',
            body: formData,
            headers: { 'Content-Type': undefined as unknown as string }
        });

        return data.text || '';
    },

    /**
     * Connectivity test Trace for a given provider configuration.
     */
    testProvider: async (config: ProviderTestConfig): Promise<{ status: string; latency?: number; message?: string }> => {
        try {
            return await apiRequest<{ status: string; latency?: number }>(`/v1/infra/providers/${config.id}/test`, {
                method: 'POST',
                body: JSON.stringify(config)
            });
        } catch (error) {
            const isTimeout = error === 'TIMEOUT';
            const msg = isTimeout
                ? 'Handshake timeout: The provider endpoint is unresponsive.'
                : (error instanceof Error ? error.message : 'Network connection refused.');
            return { status: 'error', message: msg };
        }
    },

    /**
     * Returns all registered Bunker nodes from the infrastructure tier.
     */
    getNodes: async (): Promise<SwarmNode[]> => {
        return apiRequest<SwarmNode[]>('/v1/infra/nodes', { method: 'GET' });
    },

    /**
     * Triggers a network discovery scan for new Bunkers.
     */
    discoverNodes: async (): Promise<{ status: string, discovered: string[] }> => {
        return apiRequest<{ status: string, discovered: string[] }>('/v1/infra/nodes/discover', { method: 'POST' });
    },

    /**
     * Returns all historical performance benchmark records.
     */
    getBenchmarks: async (): Promise<BenchmarkRecord[]> => {
        return apiRequest<BenchmarkRecord[]>('/v1/benchmarks', { method: 'GET' });
    },

    /**
     * Triggers a specific performance benchmark by test_id.
     */
    runBenchmark: async (testId: string): Promise<BenchmarkRecord> => {
        return apiRequest<BenchmarkRecord>(`/v1/benchmarks/run/${testId}`, { method: 'POST' });
    },

    /**
     * Lists all autonomous scheduled jobs.
     */
    getScheduledJobs: async (): Promise<ScheduledJob[]> => {
        const res = await apiRequest<{ jobs: ScheduledJob[] } | ScheduledJob[]>('/v1/continuity/jobs', { method: 'GET' });
        return Array.isArray(res) ? res : (res.jobs || []);
    },

    /**
     * Creates a new scheduled job for the Continuity Scheduler.
     */
    createScheduledJob: async (job: Partial<ScheduledJob>): Promise<ScheduledJob> => {
        return apiRequest<ScheduledJob>('/v1/continuity/jobs', {
            method: 'POST',
            body: JSON.stringify(job)
        });
    },

    /**
     * Updates an existing scheduled job.
     */
    updateScheduledJob: async (id: string, job: Partial<ScheduledJob>): Promise<ScheduledJob> => {
        return apiRequest<ScheduledJob>(`/v1/continuity/jobs/${id}`, {
            method: 'PUT',
            body: JSON.stringify(job)
        });
    },

    /**
     * Deletes a scheduled job.
     */
    deleteScheduledJob: async (id: string): Promise<void> => {
        return apiRequest<void>(`/v1/continuity/jobs/${id}`, { method: 'DELETE' });
    },

    /**
     * Fetches the run history for a specific scheduled job.
     */
    getScheduledJobRuns: async (id: string): Promise<ScheduledJobRun[]> => {
        const res = await apiRequest<{ runs: ScheduledJobRun[] } | ScheduledJobRun[]>(`/v1/continuity/jobs/${id}/runs`, { method: 'GET' });
        return Array.isArray(res) ? res : (res.runs || []);
    },

    /**
     * Fetches actions awaiting human approval.
     */
    getPendingOversight: async (): Promise<unknown[]> => {
        const res = await apiRequest<unknown | unknown[]>('/v1/oversight/pending', { method: 'GET' });
        return Array.isArray(res) ? res : ((res as { data?: unknown[] }).data || []);
    },

    /**
     * Fetches the historical ledger of all oversight decisions.
     */
    getOversightLedger: async (): Promise<unknown[]> => {
        const res = await apiRequest<unknown | unknown[]>('/v1/oversight/ledger', { method: 'GET' });
        return Array.isArray(res) ? res : ((res as { data?: unknown[] }).data || []);
    },

    /**
     * Records a decision (approve/reject) for a pending oversight action.
     */
    decideOversight: async (id: string, decision: 'approved' | 'rejected'): Promise<void> => {
        await apiRequest(`/v1/oversight/${id}/decide`, {
            method: 'POST',
            body: JSON.stringify({ decision })
        });
    },

    /**
     * Lists all available knowledge docs from the backend.
     */
    getKnowledgeDocs: async (): Promise<{ category: string; name: string; title: string; }[]> => {
        return apiRequest<{ category: string; name: string; title: string; }[]>('/v1/docs/knowledge', { method: 'GET' });
    },

    /**
     * Fetches a specific knowledge document's markdown content.
     */
    getKnowledgeDoc: async (category: string, name: string): Promise<string> => {
        return apiRequest<string>(`/v1/docs/knowledge/${category}/${name}`, {
            method: 'GET',
            headers: { 'Accept': 'text/markdown' },
            responseType: 'text'
        });
    },

    /**
     * Fetches the Operations Manual markdown content.
     */
    getOperationsManual: async (): Promise<string> => {
        return apiRequest<string>('/v1/docs/operations-manual', {
            method: 'GET',
            headers: { 'Accept': 'text/markdown' },
            responseType: 'text'
        });
    },

    /**
     * Returns all registered AI infrastructure providers.
     */
    getProviders: async (): Promise<Record<string, unknown>[]> => {
        return apiRequest<Record<string, unknown>[]>('/v1/infra/providers', { method: 'GET' });
    },

    /**
     * Updates or creates an AI infrastructure provider.
     */
    updateProvider: async (id: string, config: Record<string, unknown>): Promise<{ status: string }> => {
        return apiRequest<{ status: string }>(`/v1/infra/providers/${id}`, {
            method: 'PUT',
            body: JSON.stringify(config)
        });
    },

    /**
     * Deletes an AI infrastructure provider.
     */
    deleteProvider: async (id: string): Promise<void> => {
        await apiRequest(`/v1/infra/providers/${id}`, { method: 'DELETE' });
    },

    /**
     * Updates or creates an AI infrastructure model.
     */
    updateModel: async (id: string, entry: Record<string, unknown>): Promise<{ status: string }> => {
        return apiRequest<{ status: string }>(`/v1/infra/models/${id}`, {
            method: 'PUT',
            body: JSON.stringify(entry)
        });
    },

    /**
     * Deletes an AI infrastructure model entry.
     */
    deleteModel: async (id: string): Promise<void> => {
        await apiRequest(`/v1/infra/models/${id}`, { method: 'DELETE' });
    },

    /**
     * Returns all registered AI infrastructure models.
     */
    getModels: async (): Promise<Record<string, unknown>[]> => {
        return apiRequest<Record<string, unknown>[]>('/v1/infra/models', { method: 'GET' });
    },

    /**
     * Returns aggregate security quotas (budget vs spent).
     */
    getSecurityQuotas: async (): Promise<Quotas> => {
        return apiRequest('/v1/oversight/security/quotas', { method: 'GET' });
    },

    /**
     * Updates a specific security quota for an entity.
     */
    updateSecurityQuota: async (entityId: string, budgetUsd: number): Promise<{ status: string }> => {
        return apiRequest(`/v1/oversight/security/quotas/${entityId}`, {
            method: 'PUT',
            body: JSON.stringify({ budgetUsd })
        });
    },

    /**
     * Returns the full historical audit trail.
     */
    getAuditTrail: async (page = 1, perPage = 50): Promise<{ data: AuditEntry[]; total: number }> => {
        return apiRequest(`/v1/oversight/security/audit-trail?page=${page}&per_page=${perPage}`, { method: 'GET' });
    },

    /**
     * Returns health metrics for all agents.
     */
    getAgentHealth: async (): Promise<{ agents: AgentHealth[] }> => {
        return apiRequest('/v1/oversight/security/health', { method: 'GET' });
    },

    /**
     * Lists all existing workflows for scheduled jobs.
     */
    listContinuityWorkflows: async (): Promise<WorkflowEntry[]> => {
        const res = await apiRequest<{ workflows: WorkflowEntry[] } | WorkflowEntry[]>('/v1/continuity/workflows', { method: 'GET' });
        return Array.isArray(res) ? res : (res.workflows || []);
    },

    /**
     * Creates a new workflow definition for scheduled jobs.
     */
    createContinuityWorkflow: async (data: { name: string; description?: string }): Promise<WorkflowEntry> => {
        return apiRequest<WorkflowEntry>('/v1/continuity/workflows', {
            method: 'POST',
            body: JSON.stringify(data)
        });
    },

    /**
     * Adds a step to an existing continuity workflow.
     */
    addContinuityWorkflowStep: async (workflowId: string, step: Partial<WorkflowStep>): Promise<WorkflowStep> => {
        return apiRequest<WorkflowStep>(`/v1/continuity/workflows/${workflowId}/steps`, {
            method: 'POST',
            body: JSON.stringify(step)
        });
    },

    /**
     * Deletes a continuity workflow definition.
     */
    deleteContinuityWorkflow: async (workflowId: string): Promise<void> => {
        return apiRequest(`/v1/continuity/workflows/${workflowId}`, { method: 'DELETE' });
    },

    /**
     * Checks the Merkle chain integrity status.
     */
    getIntegrityStatus: async (): Promise<{ integrityScore: number, status: string, verifiedCount: number, totalCount: number }> => {
        return apiRequest('/v1/oversight/security/integrity', { method: 'GET' });
    }
};
