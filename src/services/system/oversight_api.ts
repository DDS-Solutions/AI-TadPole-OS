/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / oversight_api
 * - **Primary Entrypoints**: `oversight_api`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { api_request } from '../base_api_service';
import type { Agent_Health, Audit_Entry, LedgerEntry, OversightEntry, Quota_Details, Quotas, RequestOptions } from '../system_api_types';
import { use_security_store } from '../../stores/security_store';
import { get_tadpole_os_socket } from '../socket';

const sanitize_id = (id: string): string => {
    if (!id || !/^[a-zA-Z0-9_.-]{1,64}$/.test(id)) {
        throw new Error(`Invalid identifier: ${id}`);
    }
    return encodeURIComponent(id);
};

export const oversight_api = {
    get_pending_oversight: async (options?: RequestOptions): Promise<OversightEntry[]> => {
        const res = await api_request<OversightEntry[] | { data?: OversightEntry[] }>('/v1/oversight/pending', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
        return Array.isArray(res) ? res : (res.data || []);
    },

    get_oversight_ledger: async (options?: RequestOptions): Promise<LedgerEntry[]> => {
        // Request max per_page (100) to avoid the default 25-entry cap.
        // If total_pages > 1, auto-fetch all remaining pages in parallel.
        const first = await api_request<{ data?: LedgerEntry[]; total_pages?: number; total?: number } | LedgerEntry[]>(
            '/v1/oversight/ledger?per_page=100&page=1',
            { method: 'GET', signal: options?.signal, timeout: options?.timeout }
        );
        if (Array.isArray(first)) return first;
        const initial = first.data || [];
        const total_pages = first.total_pages ?? 1;
        if (total_pages <= 1) return initial;
        // Fetch remaining pages in parallel
        const page_fetches = Array.from({ length: total_pages - 1 }, (_, i) =>
            api_request<{ data?: LedgerEntry[] } | LedgerEntry[]>(
                `/v1/oversight/ledger?per_page=100&page=${i + 2}`,
                { method: 'GET', signal: options?.signal, timeout: options?.timeout }
            ).then(r => Array.isArray(r) ? r : (r.data || []))
        );
        const rest = await Promise.all(page_fetches);
        return [...initial, ...rest.flat()];
    },

    decide_oversight: async (id: string, decision: 'approved' | 'rejected', options?: RequestOptions): Promise<void> => {
        const clean_id = sanitize_id(id);
        await use_security_store.getState().generate_keys_if_needed();
        
        const timestamp = Date.now();
        const nonce = Array.from(window.crypto.getRandomValues(new Uint8Array(8)))
            .map(b => b.toString(16).padStart(2, '0'))
            .join('');

        const { signature, verifying_key } = await use_security_store.getState().sign_oversight(
            clean_id,
            decision,
            timestamp,
            nonce
        );

        await api_request<void>(`/v1/oversight/${clean_id}/decide`, {
            method: 'POST',
            body: JSON.stringify({
                decision,
                signature,
                verifying_key,
                timestamp,
                nonce
            }),
            signal: options?.signal,
            timeout: options?.timeout
        });

        // Try sending via WebSocket if open as secondary propagation
        get_tadpole_os_socket().send_json({
            type: 'oversight:decision',
            id: clean_id,
            decision,
            signature,
            verifying_key,
            timestamp,
            nonce
        });
    },

    get_security_quotas: async (options?: RequestOptions): Promise<Quotas> => {
        return api_request<Quotas>('/v1/oversight/security/quotas', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    update_security_quota: async (entity_id: string, budget_usd: number, options?: RequestOptions): Promise<{ status: string }> => {
        const clean_entity_id = sanitize_id(entity_id);
        return api_request<{ status: string }>(`/v1/oversight/security/quotas/${clean_entity_id}`, {
            method: 'PUT',
            body: JSON.stringify({ budget_usd }),
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    get_mission_quotas: async (options?: RequestOptions): Promise<{ quotas: Quota_Details[] }> => {
        return api_request<{ quotas: Quota_Details[] }>('/v1/oversight/security/missions/quotas', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    update_mission_quota: async (cluster_id: string, budget_usd: number, options?: RequestOptions): Promise<{ status: string }> => {
        const clean_cluster_id = sanitize_id(cluster_id);
        return api_request<{ status: string }>(`/v1/oversight/security/missions/${clean_cluster_id}/quota`, {
            method: 'PUT',
            body: JSON.stringify({ budget_usd }),
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    get_audit_trail: async (page = 1, per_page = 50, options?: RequestOptions): Promise<{ data: Audit_Entry[]; total: number }> => {
        return api_request<{ data: Audit_Entry[]; total: number }>(`/v1/oversight/security/audit-trail?page=${page}&per_page=${per_page}`, { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    get_agent_health: async (options?: RequestOptions): Promise<{ agents: Agent_Health[] }> => {
        return api_request<{ agents: Agent_Health[] }>('/v1/oversight/security/health', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    get_security_snapshot: async (page = 1, per_page = 10, options?: RequestOptions): Promise<{ quotas: Quotas; agent_health: Agent_Health[]; audit_trail: { data: Audit_Entry[]; total: number } }> => {
        return api_request<{ quotas: Quotas; agent_health: Agent_Health[]; audit_trail: { data: Audit_Entry[]; total: number } }>(
            `/v1/oversight/security/snapshot?page=${page}&per_page=${per_page}`,
            { 
                method: 'GET',
                signal: options?.signal,
                timeout: options?.timeout
            }
        );
    },

    get_integrity_status: async (options?: RequestOptions): Promise<{ integrity_score: number, status: string, verified_count: number, total_count: number }> => {
        return api_request<{ integrity_score: number, status: string, verified_count: number, total_count: number }>('/v1/oversight/security/integrity', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    get_governance_settings: async (options?: RequestOptions): Promise<{
        auto_approve_safe_skills: boolean;
        privacy_mode?: boolean;
        cluster_privacy_policies?: Record<string, boolean>;
        max_agents?: number;
        max_clusters?: number;
        max_swarm_depth?: number;
        max_task_length?: number;
        default_budget_usd?: number;
        default_model?: string;
        default_provider?: string;
    }> => {
        return api_request('/v1/oversight/settings', {
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    update_governance_settings: async (settings: Record<string, unknown>, options?: RequestOptions): Promise<unknown> => {
        return api_request<unknown>('/v1/oversight/settings', {
            method: 'PUT',
            body: JSON.stringify(settings),
            signal: options?.signal,
            timeout: options?.timeout
        });
    }
};
