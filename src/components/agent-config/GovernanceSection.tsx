/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Agent-Config / GovernanceSection
 * - **Primary Entrypoints**: `GovernanceSection`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useState } from 'react';
import { Info, Shield, ShieldAlert, TrendingUp, Coins, Wallet, ShoppingBag } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

interface GovernanceSectionProps {
    budget_usd: number;
    requires_oversight: boolean;
    shadows_human_id?: string;
    economic_zone: string;
    daily_spend_limit: number;
    daily_spent_accumulated: number;
    balance: number;
    inventory: Array<{
        asset_id: string;
        asset_name: string;
        asset_data?: string;
    }>;
    cost_usd: number;
    theme_color: string;
    onUpdateGovernance: (field: 'budget_usd' | 'requires_oversight' | 'economic_zone' | 'daily_spend_limit' | 'shadows_human_id', value: number | boolean | string) => void;
}

/**
 * Governance_Section
 * Handles agent fiscal limits and oversight requirements.
 * Ensures strict compliance with budgetary constraints and ethical gating.
 */
export function GovernanceSection({ 
    budget_usd, 
    requires_oversight, 
    shadows_human_id,
    economic_zone,
    daily_spend_limit,
    daily_spent_accumulated,
    balance,
    inventory,
    cost_usd,
    theme_color, 
    onUpdateGovernance 
}: GovernanceSectionProps) {
    const [local_budget, set_local_budget] = useState(budget_usd.toString());
    const [local_daily_limit, set_local_daily_limit] = useState((daily_spend_limit / 1000000).toString());

    const is_breached = cost_usd >= budget_usd && budget_usd > 0;
    const utilization = budget_usd > 0 ? (cost_usd / budget_usd) * 100 : 0;
    
    // Dynamic colors based on utilization
    const status_color = is_breached ? '#ef4444' : (utilization > 80 ? '#f59e0b' : theme_color);

    return (
        <div className="flex-1 overflow-y-auto custom-scrollbar p-6 space-y-8 animate-in fade-in slide-in-from-bottom-2 duration-500">
            {/* Header / Identity Sector */}
            <div className="space-y-1">
                <h3 className="text-xs font-bold text-zinc-400 uppercase tracking-[0.3em]">
                    {i18n.t('agent_config.tab_governance')}
                </h3>
                <p className="text-[10px] text-zinc-500 font-mono uppercase tracking-wider">
                    {i18n.t('agent_config.tooltip_governance')}
                </p>
            </div>

            {/* Neural Oversight Gate */}
            <div 
                className={`relative group overflow-hidden bg-[color:var(--color-surface)]/40 border border-[color:var(--color-border)]/50 rounded-2xl p-6 transition-all duration-300 ${requires_oversight ? 'border-amber-500/30 bg-amber-500/5' : 'hover:border-zinc-700'}`}
            >
                <div className="relative z-10 flex items-start justify-between gap-6">
                    <div className="flex items-start gap-4">
                        <div className={`p-3 rounded-xl transition-colors ${requires_oversight ? 'bg-amber-500/20 text-amber-500' : 'bg-zinc-800 text-zinc-500'}`}>
                            {requires_oversight ? <ShieldAlert size={20} /> : <Shield size={20} />}
                        </div>
                        <div className="space-y-1">
                            <h4 className="text-sm font-bold text-zinc-200 tracking-tight flex items-center gap-2">
                                {i18n.t('agent_config.label_oversight_gate')}
                                {requires_oversight && (
                                    <span className="text-[9px] px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-500 uppercase tracking-widest font-black">
                                        {i18n.t('common.active')}
                                    </span>
                                )}
                            </h4>
                            <p className="text-xs text-zinc-500 leading-relaxed max-w-sm">
                                {i18n.t('agent_config.desc_oversight_gate')}
                            </p>
                        </div>
                    </div>

                    <button
                        onClick={() => onUpdateGovernance('requires_oversight', !requires_oversight)}
                        className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-amber-500 focus:ring-offset-2 focus:ring-offset-zinc-900 ${requires_oversight ? 'bg-amber-600' : 'bg-zinc-800'}`}
                    >
                        <span
                            aria-hidden="true"
                            className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${requires_oversight ? 'translate-x-5' : 'translate-x-0'}`}
                        />
                    </button>
                </div>
                {/* Visual grid background for premium feel */}
                <div className="absolute inset-0 opacity-[0.03] pointer-events-none neural-grid" />
            </div>

            {/* Human Identity Mapping (ISO 42001 Gap #2) */}
            <div className="bg-[color:var(--color-surface)]/40 border border-[color:var(--color-border)]/50 rounded-2xl p-6 transition-all duration-300 hover:border-zinc-700 space-y-3">
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                        <div className="p-2.5 rounded-xl bg-blue-500/10 text-blue-400">
                            <Shield size={18} />
                        </div>
                        <div>
                            <h4 className="text-xs font-bold text-zinc-200 uppercase tracking-wider flex items-center gap-2">
                                Shadows Real Person
                                <span className="text-[9px] px-1.5 py-0.5 rounded bg-blue-500/10 text-blue-400 font-mono">
                                    ISO 42001
                                </span>
                            </h4>
                            <p className="text-[10px] text-zinc-500 font-mono">
                                Map agentic tasks to a specific human user ID for accountability and audit logs.
                            </p>
                        </div>
                    </div>
                </div>
                <input
                    type="text"
                    aria-label="Shadows Real Person"
                    value={shadows_human_id || ''}
                    onChange={(e) => onUpdateGovernance('shadows_human_id', e.target.value)}
                    placeholder="e.g. usr_human_admin_01"
                    className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-xl px-4 py-2.5 text-xs text-zinc-200 font-mono placeholder:text-zinc-600 focus:outline-none focus:border-blue-500/50"
                />
            </div>

            {/* Fiscal Controls */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* Budget Set */}
                <div className="bg-[color:var(--color-surface)]/30 border border-[color:var(--color-border)]/50 rounded-2xl p-6 space-y-4">
                    <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                        <TrendingUp size={12} className="text-zinc-600" />
                        {i18n.t('agent_config.budget_limit')}
                        <Tooltip content={i18n.t('agent_config.tooltip_budget')}>
                            <Info size={10} className="text-zinc-700 hover:text-zinc-300 transition-colors cursor-help" />
                        </Tooltip>
                    </label>

                    <div className="flex items-end gap-2 group">
                        <span className="text-2xl font-mono text-zinc-600 mb-1">$</span>
                        <input
                            type="text"
                            inputMode="decimal"
                            value={local_budget}
                            onChange={(e) => set_local_budget(e.target.value)}
                            onBlur={() => onUpdateGovernance('budget_usd', parseFloat(local_budget) || 0)}
                            className="bg-transparent border-none p-0 text-4xl font-mono font-bold text-zinc-100 focus:ring-0 w-full placeholder:text-zinc-800"
                            placeholder={i18n.t('common_units.placeholder_budget')}
                        />
                    </div>
                    <p className="text-[9px] text-zinc-600 font-mono uppercase tracking-[0.1em]">
                        {i18n.t('agent_config.aria_budget_limit')}
                    </p>
                </div>

                {/* Status Card */}
                <div className="bg-[color:var(--color-surface)]/30 border border-[color:var(--color-border)]/50 rounded-2xl p-6 flex flex-col justify-between">
                    <div className="space-y-1">
                        <span className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em]">
                            {i18n.t('agent_config.label_nominal_status')}
                        </span>
                        <div className="flex items-center gap-2">
                            <div 
                                className="w-2 h-2 rounded-full animate-pulse" 
                                style={{ 
                                    backgroundColor: status_color,
                                    boxShadow: `0 0 12px ${status_color}80`
                                }} 
                            />
                            <span 
                                className="text-lg font-black uppercase tracking-widest"
                                style={{ color: status_color }}
                            >
                                {is_breached ? i18n.t('agent_config.status_breached') : i18n.t('agent_config.status_nominal')}
                            </span>
                        </div>
                    </div>

                    <p className="text-[10px] text-zinc-500 italic">
                        {is_breached ? i18n.t('agent_config.status_breached_desc') : i18n.t('agent_config.status_nominal_desc')}
                    </p>
                </div>
            </div>

            {/* Budget Utilization Progress */}
            <div className="bg-[color:var(--color-surface)]/30 border border-[color:var(--color-border)]/50 rounded-2xl p-6 space-y-4">
                <div className="flex items-center justify-between">
                    <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em]">
                        {i18n.t('agent_config.budget_utilization')}
                    </label>
                    <span className="text-xs font-mono text-zinc-400">
                        ${cost_usd.toFixed(4)} <span className="text-zinc-600 mx-1">/</span> ${budget_usd.toFixed(2)}
                    </span>
                </div>

                <div className="relative h-3 w-full bg-[color:var(--color-background)] rounded-full overflow-hidden border border-[color:var(--color-border)]/50">
                    {/* Background track glass effect */}
                    <div className="absolute inset-0 opacity-[0.02] neural-grid" />
                    
                    {/* Progress Fill */}
                    <div 
                        className="absolute inset-y-0 left-0 transition-all duration-700 ease-out"
                        style={{ 
                            width: `${Math.min(utilization, 100)}%`,
                            backgroundColor: status_color,
                            boxShadow: `0 0 20px ${status_color}40`
                        }}
                    />

                    {/* Warning overlay at 80% */}
                    {utilization > 80 && !is_breached && (
                        <div className="absolute inset-0 bg-gradient-to-r from-transparent via-amber-500/10 to-transparent" />
                    )}
                </div>

                <div className="flex justify-between items-center text-[9px] font-mono text-zinc-600 uppercase">
                    <span>{i18n.t('agent_config.baseline')}: $0.00</span>
                    <span>{utilization.toFixed(1)}% {i18n.t('agent_config.label_usage')}</span>
                    <span>{i18n.t('agent_config.label_cap')}: ${budget_usd.toFixed(2)}</span>
                </div>
            </div>

            {/* Divider */}
            <div className="h-px bg-[color:var(--color-border)]/50 my-6" />

            {/* Economics & Agent Wallet Section */}
            <div className="space-y-6">
                <div className="space-y-1">
                    <h3 className="text-xs font-bold text-zinc-400 uppercase tracking-[0.3em] flex items-center gap-2">
                        <Coins size={14} className="text-emerald-500" />
                        Agent Economics & Wallet
                    </h3>
                    <p className="text-[10px] text-zinc-500 font-mono uppercase tracking-wider">
                        Configure transactional adapters, daily spend limits, and monitor owned resources.
                    </p>
                </div>

                {/* Economic Zone Selector & Wallet Balance */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    {/* Economic Zone */}
                    <div className="bg-[color:var(--color-surface)]/30 border border-[color:var(--color-border)]/50 rounded-2xl p-6 space-y-4">
                        <label htmlFor="economic-zone-select" className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] block">
                            Active Economic Zone
                        </label>
                        <select
                            id="economic-zone-select"
                            value={economic_zone}
                            onChange={(e) => onUpdateGovernance('economic_zone', e.target.value)}
                            className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)]/50 rounded-xl px-4 py-2.5 text-xs text-zinc-300 focus:outline-none focus:border-emerald-500/50"
                        >
                            <option value="DEV">Dev Zone (Mock local tokens)</option>
                            <option value="STAGING">Staging Zone (Testnet hybrid)</option>
                            <option value="PROD">Prod Zone (Base Mainnet USDC)</option>
                        </select>
                        <p className="text-[9px] text-zinc-600 leading-relaxed">
                            Determines if the agent uses offline simulated tokens or settles on-chain via Coinbase SDK.
                        </p>
                    </div>

                    {/* Wallet Balance */}
                    <div className="bg-[color:var(--color-surface)]/30 border border-[color:var(--color-border)]/50 rounded-2xl p-6 flex flex-col justify-between">
                        <div className="space-y-1">
                            <span className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-1.5">
                                <Wallet size={12} className="text-zinc-400" />
                                Current Balance
                            </span>
                            <div className="text-3xl font-mono font-bold text-emerald-400">
                                ${(balance / 1000000).toFixed(6)}{' '}
                                <span className="text-xs text-zinc-500 font-bold uppercase">USDC</span>
                            </div>
                        </div>
                        <p className="text-[9px] text-zinc-600 uppercase tracking-widest font-mono">
                            {(balance).toLocaleString()} Micros (atomic units)
                        </p>
                    </div>
                </div>

                {/* Daily Spend Limit & Spent Limit Utilization */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    {/* Daily Spend Limit Input */}
                    <div className="bg-[color:var(--color-surface)]/30 border border-[color:var(--color-border)]/50 rounded-2xl p-6 space-y-4">
                        <label htmlFor="daily-spend-limit-input" className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] block">
                            Daily Spend Limit (USDC)
                        </label>
                        <div className="flex items-end gap-2">
                            <span className="text-2xl font-mono text-zinc-600 mb-1">$</span>
                            <input
                                id="daily-spend-limit-input"
                                type="text"
                                inputMode="decimal"
                                value={local_daily_limit}
                                onChange={(e) => set_local_daily_limit(e.target.value)}
                                onBlur={() => {
                                    const val = Math.round((parseFloat(local_daily_limit) || 0) * 1000000);
                                    onUpdateGovernance('daily_spend_limit', val);
                                }}
                                className="bg-transparent border-none p-0 text-3xl font-mono font-bold text-zinc-100 focus:ring-0 w-full placeholder:text-zinc-800"
                                placeholder="0.000000"
                            />
                        </div>
                        <p className="text-[9px] text-zinc-600">
                            Maximum cumulative amount (in USDC) this agent is allowed to transact in 24 hours.
                        </p>
                    </div>

                    {/* Daily Limit Cap Utilization Progress */}
                    <div className="bg-[color:var(--color-surface)]/30 border border-[color:var(--color-border)]/50 rounded-2xl p-6 flex flex-col justify-between space-y-4">
                        <div className="space-y-1">
                            <div className="flex items-center justify-between text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em]">
                                <span>Daily Limit Cap</span>
                                <span className="font-mono text-zinc-400">
                                    ${(daily_spent_accumulated / 1000000).toFixed(4)} / ${(daily_spend_limit / 1000000).toFixed(2)}
                                </span>
                            </div>
                            <div className="relative h-2 w-full bg-[color:var(--color-background)] rounded-full overflow-hidden border border-[color:var(--color-border)]/50 mt-2">
                                <div 
                                    className="absolute inset-y-0 left-0 transition-all duration-700 ease-out"
                                    style={{ 
                                        width: `${daily_spend_limit > 0 ? Math.min((daily_spent_accumulated / daily_spend_limit) * 100, 100) : 0}%`,
                                        backgroundColor: daily_spend_limit > 0 && daily_spent_accumulated >= daily_spend_limit ? '#ef4444' : theme_color,
                                    }}
                                />
                            </div>
                        </div>
                        <p className="text-[9px] text-zinc-600 font-mono">
                            {daily_spend_limit > 0 
                                ? `${((daily_spent_accumulated / daily_spend_limit) * 100).toFixed(1)}% daily budget capacity used.`
                                : 'No daily limit cap enforced.'
                            }
                        </p>
                    </div>
                </div>

                {/* Inventory / Asset Registry */}
                <div className="bg-[color:var(--color-surface)]/30 border border-[color:var(--color-border)]/50 rounded-2xl p-6 space-y-4">
                    <span className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-1.5">
                        <ShoppingBag size={12} className="text-zinc-400" />
                        Owned Resource Inventory
                    </span>
                    {inventory.length === 0 ? (
                        <div className="text-xs text-zinc-600 py-4 italic text-center">
                            This agent does not currently own any registered economic resources or asset scopes.
                        </div>
                    ) : (
                        <div className="space-y-2 max-h-40 overflow-y-auto custom-scrollbar pr-1">
                            {inventory.map((item, idx) => (
                                <div 
                                    key={item.asset_id || idx} 
                                    className="flex items-center justify-between p-3 bg-[color:var(--color-background)]/50 border border-[color:var(--color-border)]/50 rounded-xl hover:border-zinc-700 transition-all"
                                >
                                    <div className="space-y-0.5">
                                        <div className="text-xs font-bold text-zinc-200">{item.asset_name}</div>
                                        <div className="text-[9px] font-mono text-zinc-600">{item.asset_id}</div>
                                    </div>
                                    {item.asset_data && (
                                        <span className="text-[9px] px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-mono">
                                            {item.asset_data}
                                        </span>
                                    )}
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
