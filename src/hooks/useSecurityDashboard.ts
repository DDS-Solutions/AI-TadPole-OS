/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[useSecurityDashboard]` in observability traces.
 */

import { useState, useCallback, useEffect, useMemo, useRef } from 'react';
import { tadpole_os_service, type Quotas, type Audit_Entry, type Agent_Health, type Quota_Details } from '../services/tadpoleos_service';

export interface SecurityDashboardHook {
    quotas: Quotas | null;
    audit_trail: Audit_Entry[];
    agent_health: Agent_Health[];
    is_loading: boolean;
    is_stale: boolean;
    audit_page: number;
    audit_total: number;
    health_search: string;
    health_sort: 'name' | 'failures';
    sort_config: { key: 'name' | 'status' | 'quota', direction: 'asc' | 'desc' };
    is_updating_quota: string | null;
    
    set_audit_page: (page: number) => void;
    set_health_search: (search: string) => void;
    set_health_sort: (sort: 'name' | 'failures') => void;
    set_sort_config: (config: { key: 'name' | 'status' | 'quota', direction: 'asc' | 'desc' }) => void;
    update_quota: (entity_id: string, current_budget: number, increment: number) => Promise<void>;
    refresh: () => Promise<void>;
    
    sorted_quotas: Quota_Details[];
    sorted_health: Agent_Health[];
}

export function useSecurityDashboard(polling_interval_ms: number = 5000): SecurityDashboardHook {
    const [quotas, set_quotas] = useState<Quotas | null>(null);
    const [audit_trail, set_audit_trail] = useState<Audit_Entry[]>([]);
    const [agent_health, set_agent_health] = useState<Agent_Health[]>([]);
    const [is_loading, set_is_loading] = useState(true);
    const [is_stale, set_is_stale] = useState(false);
    const [is_updating_quota, set_is_updating_quota] = useState<string | null>(null);
    
    const [audit_page, set_audit_page] = useState(1);
    const [audit_total, set_audit_total] = useState(0);
    const [health_search, set_health_search] = useState('');
    const [health_sort, set_health_sort] = useState<'name' | 'failures'>('failures');
    const [sort_config, set_sort_config] = useState<{ key: 'name' | 'status' | 'quota', direction: 'asc' | 'desc' }>({ key: 'name', direction: 'asc' });

    const abort_controller_ref = useRef<AbortController | null>(null);

    const fetch_data = useCallback(async (is_polling = false) => {
        if (abort_controller_ref.current) {
            abort_controller_ref.current.abort();
        }
        abort_controller_ref.current = new AbortController();

        try {
            if (!is_polling) set_is_loading(true);
            
            const [q, a, h] = await Promise.all([
                tadpole_os_service.get_security_quotas(),
                tadpole_os_service.get_audit_trail(audit_page, 10),
                tadpole_os_service.get_agent_health()
            ]);

            set_quotas(q);
            set_audit_trail(a.data || []);
            set_audit_total(a.total || 0);
            set_agent_health(h.agents || []);
            set_is_stale(false);
        } catch (error: unknown) {
            if (error instanceof Error && error.name === 'AbortError') return;
            console.error("[use_security_dashboard] Failed to fetch security data", error);
            set_is_stale(true);
        } finally {
            if (!is_polling) set_is_loading(false);
        }
    }, [audit_page]);

    const update_quota = async (entity_id: string, current_budget: number, increment: number) => {
        set_is_updating_quota(entity_id);
        try {
            await tadpole_os_service.update_security_quota(entity_id, current_budget + increment);
            await fetch_data(true);
        } catch (error) {
            console.error("[use_security_dashboard] Failed to update quota", error);
            throw error;
        } finally {
            set_is_updating_quota(null);
        }
    };

    const sorted_quotas = useMemo(() => {
        if (!quotas?.agent_quotas) return [];
        return [...quotas.agent_quotas].sort((a, b) => {
            const health_a = agent_health.find(h => h.agent_id === a.entity_id);
            const health_b = agent_health.find(h => h.agent_id === b.entity_id);

            if (sort_config.key === 'name') {
                const name_a = health_a?.name || a.entity_id;
                const name_b = health_b?.name || b.entity_id;
                return sort_config.direction === 'asc'
                    ? name_a.localeCompare(name_b)
                    : name_b.localeCompare(name_a);
            }
            if (sort_config.key === 'status') {
                const status_a = health_a?.status || 'offline';
                const status_b = health_b?.status || 'offline';
                return sort_config.direction === 'asc'
                    ? status_a.localeCompare(status_b)
                    : status_b.localeCompare(status_a);
            }
            if (sort_config.key === 'quota') {
                const ratio_a = a.budget_usd > 0 ? a.used_usd / a.budget_usd : (a.used_usd > 0 ? 1 : 0);
                const ratio_b = b.budget_usd > 0 ? b.used_usd / b.budget_usd : (b.used_usd > 0 ? 1 : 0);
                return sort_config.direction === 'asc' ? ratio_a - ratio_b : ratio_b - ratio_a;
            }
            return 0;
        });
    }, [quotas, agent_health, sort_config]);

    const sorted_health = useMemo(() => {
        const filtered = agent_health.filter(a =>
            a.name.toLowerCase().includes(health_search.toLowerCase()) ||
            a.agent_id.toLowerCase().includes(health_search.toLowerCase())
        );

        return [...filtered].sort((a, b) => {
            if (health_sort === 'failures') {
                return b.failure_count - a.failure_count;
            }
            return a.name.localeCompare(b.name);
        });
    }, [agent_health, health_search, health_sort]);

    useEffect(() => {
        const load = (is_polling = false) => { void fetch_data(is_polling); };
        load();
        const interval = setInterval(() => load(true), polling_interval_ms);
        return () => {
            clearInterval(interval);
            if (abort_controller_ref.current) {
                abort_controller_ref.current.abort();
            }
        };
    }, [fetch_data, polling_interval_ms]);

    return {
        quotas,
        audit_trail,
        agent_health,
        is_loading,
        is_stale,
        audit_page,
        audit_total,
        health_search,
        health_sort,
        sort_config,
        is_updating_quota,
        set_audit_page,
        set_health_search,
        set_health_sort,
        set_sort_config,
        update_quota,
        refresh: () => fetch_data(true),
        sorted_quotas,
        sorted_health
    };
}

// Metadata: [useSecurityDashboard]
