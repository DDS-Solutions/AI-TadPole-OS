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
import type { Agent_Health, Audit_Entry, LedgerEntry, OversightEntry, Quota_Details, Quotas } from '../system_api_types';
import { use_security_store } from '../../stores/security_store';
import { tadpole_os_socket } from '../socket';

export const oversight_api = {
    get_pending_oversight: async (): Promise<OversightEntry[]> => {
        const res = await api_request<OversightEntry[] | { data?: OversightEntry[] }>('/v1/oversight/pending', { method: 'GET' });
        return Array.isArray(res) ? res : (res.data || []);
    },

    get_oversight_ledger: async (): Promise<LedgerEntry[]> => {
        const res = await api_request<LedgerEntry[] | { data?: LedgerEntry[] }>('/v1/oversight/ledger', { method: 'GET' });
        return Array.isArray(res) ? res : (res.data || []);
    },

    decide_oversight: async (id: string, decision: 'approved' | 'rejected'): Promise<void> => {
        await use_security_store.getState().generate_keys_if_needed();
        const { signature, verifying_key } = await use_security_store.getState().sign_oversight(id, decision);

        // Try sending via WebSocket if open as secondary propagation
        tadpole_os_socket.send_json({
            type: 'oversight:decision',
            id,
            decision,
            signature,
            verifying_key
        });

        await api_request(`/v1/oversight/${id}/decide`, {
            method: 'POST',
            body: JSON.stringify({
                decision,
                signature,
                verifying_key
            })
        });
    },

    get_security_quotas: async (): Promise<Quotas> => {
        return api_request('/v1/oversight/security/quotas', { method: 'GET' });
    },

    update_security_quota: async (entity_id: string, budget_usd: number): Promise<{ status: string }> => {
        return api_request(`/v1/oversight/security/quotas/${entity_id}`, {
            method: 'PUT',
            body: JSON.stringify({ budget_usd })
        });
    },

    get_mission_quotas: async (): Promise<{ quotas: Quota_Details[] }> => {
        return api_request('/v1/oversight/security/missions/quotas', { method: 'GET' });
    },

    update_mission_quota: async (cluster_id: string, budget_usd: number): Promise<{ status: string }> => {
        return api_request(`/v1/oversight/security/missions/${cluster_id}/quota`, {
            method: 'PUT',
            body: JSON.stringify({ budget_usd })
        });
    },

    get_audit_trail: async (page = 1, per_page = 50): Promise<{ data: Audit_Entry[]; total: number }> => {
        return api_request(`/v1/oversight/security/audit-trail?page=${page}&per_page=${per_page}`, { method: 'GET' });
    },

    get_agent_health: async (): Promise<{ agents: Agent_Health[] }> => {
        return api_request('/v1/oversight/security/health', { method: 'GET' });
    },

    get_integrity_status: async (): Promise<{ integrity_score: number, status: string, verified_count: number, total_count: number }> => {
        return api_request('/v1/oversight/security/integrity', { method: 'GET' });
    },

    update_governance_settings: async (settings: Record<string, unknown>): Promise<unknown> => {
        return api_request('/v1/oversight/settings', {
            method: 'PUT',
            body: JSON.stringify(settings)
        });
    }
};

// Metadata: [oversight_api]
