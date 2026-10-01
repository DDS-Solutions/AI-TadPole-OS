/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / Security_Dashboard
 * - **Primary Entrypoints**: `Security_Dashboard`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { Activity, Zap, Heart, Lock, DollarSign } from 'lucide-react';
import { useState, type FormEvent } from 'react';
import { useSecurityDashboard } from '../hooks/useSecurityDashboard';
import { use_security_store } from '../stores/security_store';
import { Security_Header } from '../components/security/Security_Header';
import { Audit_Trail_Section } from '../components/security/Audit_Trail_Section';
import { Agent_Health_Section } from '../components/security/Agent_Health_Section';
import { Resource_Quotas_Section } from '../components/security/Resource_Quotas_Section';
import { Defense_Matrix_Section } from '../components/security/Defense_Matrix_Section';
import { Tooltip } from '../components/ui';
import { i18n } from '../i18n';

/**
 * Security Dashboard: The central nexus for system-wide governance monitoring.
 */
export default function Security_Dashboard() {
    const imported_public_key = use_security_store(state => state.publicKey);
    const key_source = use_security_store(state => state.key_source);
    const set_key_pair = use_security_store(state => state.set_key_pair);
    const clear_key_pair = use_security_store(state => state.clear_key_pair);
    const [public_key_input, set_public_key_input] = useState(() => imported_public_key ?? '');
    const [private_key_input, set_private_key_input] = useState('');
    const [key_error, set_key_error] = useState<string | null>(null);
    const [is_importing_key, set_is_importing_key] = useState(false);

    const {
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
        sorted_quotas,
        sorted_health
    } = useSecurityDashboard(5000);

    const import_operator_key = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        set_is_importing_key(true);
        set_key_error(null);
        try {
            await set_key_pair(public_key_input.trim(), private_key_input.trim());
            set_private_key_input('');
        } catch (error) {
            set_key_error(error instanceof Error ? error.message : 'Could not import this key pair.');
        } finally {
            set_is_importing_key(false);
        }
    };

    if (is_loading && !quotas) {
        return (
            <div className="flex flex-col items-center justify-center p-20 gap-4">
                <Activity className="animate-spin text-emerald-500 w-10 h-10" />
                <span className="text-zinc-400 font-mono text-sm tracking-widest uppercase">
                    {i18n.t('security.loading')}
                </span>
            </div>
        );
    }

    return (
        <div className="p-6 space-y-6 max-w-7xl mx-auto animate-in fade-in duration-500" aria-label="Security Dashboard">
            {is_stale && (
                <div className="bg-amber-500/10 border border-amber-500/20 p-3 rounded-xl flex items-center gap-3 text-amber-500 text-xs font-bold uppercase tracking-widest">
                    <Activity className="w-4 h-4 animate-pulse" />
                    {i18n.t('security.stale_data_warning', { defaultValue: 'Telemetry Link Interrupted — Showing Stale Data' })}
                </div>
            )}

            <Security_Header 
                agent_health={agent_health} 
                merkle_integrity={quotas?.system_defense?.merkle_integrity ?? 1.0} 
            />

            <section className="sovereign-panel p-5 space-y-4" aria-labelledby="oversight-key-heading">
                <div>
                    <h2 id="oversight-key-heading" className="text-sm font-bold uppercase tracking-widest text-zinc-100">
                        Oversight signing key
                    </h2>
                    <p className="mt-1 text-xs text-zinc-400">
                        Import the operator key pair for this session. The private key stays in memory and must be entered again after reload.
                        The public key must match the server’s configured oversight key.
                    </p>
                </div>

                {key_source === 'operator' && (
                    <div className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-emerald-500/20 bg-emerald-500/5 p-3">
                        <span className="text-xs font-medium text-emerald-400">
                            Operator key loaded for this session · public key ending {imported_public_key?.slice(-8)}
                        </span>
                        <button
                            type="button"
                            onClick={clear_key_pair}
                            className="text-xs font-bold text-zinc-300 underline decoration-zinc-600 underline-offset-4 hover:text-white"
                        >
                            Clear key
                        </button>
                    </div>
                )}

                <form onSubmit={import_operator_key} className="grid grid-cols-1 gap-3 md:grid-cols-2">
                    <label className="space-y-1 text-xs text-zinc-400">
                        <span>Public key · 32-byte hex</span>
                        <input
                            value={public_key_input}
                            onChange={event => set_public_key_input(event.target.value)}
                            autoComplete="off"
                            spellCheck={false}
                            className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-xs text-zinc-100 outline-none focus:border-emerald-500"
                            aria-label="Oversight public key"
                        />
                    </label>
                    <label className="space-y-1 text-xs text-zinc-400">
                        <span>Private key · PKCS#8 hex</span>
                        <input
                            type="password"
                            value={private_key_input}
                            onChange={event => set_private_key_input(event.target.value)}
                            autoComplete="new-password"
                            spellCheck={false}
                            className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-xs text-zinc-100 outline-none focus:border-emerald-500"
                            aria-label="Oversight private key"
                        />
                    </label>
                    <div className="flex flex-wrap items-center gap-3 md:col-span-2">
                        <button
                            type="submit"
                            disabled={is_importing_key || !public_key_input.trim() || !private_key_input.trim()}
                            className="rounded-lg bg-emerald-600 px-4 py-2 text-xs font-bold uppercase tracking-wider text-white hover:bg-emerald-500 disabled:cursor-not-allowed disabled:opacity-50"
                        >
                            {is_importing_key ? 'Checking key pair…' : 'Import for this session'}
                        </button>
                        {key_error && <span role="alert" className="text-xs text-red-400">{key_error}</span>}
                    </div>
                </form>
            </section>

            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                <Tooltip content={i18n.t('security.tooltip_budget_card')}>
                    <div className="sovereign-panel p-4 cursor-help hover:border-emerald-500/30">
                        <div className="flex items-center justify-between mb-2">
                            <p className="text-zinc-400 text-xs font-mono uppercase">{i18n.t('security.budget_consumption')}</p>
                            <DollarSign size={14} className="text-zinc-500" />
                        </div>
                        <p className="text-2xl font-bold text-zinc-100">${(quotas?.total_spent || 0).toFixed(2)}</p>
                        <div className="mt-3 h-1.5 bg-zinc-800 rounded-full overflow-hidden">
                            <div
                                className="h-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.3)]"
                                style={{ width: `${Math.min(quotas?.efficiency || 0, 100)}%` }}
                            />
                        </div>
                        <p className="text-[10px] text-zinc-500 mt-2">
                            {i18n.t('security.efficiency_label', { percentage: (quotas?.efficiency ?? 0).toFixed(1) })}
                        </p>
                    </div>
                </Tooltip>

                <Tooltip content={i18n.t('security.tooltip_agents_card')}>
                    <div className="sovereign-panel p-4 cursor-help hover:border-amber-500/30">
                        <div className="flex items-center justify-between mb-2">
                            <p className="text-zinc-400 text-xs font-mono uppercase">{i18n.t('security.active_agents')}</p>
                            <Zap size={14} className="text-amber-500" />
                        </div>
                        <p className="text-2xl font-bold text-zinc-100">{agent_health.length}</p>
                        <p className="text-[10px] text-zinc-500 mt-2 font-mono">
                            {i18n.t('security.executing_missions', { count: agent_health.filter(a => ['active', 'thinking', 'speaking', 'coding'].includes(a.status)).length })}
                        </p>
                    </div>
                </Tooltip>

                <Tooltip content={i18n.t('security.tooltip_health_card')}>
                    <div className="sovereign-panel p-4 cursor-help hover:border-emerald-500/30">
                        <div className="flex items-center justify-between mb-2">
                            <p className="text-zinc-400 text-xs font-mono uppercase">{i18n.t('security.system_health')}</p>
                            <Heart size={14} className="text-emerald-500" />
                        </div>
                        <p className="text-2xl font-bold text-zinc-100">
                            {i18n.t('security.health_ratio', { healthy: agent_health.filter(a => a.is_healthy).length, total: agent_health.length })}
                        </p>
                        <p className="text-[10px] text-zinc-500 mt-2 font-mono uppercase tracking-tighter">
                            {agent_health.every(a => a.is_healthy) ? i18n.t('security.optimal_ops') : i18n.t('security.degraded_ops')}
                        </p>
                    </div>
                </Tooltip>

                <Tooltip content={i18n.t('security.tooltip_decisions_card')}>
                    <div className="sovereign-panel p-4 cursor-help hover:border-blue-500/30">
                        <div className="flex items-center justify-between mb-2">
                            <p className="text-zinc-400 text-xs font-mono uppercase">{i18n.t('security.verified_decisions')}</p>
                            <Lock size={14} className={quotas?.system_defense?.merkle_integrity === 1.0 ? "text-emerald-500" : "text-red-500"} />
                        </div>
                        <p className="text-2xl font-bold text-zinc-100">{audit_trail.length}</p>
                        <p className={`text-[10px] mt-2 font-mono uppercase tracking-tighter ${quotas?.system_defense?.merkle_integrity === 1.0 ? "text-emerald-500" : "text-red-500"}`}>
                            {i18n.t('security.crypto_integrity', { percentage: ((quotas?.system_defense?.merkle_integrity || 0) * 100).toFixed(0) })}
                        </p>
                    </div>
                </Tooltip>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <Audit_Trail_Section 
                    audit_trail={audit_trail}
                    agent_health={agent_health}
                    audit_page={audit_page}
                    audit_total={audit_total}
                    set_audit_page={set_audit_page}
                />

                <Agent_Health_Section 
                    sorted_health={sorted_health}
                    health_search={health_search}
                    set_health_search={set_health_search}
                    health_sort={health_sort}
                    set_health_sort={set_health_sort}
                />
            </div>

            <Resource_Quotas_Section 
                sorted_quotas={sorted_quotas}
                agent_health={agent_health}
                sort_config={sort_config}
                set_sort_config={set_sort_config}
                is_updating_quota={is_updating_quota}
                handle_update_quota={update_quota}
            />

            <Defense_Matrix_Section 
                system_defense={quotas?.system_defense || { 
                    memory_pressure: 0, 
                    cpu_load: 0, 
                    sandbox_status: 'initializing', 
                    sandbox_type: 'unknown', 
                    merkle_integrity: 1.0 
                }}
            />
        </div>
    );
}
