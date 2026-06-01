/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[system_api_types]` in observability traces.
 */

import type { Swarm_Node } from '../types/index';
import type { OversightEntry, LedgerEntry } from '../types/oversight';

export type { Swarm_Node, OversightEntry, LedgerEntry };

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

export interface Audit_Entry {
    id: string;
    agent_id: string;
    skill: string | null;
    status: string;
    decision: string | null;
    decided_at: string | null;
    created_at: string;
    is_verified: boolean;
}

export interface Agent_Health {
    agent_id: string;
    name: string;
    status: string;
    failure_count: number;
    last_failure_at: string | null;
    is_healthy: boolean;
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

export interface Infra_Node {
    id: string;
    name: string;
    host: string;
    status: string;
    [key: string]: unknown;
}

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

export interface Workspace_Status {
    id: string;
    agent_id: string;
    source_type: string;
    source_uri: string;
    status: string;
    last_sync_at: string | null;
    file_count: number;
    total_bytes: number;
}

// Metadata: [system_api_types]
