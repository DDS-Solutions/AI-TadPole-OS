/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: System-wide security and encryption oversight hub. 
 * Orchestrates the visualization of active defense layers, key rotations, and intrusion detection logs.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Security alert masking by UI noise, or vault health reporting lag.
 * - **Telemetry Link**: Search for `[Security_Dashboard]` or `DEFENSE_PULSE` in service logs.
 */

import { useEffect, useState } from 'react';
import { 
    ShieldCheck, 
    Lock, 
    Activity, 
    AlertCircle, 
    Heart, 
    Zap, 
    History,
    DollarSign,
    CheckCircle2,
    ArrowUpDown,
    Plus,
    Minus
} from 'lucide-react';
import { tadpole_os_service, type Quotas, type Audit_Entry, type Agent_Health } from '../services/tadpoleos_service';
import { Tooltip } from '../components/ui';
import { i18n } from '../i18n';

/**
 * Security Dashboard: The central nexus for system-wide governance monitoring.
 * 
 * ### 📊 Core Responsibilities
 * 1. **Budget Enforcement**: Real-time tracking of mission spend vs. quotas.
 * 2. **Audit Verification**: Visualization of the Merkle Hash Chain integrity.
 * 3. **Swarm Health**: Monitoring agent heartbeats and failure rates.
 * 4. **Defense Matrix**: Real-time stats on sandbox isolation and secret redaction.
 */
export default function Security_Dashboard() {
    const [quotas_state, set_quotas_state] = useState<Quotas | null>(null);
    const [audit_trail_state, set_audit_trail_state] = useState<Audit_Entry[]>([]);
    const [agent_health_state, set_agent_health_state] = useState<Agent_Health[]>([]);
    const [is_loading, set_is_loading] = useState(true);
    const [sort_config, set_sort_config] = useState<{ key: 'name' | 'status' | 'quota', direction: 'asc' | 'desc' }>({ key: 'name', direction: 'asc' });
    const [is_updating_quota, set_is_updating_quota] = useState<string | null>(null);

    /**
     * Orchestrates the multi-source data load from the Tadpole OS service.
     * Fetches quotas, audit trails (paginated), and global agent health.
     */
    const fetch_data = async () => {
        try {
            const [q, a, h] = await Promise.all([
                tadpole_os_service.get_security_quotas(),
                tadpole_os_service.get_audit_trail(1, 10),
                tadpole_os_service.get_agent_health()
            ]);
            set_quotas_state(q);
            set_audit_trail_state(a.data || []);
            set_agent_health_state(h.agents || []);
        } catch (error: unknown) {
            console.error("Failed to fetch security data", error);
        } finally {
            set_is_loading(false);
        }
    };

    /**
     * Updates an entity's resource quota. 
     * Triggers a 'is_updating' state to prevent UI double-submission.
     */
    const handle_update_quota = async (entity_id: string, current_budget: number, increment: number) => {
        set_is_updating_quota(entity_id);
        try {
            await tadpole_os_service.update_security_quota(entity_id, current_budget + increment);
            await fetch_data();
        } catch (error) {
            console.error("Failed to update quota", error);
        } finally {
            set_is_updating_quota(null);
        }
    };

    const get_sorted_quotas = () => {
        if (!quotas_state?.agent_quotas) return [];
        return [...quotas_state.agent_quotas].sort((a, b) => {
            const health_a = agent_health_state.find(h => h.agent_id === a.entity_id);
            const health_b = agent_health_state.find(h => h.agent_id === b.entity_id);

            if (sort_config.key === 'name') {
                return sort_config.direction === 'asc' 
                    ? a.entity_id.localeCompare(b.entity_id)
                    : b.entity_id.localeCompare(a.entity_id);
            }
            if (sort_config.key === 'status') {
                const status_a = health_a?.status || 'offline';
                const status_b = health_b?.status || 'offline';
                return sort_config.direction === 'asc'
                    ? status_a.localeCompare(status_b)
                    : status_b.localeCompare(status_a);
            }
            if (sort_config.key === 'quota') {
                const ratio_a = a.used_usd / a.budget_usd;
                const ratio_b = b.used_usd / b.budget_usd;
                return sort_config.direction === 'asc' ? ratio_a - ratio_b : ratio_b - ratio_a;
            }
            return 0;
        });
    };

    useEffect(() => {
        void (async () => {
            await Promise.resolve();
            fetch_data();
        })();
        const interval = setInterval(fetch_data, 5000);
        return () => clearInterval(interval);
    }, []);

    if (is_loading && !quotas_state) {
        return (
            <div className="flex items-center justify-center p-20">
                <Activity className="animate-spin text-green-500 mr-2" />
                <span className="text-zinc-400 font-mono">{i18n.t('security.loading')}</span>
            </div>
        );
    }

    return (
        <div className="p-6 space-y-6 max-w-7xl mx-auto" aria-label="Security Dashboard">
            {/* GEO Optimization: Structured Data & Semantic Header */}
            <script type="application/ld+json">
            {JSON.stringify({
              "@context": "https://schema.org",
              "@type": "SoftwareApplication",
              "name": "Tadpole OS Security Hub",
              "description": "System-wide governance monitoring, budget enforcement, and cryptographic audit verification center.",
              "provider": { "@type": "Organization", "name": "Sovereign Engineering" },
              "applicationCategory": "Security System"
            })}
            </script>
            <header className="flex justify-between items-start bg-zinc-900 border border-zinc-800 p-6 rounded-2xl">
                <div className="space-y-1">
                    <h1 className="text-2xl font-bold text-zinc-100 flex items-center gap-2">
                        <ShieldCheck className="text-emerald-500" />
                        {i18n.t('security.title')}
                    </h1>
                    <p className="text-zinc-500 text-sm mt-1">
                        {i18n.t('security.subtitle')}
                    </p>
                </div>
                <div className="flex items-center gap-3">
                    <div className="grid grid-rows-4 grid-flow-col gap-1">
                        {agent_health_state.map(a => (
                            <Tooltip key={a.agent_id} content={`${a.name}: ${a.is_healthy ? i18n.t('security.status_healthy') : i18n.t('security.status_degraded')}`}>
                                <div className={`w-8 h-8 rounded-full border-2 border-zinc-900 flex items-center justify-center text-[10px] font-bold ${a.is_healthy ? 'bg-emerald-500/20 text-emerald-400' : 'bg-red-500/20 text-red-400'}`}>
                                    {a.name.charAt(0)}
                                </div>
                            </Tooltip>
                        ))}
                    </div>
                    <div className="h-8 w-px bg-zinc-800 mx-2" />
                    {/* Merkle Integrity Pulse: Displays the cryptographic health of the Audit Trail */}
                    <Tooltip content={i18n.t('security.tooltip_audit_integrity')}>
                        <div className={`px-3 py-1 border rounded-full flex items-center gap-2 ${quotas_state?.system_defense?.merkle_integrity === 1.0 ? 'bg-emerald-500/10 border-emerald-500/30' : 'bg-red-500/10 border-red-500/30'}`}>
                            <div className={`w-2 h-2 rounded-full animate-pulse ${quotas_state?.system_defense?.merkle_integrity === 1.0 ? 'bg-emerald-500' : 'bg-red-500'}`} />
                            <span className={`text-[10px] font-bold uppercase tracking-widest ${quotas_state?.system_defense?.merkle_integrity === 1.0 ? 'text-emerald-400' : 'text-red-400'}`}>
                                {quotas_state?.system_defense?.merkle_integrity === 1.0 ? i18n.t('security.system_secured') : i18n.t('security.integrity_compromised')}
                            </span>
                        </div>
                    </Tooltip>
                </div>
            </header>

            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                <Tooltip content={i18n.t('security.tooltip_budget_card')}>
                    <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-xl cursor-help">
                        <div className="flex items-center justify-between mb-2">
                            <p className="text-zinc-400 text-xs font-mono uppercase">{i18n.t('security.budget_consumption')}</p>
                            <DollarSign size={14} className="text-zinc-500" />
                        </div>
                        <p className="text-2xl font-bold text-zinc-100">${(quotas_state?.total_spent || 0).toFixed(2)}</p>
                        <div className="mt-3 h-1.5 bg-zinc-800 rounded-full overflow-hidden">
                            <div 
                                className="h-full bg-green-500" 
                                style={{ width: `${Math.min(quotas_state?.efficiency || 0, 100)}%` }}
                            />
                        </div>
                        <p className="text-[10px] text-zinc-500 mt-2">
                            {i18n.t('security.efficiency_label', { percentage: (quotas_state?.efficiency ?? 0).toFixed(1) })}
                        </p>
                    </div>
                </Tooltip>

                <Tooltip content={i18n.t('security.tooltip_agents_card')}>
                    <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-xl cursor-help">
                        <div className="flex items-center justify-between mb-2">
                            <p className="text-zinc-400 text-xs font-mono uppercase">{i18n.t('security.active_agents')}</p>
                            <Zap size={14} className="text-amber-500" />
                        </div>
                        <p className="text-2xl font-bold text-zinc-100">{agent_health_state.length}</p>
                        <p className="text-[10px] text-zinc-500 mt-2 font-mono">
                            {i18n.t('security.executing_missions', { count: agent_health_state.filter(a => a.status === 'active').length })}
                        </p>
                    </div>
                </Tooltip>

                <Tooltip content={i18n.t('security.tooltip_health_card')}>
                    <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-xl cursor-help">
                        <div className="flex items-center justify-between mb-2">
                            <p className="text-zinc-400 text-xs font-mono uppercase">{i18n.t('security.system_health')}</p>
                            <Heart size={14} className="text-emerald-500" />
                        </div>
                        <p className="text-2xl font-bold text-zinc-100">
                            {i18n.t('security.health_ratio', { healthy: agent_health_state.filter(a => a.is_healthy).length, total: agent_health_state.length })}
                        </p>
                        <p className="text-[10px] text-zinc-500 mt-2 font-mono uppercase tracking-tighter">
                            {agent_health_state.every(a => a.is_healthy) ? i18n.t('security.optimal_ops') : i18n.t('security.degraded_ops')}
                        </p>
                    </div>
                </Tooltip>

                <Tooltip content={i18n.t('security.tooltip_decisions_card')}>
                    <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-xl cursor-help">
                        <div className="flex items-center justify-between mb-2">
                            <p className="text-zinc-400 text-xs font-mono uppercase">{i18n.t('security.verified_decisions')}</p>
                            <Lock size={14} className={quotas_state?.system_defense?.merkle_integrity === 1.0 ? "text-emerald-500" : "text-red-500"} />
                        </div>
                        <p className="text-2xl font-bold text-zinc-100">{audit_trail_state.length}</p>
                        <p className={`text-[10px] mt-2 font-mono uppercase tracking-tighter ${quotas_state?.system_defense?.merkle_integrity === 1.0 ? "text-emerald-500" : "text-red-500"}`}>
                            {i18n.t('security.crypto_integrity', { percentage: ((quotas_state?.system_defense?.merkle_integrity || 0) * 100).toFixed(0) })}
                        </p>
                    </div>
                </Tooltip>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {/* Audit Trail */}
                <div className="bg-zinc-900 border border-zinc-800 rounded-2xl flex flex-col h-[400px]">
                    <div className="p-4 border-b border-zinc-800 bg-zinc-900/50 flex items-center justify-between">
                        <div className="flex items-center gap-2">
                            <History className="w-4 h-4 text-emerald-400" />
                            <h2 className="font-semibold text-zinc-100">{i18n.t('security.audit_trail_title')}</h2>
                        </div>
                        <Tooltip content={i18n.t('security.tooltip_merkle_chain')}>
                            <span className="text-[10px] bg-emerald-500/10 text-emerald-400 px-2 py-0.5 rounded border border-emerald-500/20 font-mono uppercase tracking-widest cursor-help">
                                {i18n.t('security.merkle_chain_active')}
                            </span>
                        </Tooltip>
                    </div>
                    <div className="overflow-auto flex-1">
                        <table className="w-full text-left text-xs">
                            <thead className="bg-black/20 text-zinc-500 sticky top-0 z-10">
                                <tr>
                                    <th className="p-3 font-medium border-b border-zinc-800">{i18n.t('security.th_decided')}</th>
                                    <th className="p-3 font-medium border-b border-zinc-800">{i18n.t('security.th_agent')}</th>
                                    <th className="p-3 font-medium border-b border-zinc-800">{i18n.t('security.th_skill')}</th>
                                    <th className="p-3 font-medium border-b border-zinc-800">{i18n.t('security.th_status')}</th>
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-zinc-800/50">
                                {audit_trail_state.map(entry => (
                                    <tr key={entry.id} className="hover:bg-zinc-800/20 transition-colors group">
                                        <td className="p-3 text-zinc-500 font-mono">
                                            {entry.decided_at ? new Date(entry.decided_at).toLocaleTimeString() : i18n.t('security.status_pending')}
                                        </td>
                                        <td className="p-3 text-zinc-300 font-medium">
                                            {entry.agent_id}
                                        </td>
                                        <td className="p-3 text-green-400 font-mono">
                                            {entry.skill || 'â€”'}
                                        </td>
                                        <td className="p-3">
                                            <div className="flex items-center gap-2">
                                                {entry.is_verified ? (
                                                    <CheckCircle2 size={12} className="text-emerald-500" />
                                                ) : (
                                                    <AlertCircle size={12} className="text-red-500" />
                                                )}
                                                <span className={`uppercase font-bold tracking-tighter ${entry.is_verified ? 'text-emerald-500' : 'text-red-500'}`}>
                                                    {entry.is_verified ? (entry.decision || i18n.t('security.status_recorded')) : i18n.t('security.status_recorded')}
                                                </span>
                                            </div>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </div>

                {/* Agent Health Monitoring */}
                <div className="bg-zinc-900 border border-zinc-800 rounded-2xl flex flex-col h-[400px]">
                    <div className="p-4 border-b border-zinc-800 bg-zinc-900/50 flex items-center justify-between">
                        <div className="flex items-center gap-2">
                            <Activity className="w-4 h-4 text-amber-400" />
                            <h2 className="font-semibold text-zinc-100">{i18n.t('security.swarm_health_monitor')}</h2>
                        </div>
                    </div>
                    <div className="p-4 grid grid-cols-1 gap-3 overflow-auto">
                        {agent_health_state.map(a => (
                            <div key={a.agent_id} className={`p-4 rounded-xl border flex items-center justify-between ${a.is_healthy ? 'bg-zinc-950/50 border-zinc-800' : 'bg-red-500/5 border-red-500/20 ring-1 ring-red-500/10'}`}>
                                <div className="flex items-center gap-3">
                                    <div className={`w-10 h-10 rounded-xl border flex items-center justify-center font-bold ${a.is_healthy ? 'bg-zinc-900 border-zinc-800 text-zinc-300' : 'bg-red-500/10 border-red-500/20 text-red-400'}`}>
                                        {a.name.charAt(0)}
                                    </div>
                                    <div>
                                        <h3 className="text-sm font-bold text-zinc-200">{a.name}</h3>
                                        <p className="text-[10px] text-zinc-500 font-mono">{a.agent_id}</p>
                                    </div>
                                </div>
                                <div className="flex flex-col items-end gap-1">
                                    <div className="flex items-center gap-1.5">
                                        <span className={`w-2 h-2 rounded-full ${a.is_healthy ? 'bg-emerald-500' : 'bg-red-500'}`} />
                                        <span className={`text-[10px] font-bold uppercase ${a.is_healthy ? 'text-zinc-500' : 'text-red-400'}`}>
                                            {i18n.t('security.failures_label', { count: a.failure_count })}
                                        </span>
                                    </div>
                                    {a.is_throttled && (
                                        <span className="text-[9px] bg-red-500/20 text-red-500 px-2 py-0.5 rounded-full font-bold uppercase tracking-widest">
                                            {i18n.t('security.throttled')}
                                        </span>
                                    )}
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            </div>

            {/* Periodic Resource Quotas (BudgetGuard) */}
            <div className="bg-zinc-900 border border-zinc-800 rounded-2xl flex flex-col overflow-hidden">
                <div className="p-4 border-b border-zinc-800 bg-zinc-900/50 flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <DollarSign className="w-4 h-4 text-green-400" />
                        <h2 className="font-semibold text-zinc-100">{i18n.t('security.resource_quotas')}</h2>
                    </div>
                    <div className="flex items-center gap-2">
                        <button 
                            onClick={() => set_sort_config({ key: 'name', direction: sort_config.direction === 'asc' ? 'desc' : 'asc' })}
                            className={`px-3 py-1 rounded text-[10px] font-bold uppercase transition-colors flex items-center gap-1 ${sort_config.key === 'name' ? 'bg-green-500/20 text-green-400' : 'bg-zinc-800 text-zinc-500 hover:bg-zinc-700'}`}
                        >
                            <ArrowUpDown size={10} /> {i18n.t('security.sort_name')}
                        </button>
                        <button 
                            onClick={() => set_sort_config({ key: 'status', direction: sort_config.direction === 'asc' ? 'desc' : 'asc' })}
                            className={`px-3 py-1 rounded text-[10px] font-bold uppercase transition-colors flex items-center gap-1 ${sort_config.key === 'status' ? 'bg-green-500/20 text-green-400' : 'bg-zinc-800 text-zinc-500 hover:bg-zinc-700'}`}
                        >
                            <ArrowUpDown size={10} /> {i18n.t('security.sort_status')}
                        </button>
                        <button 
                            onClick={() => set_sort_config({ key: 'quota', direction: sort_config.direction === 'asc' ? 'desc' : 'asc' })}
                            className={`px-3 py-1 rounded text-[10px] font-bold uppercase transition-colors flex items-center gap-1 ${sort_config.key === 'quota' ? 'bg-green-500/20 text-green-400' : 'bg-zinc-800 text-zinc-500 hover:bg-zinc-700'}`}
                        >
                            <ArrowUpDown size={10} /> {i18n.t('security.sort_quota')}
                        </button>
                    </div>
                </div>
                <div className="p-4 grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
                    {get_sorted_quotas().map(q => {
                        const health = agent_health_state.find(h => h.agent_id === q.entity_id);
                        const is_exceeded = q.used_usd >= q.budget_usd;
                        return (
                            <div key={q.entity_id} className={`p-4 rounded-xl border flex flex-col gap-4 ${is_exceeded ? 'bg-red-500/5 border-red-500/30' : 'bg-zinc-950/50 border-zinc-800'}`}>
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center gap-3">
                                        <div className={`w-8 h-8 rounded-lg flex items-center justify-center font-bold text-xs ${health?.is_healthy ? 'bg-emerald-500/10 text-emerald-500 border border-emerald-500/20' : 'bg-zinc-800 text-zinc-500 border border-zinc-700'}`}>
                                            {q.entity_id.charAt(0).toUpperCase()}
                                        </div>
                                        <div>
                                            <h3 className="text-sm font-bold text-zinc-200">{health?.name || q.entity_id}</h3>
                                            <div className="flex items-center gap-2">
                                                <span className={`w-1.5 h-1.5 rounded-full ${health?.status === 'active' ? 'bg-emerald-500 animate-pulse' : 'bg-zinc-600'}`} />
                                                <p className="text-[10px] text-zinc-500 font-mono uppercase">{health?.status || 'offline'}</p>
                                            </div>
                                        </div>
                                    </div>
                                    <div className="text-right">
                                        <p className={`text-xs font-bold font-mono ${is_exceeded ? 'text-red-500' : 'text-zinc-300'}`}>
                                            ${(q.used_usd || 0).toFixed(3)} / ${(q.budget_usd || 0).toFixed(2)}
                                        </p>
                                        <p className="text-[9px] text-zinc-500 uppercase font-bold tracking-tighter">
                                            {i18n.t('security.reset_label', { period: q.reset_period })}
                                        </p>
                                    </div>
                                </div>
                                
                                <div className="space-y-2">
                                    <div className="h-1.5 bg-zinc-900 rounded-full overflow-hidden border border-zinc-800">
                                        <div 
                                            className={`h-full transition-all duration-500 ${is_exceeded ? 'bg-red-500' : (q.used_usd/q.budget_usd > 0.8 ? 'bg-amber-500' : 'bg-green-500')}`}
                                            style={{ width: `${Math.min((q.used_usd / q.budget_usd) * 100, 100)}%` }}
                                        />
                                    </div>
                                    <div className="flex justify-between items-center">
                                        <span className="text-[10px] text-zinc-500 font-mono">{i18n.t('security.usage_label', { percentage: (((q.used_usd || 0) / (q.budget_usd || 1)) * 100).toFixed(1) })}</span>
                                        <div className="flex items-center gap-2">
                                            <Tooltip content={i18n.t('security.tooltip_quota_decrease')}>
                                                <button 
                                                    disabled={is_updating_quota === q.entity_id || q.budget_usd <= 0.1}
                                                    onClick={() => handle_update_quota(q.entity_id, q.budget_usd, -0.5)}
                                                    className="p-1.5 rounded-lg bg-zinc-900 border border-zinc-800 text-zinc-400 hover:text-red-400 hover:border-red-500/30 disabled:opacity-50 transition-all shadow-sm"
                                                >
                                                    <Minus size={12} />
                                                </button>
                                            </Tooltip>
                                            <Tooltip content={i18n.t('security.tooltip_quota_increase')}>
                                                <button 
                                                    disabled={is_updating_quota === q.entity_id}
                                                    onClick={() => handle_update_quota(q.entity_id, q.budget_usd, 0.5)}
                                                    className="p-1.5 rounded-lg bg-zinc-900 border border-zinc-800 text-zinc-400 hover:text-emerald-400 hover:border-emerald-500/30 disabled:opacity-50 transition-all shadow-sm flex items-center gap-1 pr-2"
                                                >
                                                    <Plus size={12} />
                                                    <span className="text-[10px] font-bold">+$0.50</span>
                                                </button>
                                            </Tooltip>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        );
                    })}
                </div>
            </div>

            {/* Proactive Defense Matrix */}
            <div className="bg-zinc-900 border border-green-500/30 rounded-2xl overflow-hidden shadow-[0_0_30px_rgba(59,130,246,0.1)]">
                <div className="bg-green-500/10 p-4 border-b border-green-500/20 flex items-center gap-2">
                    <ShieldCheck className="w-4 h-4 text-green-400" />
                    <h2 className="font-semibold text-blue-100 uppercase tracking-widest text-xs">{i18n.t('security.defense_matrix')}</h2>
                </div>
                <div className="p-6">
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        <div className="space-y-4">
                            <h3 className="text-xs font-bold text-zinc-500 uppercase flex items-center gap-2">
                                <Lock size={12} /> {i18n.t('security.resource_guard')}
                            </h3>
                            <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800">
                                <div className="flex justify-between items-center mb-1">
                                    <span className="text-xs text-zinc-400">{i18n.t('security.memory_pressure')}</span>
                                    <span className={`text-[10px] font-mono ${ (quotas_state?.system_defense?.memory_pressure || 0) > 0.8 ? 'text-red-500' : 'text-emerald-500'}`}>
                                        {((quotas_state?.system_defense?.memory_pressure || 0) * 100).toFixed(1)}%
                                    </span>
                                </div>
                                <div className="h-1 bg-zinc-800 rounded-full">
                                    <div 
                                        className={`h-full transition-all duration-1000 ${ (quotas_state?.system_defense?.memory_pressure || 0) > 0.8 ? 'bg-red-500' : 'bg-emerald-500'}`} 
                                        style={{ width: `${(quotas_state?.system_defense?.memory_pressure || 0) * 100}%` }}
                                    />
                                </div>
                            </div>
                        </div>

                        <div className="space-y-4">
                            <h3 className="text-xs font-bold text-zinc-500 uppercase flex items-center gap-2">
                                <AlertCircle size={12} /> {i18n.t('security.capability_bounds')}
                            </h3>
                            <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800">
                                <div className="flex justify-between items-center mb-1">
                                    <span className="text-xs text-zinc-400">{i18n.t('security.environment')}</span>
                                    <span className={`text-[10px] font-mono ${quotas_state?.system_defense?.sandbox_status === 'ACTIVE' ? 'text-green-500' : 'text-amber-500'}`}>
                                        {quotas_state?.system_defense?.sandbox_type || 'Unknown'}
                                    </span>
                                </div>
                                <div className="h-1 bg-zinc-800 rounded-full">
                                    <div 
                                        className={`h-full ${quotas_state?.system_defense?.sandbox_status === 'ACTIVE' ? 'bg-green-500/30' : 'bg-amber-500/30'}`} 
                                        style={{ width: quotas_state?.system_defense?.sandbox_status === 'ACTIVE' ? '100%' : '50%' }}
                                    />
                                </div>
                            </div>
                        </div>

                        <div className="space-y-4">
                            <h3 className="text-xs font-bold text-zinc-500 uppercase flex items-center gap-2">
                                <ShieldCheck size={12} /> {i18n.t('security.shell_safety')}
                            </h3>
                            <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800">
                                <div className="flex justify-between items-center mb-1">
                                    <span className="text-xs text-zinc-400">{i18n.t('security.secret_leak_prevention')}</span>
                                    <span className="text-[10px] text-emerald-500 font-mono">{i18n.t('security.enabled')}</span>
                                </div>
                                <div className="h-1 bg-zinc-800 rounded-full">
                                    <div className="w-[45%] h-full bg-emerald-500" />
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}


// Metadata: [Security_Dashboard]
