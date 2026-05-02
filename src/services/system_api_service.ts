/**
 * @docs ARCHITECTURE:Services
 * @docs API_REFERENCE:Endpoints
 * 
 * ### AI Assist Note
 * **Engine Domain Service**: Low-level infrastructure interface for system health, deployment, and benchmark diagnostics. 
 * Orchestrates hardware discovery, scheduled job lifecycles, and direct Rust sidecar lifecycle commands.
 * Includes **IMR-01 (Intelligent Model Registry)** bridge for automated model discovery and capability sync.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Deployment timeout (exceeding 2h `DEPLOY_TIMEOUT`), hardware discovery failure (unreachable nodes), or job queue starvation.
 * - **IMR-01 Link**: See `sync_provider_models` for dynamic discovery logic.
 * - **Telemetry Link**: Look for `engine:health` or search `[SystemAPI]` in tracing logs.
 * 
 * @aiContext
 * - **Dependencies**: `base_api_service`.
 * - **Side Effects**: Triggers high-privilege platform commands (engine deploy/shutdown/restart).
 * - **Mocking**: Mock `api_request` from `base_api_service` for unit tests.
 */


import type { Swarm_Node } from '../types/index';
export type { Swarm_Node };
import { api_request, DEPLOY_TIMEOUT } from './base_api_service';

/** Quota_Details - Consumption metrics for budget governance. */
export interface Quota_Details {
    entity_id: string;
    budget_usd: number;
    used_usd: number;
    reset_period: 'daily' | 'monthly' | 'never';
    last_reset_at: string;
    next_reset_at: string;
}

export interface System_Defense {
    memory_pressure: number;
    cpu_load: number;
    sandbox_status: string;
    sandbox_type: string;
    merkle_integrity: number;
}

export interface Quotas {
    total_budget: number;
    total_spent: number;
    remaining: number;
    efficiency: number;
    agent_quotas: Quota_Details[];
    system_defense: System_Defense;
}

/** Audit_Entry - Represents a single decision record in the immutable audit trail. */
export interface Audit_Entry {
    id: string;
    agent_id: string;
    skill: string | null;
    status: string;
    decision: string | null;
    decided_at: string | null;
    created_at: string;
    /** Whether the entry passed the cryptographic integrity check. */
    is_verified: boolean;
}

/** Agent_Health - Real-time health metrics for an active agent. */
export interface Agent_Health {
    agent_id: string;
    name: string;
    status: string;
    failure_count: number;
    last_failure_at: string | null;
    /** True if failure_count is below the threshold and agent is not throttled. */
    is_healthy: boolean;
    /** True if the agent is currently paused due to repeated failure loops. */
    is_throttled: boolean;
}

export interface Store_Model {
    id: string;
    name: string;
    provider: string;
    description: string;
    size: string;
    vram: string;
    tags: string[];
}

export interface Infra_Node { id: string; name: string; host: string; status: string;[key: string]: unknown; }

export interface Provider_Test_Config {
    id: string;
    name: string;
    protocol: string;
    api_key?: string;
    base_url?: string;
    external_id?: string;
    custom_headers?: Record<string, string>;
    audio_model?: string;
    [key: string]: unknown;
}

export interface Benchmark_Record {
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

export interface Scheduled_Job {
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

export interface Workflow_Entry {
    id: string;
    name: string;
    description: string | null;
    created_at: string;
}

export interface Workflow_Step {
    id: string;
    workflow_id: string;
    step_number: number;
    agent_id: string;
    prompt: string;
    budget_usd: number;
}

export interface Scheduled_Job_Run {
    id: string;
    job_id: string;
    mission_id: string | null;
    started_at: string;
    completed_at: string | null;
    status: string;
    cost_usd: number;
    output_summary: string | null;
}

export const system_api_service = {
    /**
     * get_engine_status
     * Fetches detailed health and feature metrics from the engine.
     */
    get_engine_status: async (options: RequestInit = {}): Promise<{ status: string, version: string, heartbeat: string, active_agents: number, features: string[] } | null> => {
        try {
            return await api_request<{ status: string, version: string, heartbeat: string, active_agents: number, features: string[] }>('/v1/engine/health', { 
                method: 'GET', 
                timeout: 5000,
                ...options 
            });
        } catch {
            return null;
        }
    },

    /**
     * check_health
     * Checks if the TadpoleOS instance is reachable.
     */
    check_health: async (): Promise<boolean> => {
        try {
            const status = await system_api_service.get_engine_status();
            return status !== null;
        } catch {
            return false;
        }
    },

    /**
     * deploy_engine
     * Triggers a production deployment of the engine.
     * 
     * ### 🏗️ CI/CD Orchestration
     * This command initiates the native build/deploy cycle. It bypasses 
     * the standard 30s timeout and uses a specialized 2h `DEPLOY_TIMEOUT` 
     * block to accommodate heavy containerization and hardware provisioning.
     */
    deploy_engine: async (target?: string | number): Promise<{ status: string, output?: string }> => {
        const url = target ? `/v1/engine/deploy?target=${target}` : '/v1/engine/deploy';
        return api_request<{ status: string, output?: string }>(url, {
            method: 'POST',
            timeout: DEPLOY_TIMEOUT
        });
    },

    /**
     * speak
     * Synthesizes text to audio using the backend TTS engine.
     */
    speak: async (text: string, voice?: string, engine?: string): Promise<Blob> => {
        return api_request<Blob>('/v1/engine/speak', {
            method: 'POST',
            body: JSON.stringify({ text, voice, engine }),
            response_type: 'blob'
        });
    },

    /**
     * kill_agents
     * Halts all running agents.
     */
    kill_agents: async (): Promise<void> => {
        await api_request('/v1/engine/kill', { method: 'POST' });
    },

    /**
     * shutdown_engine
     * Shuts down the backend server.
     */
    shutdown_engine: async (): Promise<void> => {
        await api_request('/v1/engine/shutdown', { method: 'POST' });
    },

    /**
     * transcribe
     * Transcribes audio using the backend's high-fidelity Whisper engine.
     */
    transcribe: async (audio_blob: Blob): Promise<string> => {
        const form_data = new FormData();
        form_data.append('file', audio_blob, 'speech.wav');

        const data = await api_request<{ text?: string }>('/v1/engine/transcribe', {
            method: 'POST',
            body: form_data,
            headers: { 'Content-Type': undefined as unknown as string }
        });

        return data.text || '';
    },

    /**
     * test_provider
     * Connectivity test Trace for a given provider configuration.
     */
    test_provider: async (config: Provider_Test_Config): Promise<{ status: string; latency?: number; message?: string }> => {
        try {
            return await api_request<{ status: string; latency?: number }>(`/v1/infra/providers/${config.id}/test`, {
                method: 'POST',
                body: JSON.stringify(config)
            });
        } catch (error) {
            const is_timeout = error === 'TIMEOUT';
            const msg = is_timeout
                ? 'Handshake timeout: The provider endpoint is unresponsive.'
                : (error instanceof Error ? error.message : 'Network connection refused.');
            return { status: 'error', message: msg };
        }
    },

    /**
     * get_nodes
     * Returns all registered Bunker nodes from the infrastructure tier.
     */
    get_nodes: async (options: RequestInit = {}): Promise<Swarm_Node[]> => {
        return api_request<Swarm_Node[]>('/v1/infra/nodes', { 
            method: 'GET',
            ...options
        });
    },

    /**
     * discover_nodes
     * Triggers a network discovery scan for new Bunkers.
     */
    discover_nodes: async (): Promise<{ status: string, discovered: string[] }> => {
        return api_request<{ status: string, discovered: string[] }>('/v1/infra/nodes/discover', { method: 'POST' });
    },

    /**
     * get_benchmarks
     * Returns all historical performance benchmark records.
     */
    get_benchmarks: async (): Promise<Benchmark_Record[]> => {
        return api_request<Benchmark_Record[]>('/v1/benchmarks', { method: 'GET' });
    },

    /**
     * run_benchmark
     * Triggers a specific performance benchmark by test_id.
     */
    run_benchmark: async (test_id: string): Promise<Benchmark_Record> => {
        return api_request<Benchmark_Record>(`/v1/benchmarks/run/${test_id}`, { method: 'POST' });
    },

    /**
     * get_scheduled_jobs
     * Lists all autonomous scheduled jobs.
     */
    get_scheduled_jobs: async (): Promise<Scheduled_Job[]> => {
        const res = await api_request<{ jobs: Scheduled_Job[] } | Scheduled_Job[]>('/v1/continuity/jobs', { method: 'GET' });
        return Array.isArray(res) ? res : (res.jobs || []);
    },

    /**
     * create_scheduled_job
     * Creates a new scheduled job for the Continuity Scheduler.
     */
    create_scheduled_job: async (job: Partial<Scheduled_Job>): Promise<Scheduled_Job> => {
        return api_request<Scheduled_Job>('/v1/continuity/jobs', {
            method: 'POST',
            body: JSON.stringify(job)
        });
    },

    /**
     * update_scheduled_job
     * Updates an existing scheduled job.
     */
    update_scheduled_job: async (id: string, job: Partial<Scheduled_Job>): Promise<Scheduled_Job> => {
        return api_request<Scheduled_Job>(`/v1/continuity/jobs/${id}`, {
            method: 'PUT',
            body: JSON.stringify(job)
        });
    },

    /**
     * delete_scheduled_job
     * Deletes a scheduled job.
     */
    delete_scheduled_job: async (id: string): Promise<void> => {
        return api_request<void>(`/v1/continuity/jobs/${id}`, { method: 'DELETE' });
    },

    /**
     * get_scheduled_job_runs
     * Fetches the run history for a specific scheduled job.
     */
    get_scheduled_job_runs: async (id: string): Promise<Scheduled_Job_Run[]> => {
        const res = await api_request<{ runs: Scheduled_Job_Run[] } | Scheduled_Job_Run[]>(`/v1/continuity/jobs/${id}/runs`, { method: 'GET' });
        return Array.isArray(res) ? res : (res.runs || []);
    },

    /**
     * get_pending_oversight
     * Fetches actions awaiting human approval.
     */
    get_pending_oversight: async (): Promise<unknown[]> => {
        const res = await api_request<unknown | unknown[]>('/v1/oversight/pending', { method: 'GET' });
        return Array.isArray(res) ? res : ((res as { data?: unknown[] }).data || []);
    },

    /**
     * get_oversight_ledger
     * Fetches the historical ledger of all oversight decisions.
     */
    get_oversight_ledger: async (): Promise<unknown[]> => {
        const res = await api_request<unknown | unknown[]>('/v1/oversight/ledger', { method: 'GET' });
        return Array.isArray(res) ? res : ((res as { data?: unknown[] }).data || []);
    },

    /**
     * decide_oversight
     * Records a decision (approve/reject) for a pending oversight action.
     */
    decide_oversight: async (id: string, decision: 'approved' | 'rejected'): Promise<void> => {
        await api_request(`/v1/oversight/${id}/decide`, {
            method: 'POST',
            body: JSON.stringify({ decision })
        });
    },

    /**
     * get_knowledge_docs
     * Lists all available knowledge docs from the backend.
     */
    get_knowledge_docs: async (): Promise<{ category: string; name: string; title: string; }[]> => {
        return api_request<{ category: string; name: string; title: string; }[]>('/v1/docs/knowledge', { method: 'GET' });
    },

    /**
     * get_knowledge_doc
     * Fetches a specific knowledge document's markdown content.
     */
    get_knowledge_doc: async (category: string, name: string): Promise<string> => {
        return api_request<string>(`/v1/docs/knowledge/${category}/${name}`, {
            method: 'GET',
            headers: { 'Accept': 'text/markdown' },
            response_type: 'text'
        });
    },

    /**
     * get_operations_manual
     * Fetches the Operations Manual markdown content.
     */
    get_operations_manual: async (): Promise<string> => {
        return api_request<string>('/v1/docs/operations-manual', {
            method: 'GET',
            headers: { 'Accept': 'text/markdown' },
            response_type: 'text'
        });
    },

    /**
     * get_providers
     * Returns all registered AI infrastructure providers.
     */
    get_providers: async (): Promise<Record<string, unknown>[]> => {
        return api_request<Record<string, unknown>[]>('/v1/infra/providers', { method: 'GET' });
    },

    /**
     * update_provider
     * Updates or creates an AI infrastructure provider.
     */
    update_provider: async (id: string, config: Record<string, unknown>): Promise<{ status: string }> => {
        return api_request<{ status: string }>(`/v1/infra/providers/${id}`, {
            method: 'PUT',
            body: JSON.stringify(config)
        });
    },

    /**
     * delete_provider
     * Deletes an AI infrastructure provider.
     */
    delete_provider: async (id: string): Promise<void> => {
        await api_request(`/v1/infra/providers/${id}`, { method: 'DELETE' });
    },

    /**
     * sync_provider_models
     * Triggers dynamic discovery of available models for a provider via IMR-01.
     * 
     * ### 💎 Intelligent Model Registry (IMR-01)
     * Queries the provider's native list-models endpoint and synchronizes 
     * with the local `infra_models.json` registry. Automatically 
     * reconciles pricing/token mappings for new capabilities (IMR-P2).
     */
    sync_provider_models: async (id: string): Promise<{ status: string; added: number; discovered: number; message: string }> => {
        return api_request<{ status: string; added: number; discovered: number; message: string }>(`/v1/infra/providers/${id}/sync`, {
            method: 'POST'
        });
    },

    /**
     * update_model
     * Updates or creates an AI infrastructure model.
     */
    update_model: async (id: string, entry: Record<string, unknown>): Promise<{ status: string }> => {
        return api_request<{ status: string }>(`/v1/infra/models/${id}`, {
            method: 'PUT',
            body: JSON.stringify(entry)
        });
    },

    /**
     * delete_model
     * Deletes an AI infrastructure model entry.
     */
    delete_model: async (id: string): Promise<void> => {
        await api_request(`/v1/infra/models/${id}`, { method: 'DELETE' });
    },

    /**
     * get_models
     * Returns all registered AI infrastructure models.
     */
    get_models: async (): Promise<Record<string, unknown>[]> => {
        return api_request<Record<string, unknown>[]>('/v1/infra/models', { method: 'GET' });
    },

    /**
     * get_security_quotas
     * Returns aggregate security quotas (budget vs spent).
     */
    get_security_quotas: async (): Promise<Quotas> => {
        return api_request('/v1/oversight/security/quotas', { method: 'GET' });
    },

    /**
     * update_security_quota
     * Updates a specific security quota for an entity.
     */
    update_security_quota: async (entity_id: string, budget_usd: number): Promise<{ status: string }> => {
        return api_request(`/v1/oversight/security/quotas/${entity_id}`, {
            method: 'PUT',
            body: JSON.stringify({ budget_usd })
        });
    },

    /**
     * get_mission_quotas
     * Returns all registered mission quotas.
     */
    get_mission_quotas: async (): Promise<{ quotas: Quota_Details[] }> => {
        return api_request('/v1/oversight/security/missions/quotas', { method: 'GET' });
    },

    /**
     * update_mission_quota
     * Updates the budget quota for a specific mission cluster.
     */
    update_mission_quota: async (cluster_id: string, budget_usd: number): Promise<{ status: string }> => {
        return api_request(`/v1/oversight/security/missions/${cluster_id}/quota`, {
            method: 'PUT',
            body: JSON.stringify({ budget_usd })
        });
    },

    /**
     * get_audit_trail
     * Returns the full historical audit trail.
     */
    get_audit_trail: async (page = 1, per_page = 50): Promise<{ data: Audit_Entry[]; total: number }> => {
        return api_request(`/v1/oversight/security/audit-trail?page=${page}&per_page=${per_page}`, { method: 'GET' });
    },

    /**
     * get_agent_health
     * Returns health metrics for all agents.
     */
    get_agent_health: async (): Promise<{ agents: Agent_Health[] }> => {
        return api_request('/v1/oversight/security/health', { method: 'GET' });
    },

    /**
     * list_continuity_workflows
     * Lists all existing workflows for scheduled jobs.
     */
    list_continuity_workflows: async (): Promise<Workflow_Entry[]> => {
        const res = await api_request<{ workflows: Workflow_Entry[] } | Workflow_Entry[]>('/v1/continuity/workflows', { method: 'GET' });
        return Array.isArray(res) ? res : (res.workflows || []);
    },

    /**
     * create_continuity_workflows
     * Creates a new workflow definition for scheduled jobs.
     */
    create_continuity_workflows: async (data: { name: string; description?: string }): Promise<Workflow_Entry> => {
        return api_request<Workflow_Entry>('/v1/continuity/workflows', {
            method: 'POST',
            body: JSON.stringify(data)
        });
    },

    /**
     * add_continuity_workflows_step
     * Adds a step to an existing continuity workflow.
     */
    add_continuity_workflows_step: async (workflow_id: string, step: Partial<Workflow_Step>): Promise<Workflow_Step> => {
        return api_request<Workflow_Step>(`/v1/continuity/workflows/${workflow_id}/steps`, {
            method: 'POST',
            body: JSON.stringify(step)
        });
    },

    /**
     * delete_continuity_workflows
     * Deletes a continuity workflow definition.
     */
    delete_continuity_workflows: async (workflow_id: string): Promise<void> => {
        return api_request(`/v1/continuity/workflows/${workflow_id}`, { method: 'DELETE' });
    },

    /**
     * get_integrity_status
     * Checks the Merkle chain integrity status.
     */
    get_integrity_status: async (): Promise<{ integrity_score: number, status: string, verified_count: number, total_count: number }> => {
        return api_request('/v1/oversight/security/integrity', { method: 'GET' });
    },

    /**
     * update_governance_settings
     * Updates global governance and oversight settings.
     */
    update_governance_settings: async (settings: Record<string, unknown>): Promise<unknown> => {
        return api_request('/v1/oversight/settings', {
            method: 'PUT',
            body: JSON.stringify(settings)
        });
    },

    /**
     * get_model_catalog
     * Fetches the curated model catalog from the infrastructure tier.
     */
    get_model_catalog: async (): Promise<Store_Model[]> => {
        return api_request<Store_Model[]>('/v1/infra/model-store/catalog', { method: 'GET' });
    },

    /**
     * pull_model
     * Initiates a model pull/download sequence on a specific Bunker node.
     */
    pull_model: async (model_id: string, node_id: string): Promise<{ status: string }> => {
        return api_request<{ status: string }>('/v1/infra/model-store/pull', {
            method: 'POST',
            body: JSON.stringify({ tag: model_id, node_id: node_id })
        });
    },

    /**
     * install_template
     * Installs a template from the official repository.
     */
    install_template: async (repository_url: string, path: string): Promise<void> => {
        await api_request('/v1/engine/templates/install', {
            method: 'POST',

            body: JSON.stringify({ repository_url: repository_url, path })
        });
    }
};

// Re-export types for consumers
export type { Skill_Manifest } from './mission_api_service';


// Metadata: [system_api_service]

// Metadata: [system_api_service]
