/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Fiscal and ethical oversight controller for agent nodes. 
 * Monitors budget utilization, manages "Requires Oversight" flagging, and visualizes cost-to-limit ratios with high-fidelity progress bars.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Progress bar overflow (utilization > 100%), budget update rejection due to invalid numeric input, or `status_breached` indicator desync with backend enforcement.
 * - **Telemetry Link**: Search for `[Governance_Section]` or `budget_breached` in monitoring logs.
 */

import { useState } from 'react';
import { Info, Shield, ShieldAlert, TrendingUp } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

interface GovernanceSectionProps {
    budget_usd: number;
    requires_oversight: boolean;
    cost_usd: number;
    theme_color: string;
    onUpdateGovernance: (field: 'budget_usd' | 'requires_oversight', value: number | boolean) => void;
}

/**
 * Governance_Section
 * Handles agent fiscal limits and oversight requirements.
 * Ensures strict compliance with budgetary constraints and ethical gating.
 */
export function GovernanceSection({ 
    budget_usd, 
    requires_oversight, 
    cost_usd, 
    theme_color, 
    onUpdateGovernance 
}: GovernanceSectionProps) {
    const [local_budget, set_local_budget] = useState(budget_usd.toString());
    const [prev_budget_usd, set_prev_budget_usd] = useState(budget_usd);

    if (budget_usd !== prev_budget_usd) {
        set_prev_budget_usd(budget_usd);
        set_local_budget(budget_usd.toString());
    }

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
                className={`relative group overflow-hidden bg-zinc-900/40 border border-zinc-800/50 rounded-2xl p-6 transition-all duration-300 ${requires_oversight ? 'border-amber-500/30 bg-amber-500/5' : 'hover:border-zinc-700'}`}
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
                                        Active
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

            {/* Fiscal Controls */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* Budget Set */}
                <div className="bg-zinc-900/30 border border-zinc-800/50 rounded-2xl p-6 space-y-4">
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
                            placeholder="0.00"
                        />
                    </div>
                    <p className="text-[9px] text-zinc-600 font-mono uppercase tracking-[0.1em]">
                        {i18n.t('agent_config.aria_budget_limit')}
                    </p>
                </div>

                {/* Status Card */}
                <div className="bg-zinc-900/30 border border-zinc-800/50 rounded-2xl p-6 flex flex-col justify-between">
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
            <div className="bg-zinc-900/30 border border-zinc-800/50 rounded-2xl p-6 space-y-4">
                <div className="flex items-center justify-between">
                    <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em]">
                        {i18n.t('agent_config.budget_utilization')}
                    </label>
                    <span className="text-xs font-mono text-zinc-400">
                        ${cost_usd.toFixed(4)} <span className="text-zinc-600 mx-1">/</span> ${budget_usd.toFixed(2)}
                    </span>
                </div>

                <div className="relative h-3 w-full bg-zinc-950 rounded-full overflow-hidden border border-zinc-800/50">
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
                    <span>Baseline: $0.00</span>
                    <span>{utilization.toFixed(1)}% Usage</span>
                    <span>Cap: ${budget_usd.toFixed(2)}</span>
                </div>
            </div>
        </div>
    );
}

// Metadata: [Governance_Section]

// Metadata: [Governance_Section]

// Metadata: [GovernanceSection]

// Metadata: [GovernanceSection]
