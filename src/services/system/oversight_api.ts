/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[oversight_api]` in observability traces.
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
        const res = await api_request<LedgerEntry[] | { data?: LedgerEntry[] }>('/v1/oversight/ledger', { 
            method: 'GET',
            signal: options?.signal,
            timeout: options?.timeout
        });
        return Array.isArray(res) ? res : (res.data || []);
    },

    decide_oversight: async (id: string, decision: 'approved' | 'rejected', options?: RequestOptions): Promise<void> => {
        const clean_id = sanitize_id(id);
        await use_security_store.getState().generate_keys_if_needed();
        const { signature, verifying_key } = await use_security_store.getState().sign_oversight(clean_id, decision);

        await api_request<void>(`/v1/oversight/${clean_id}/decide`, {
            method: 'POST',
            body: JSON.stringify({
                decision,
                signature,
                verifying_key
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
            verifying_key
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

    get_integrity_status: async (options?: RequestOptions): Promise<{ integrity_score: number, status: string, verified_count: number, total_count: number }> => {
        return api_request<{ integrity_score: number, status: string, verified_count: number, total_count: number }>('/v1/oversight/security/integrity', { 
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

// Metadata: [oversight_api]
