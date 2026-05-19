/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[useOversightDashboard]` in observability traces.
 */

import { useState, useEffect, useMemo, useCallback } from 'react';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { useEngineStatus } from './use_engine_status';
import { MOCK_PENDING, MOCK_LEDGER, type OversightEntry, type LedgerEntry } from '../data/mock_oversight';
import { agents as all_agents } from '../data/mock_agents';
import { i18n } from '../i18n';

export interface OversightStats {
    pending: number;
    approved: number;
    rejected: number;
}

export interface UseOversightDashboardHook {
    pending: OversightEntry[];
    ledger: LedgerEntry[];
    stats: OversightStats;
    filter: string;
    is_simulated: boolean;
    selected_cluster_id: string;
    is_online: boolean;
    
    // Actions
    set_filter: (filter: string) => void;
    set_selected_cluster_id: (id: string) => void;
    set_is_simulated: (simulated: boolean) => void;
    handle_decide: (id: string, decision: 'approved' | 'rejected') => Promise<void>;
    handle_kill_switch: () => Promise<void>;
    handle_kill_engine: () => Promise<void>;
    resolve_agent_name: (id: string) => string;
    
    // Filtered data
    filtered_ledger: LedgerEntry[];
    filtered_pending: OversightEntry[];
}

export function useOversightDashboard(): UseOversightDashboardHook {
    const { is_online } = useEngineStatus();

    const [pending, set_pending] = useState<OversightEntry[]>(() => {
        const saved = localStorage.getItem('tadpole_oversight_pending');
        if (saved) {
            try { return JSON.parse(saved); } catch { return []; }
        }
        return [];
    });

    const [ledger, set_ledger] = useState<LedgerEntry[]>(() => {
        const saved = localStorage.getItem('tadpole_oversight_ledger');
        if (saved) {
            try { return JSON.parse(saved); } catch { return []; }
        }
        return [];
    });

    const [filter, set_filter] = useState('');
    const [is_simulated, set_is_simulated] = useState(false);
    const [has_attempted_fetch, set_has_attempted_fetch] = useState(false);
    const [selected_cluster_id, set_selected_cluster_id] = useState<string>('all');

    // Persistence
    useEffect(() => {
        if (pending.length > 0 || has_attempted_fetch) {
            localStorage.setItem('tadpole_oversight_pending', JSON.stringify(pending));
        }
    }, [pending, has_attempted_fetch]);

    useEffect(() => {
        if (ledger.length > 0 || has_attempted_fetch) {
            localStorage.setItem('tadpole_oversight_ledger', JSON.stringify(ledger));
        }
    }, [ledger, has_attempted_fetch]);

    const fetch_data = useCallback(async () => {
        if (is_simulated) {
            if (pending.length === 0 && !has_attempted_fetch) {
                set_pending(MOCK_PENDING);
                set_ledger(MOCK_LEDGER);
            }
            return;
        }

        try {
            const [pending_data, ledger_data] = await Promise.all([
                tadpole_os_service.get_pending_oversight(),
                tadpole_os_service.get_oversight_ledger()
            ]);

            set_pending(pending_data);
            set_ledger(ledger_data);
            set_is_simulated(false);
            set_has_attempted_fetch(true);
        } catch {
            if (!has_attempted_fetch) {
                set_pending(MOCK_PENDING);
                set_ledger(MOCK_LEDGER);
                set_is_simulated(true);
                set_has_attempted_fetch(true);
            }
        }
    }, [is_simulated, pending.length, has_attempted_fetch]);

    useEffect(() => {
        const load = () => { void fetch_data(); };
        load();
        const interval = setInterval(() => {
            if (document.visibilityState === 'visible') load();
        }, is_simulated ? 5000 : 2000);
        return () => clearInterval(interval);
    }, [fetch_data, is_simulated]);

    const stats = useMemo(() => ({
        pending: pending.length,
        approved: ledger.filter((entry) => entry.decision === 'approved').length,
        rejected: ledger.filter((entry) => entry.decision === 'rejected').length
    }), [pending, ledger]);

    const resolve_agent_name = useCallback((id: string) => {
        const agent = all_agents.find(a => a.id === id || a.id.split('-').pop() === id);
        return agent?.name || id;
    }, []);

    const handle_decide = async (id: string, decision: 'approved' | 'rejected') => {
        if (is_simulated) {
            const entry = pending.find(p => p.id === id);
            if (entry) {
                set_ledger(prev => [{ ...entry, decision, timestamp: new Date().toISOString() }, ...prev]);
                set_pending(prev => prev.filter(p => p.id !== id));
            }
            return;
        }

        try {
            await tadpole_os_service.decide_oversight(id, decision);
            set_pending(prev => prev.filter(p => p.id !== id));
        } catch (error) {
            console.error('Failed to decide oversight:', error);
        }
    };

    const handle_kill_switch = async () => {
        if (!confirm(i18n.t('oversight.confirm_halt_agents'))) return;

        if (is_simulated) {
            set_pending([]);
            return;
        }

        try {
            await tadpole_os_service.kill_agents();
            alert(i18n.t('oversight.agents_halted'));
        } catch {
            alert(i18n.t('oversight.halt_agents_failed'));
        }
    };

    const handle_kill_engine = async () => {
        if (!confirm(i18n.t('oversight.confirm_kill_engine'))) return;

        const userInput = prompt(i18n.t('oversight.shutdown_confirm_prompt'));
        if (userInput !== 'SHUTDOWN') return;

        try {
            await tadpole_os_service.shutdown_engine();
            alert(i18n.t('oversight.engine_shutting_down'));
        } catch (e: unknown) {
            alert(i18n.t('oversight.kill_engine_failed', { error: e instanceof Error ? e.message : 'Unknown error' }));
        }
    };

    const filtered_ledger = useMemo(() => {
        return ledger
            .filter(l => {
                const tool_call = l.tool_call || l;
                const matches_cluster = selected_cluster_id === 'all' || tool_call.cluster_id === selected_cluster_id;
                const matches_search = (tool_call.agent_id || '').toLowerCase().includes(filter.toLowerCase()) ||
                    (tool_call.skill || '').toLowerCase().includes(filter.toLowerCase()) ||
                    JSON.stringify(tool_call.params || {}).toLowerCase().includes(filter.toLowerCase());
                return matches_cluster && matches_search;
            })
            .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());
    }, [ledger, selected_cluster_id, filter]);

    const filtered_pending = useMemo(() => {
        return pending.filter(p => {
            const tool_call = p.tool_call || p;
            return selected_cluster_id === 'all' || tool_call.cluster_id === selected_cluster_id;
        });
    }, [pending, selected_cluster_id]);

    return {
        pending,
        ledger,
        stats,
        filter,
        is_simulated,
        selected_cluster_id,
        is_online,
        set_filter,
        set_selected_cluster_id,
        set_is_simulated,
        handle_decide,
        handle_kill_switch,
        handle_kill_engine,
        resolve_agent_name,
        filtered_ledger,
        filtered_pending
    };
}

// Metadata: [useOversightDashboard]
