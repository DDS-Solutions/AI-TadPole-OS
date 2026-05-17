/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Oversight_Stats]` in observability traces.
 */

import React from 'react';
import { Clock, CheckCircle, XCircle, Shield, WifiOff, ShieldCheck } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

interface OversightStatsProps {
    stats: { pending: number; approved: number; rejected: number };
    is_online: boolean;
    handle_kill_switch: () => void;
    handle_kill_engine: () => void;
    on_navigate_security: () => void;
}

export const Oversight_Stats: React.FC<OversightStatsProps> = ({ 
    stats, 
    is_online, 
    handle_kill_switch, 
    handle_kill_engine,
    on_navigate_security
}) => {
    return (
        <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
            <Tooltip content={i18n.t('oversight.pending_actions_tooltip')} position="top" class_name="w-full">
                <div className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] p-4 rounded-lg flex items-center justify-between cursor-help w-full">
                    <div>
                        <p className="text-zinc-400 text-sm">{i18n.t('oversight.pending_actions_label')}</p>
                        <p className="text-2xl font-bold text-yellow-400">{stats.pending}</p>
                    </div>
                    <Clock className="w-8 h-8 text-yellow-500/20" />
                </div>
            </Tooltip>
            <Tooltip content={i18n.t('oversight.approved_tooltip')} position="top" class_name="w-full">
                <div className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] p-4 rounded-lg flex items-center justify-between cursor-help w-full">
                    <div>
                        <p className="text-zinc-400 text-sm">{i18n.t('oversight.approved_label')}</p>
                        <p className="text-2xl font-bold text-green-400">{stats.approved}</p>
                    </div>
                    <CheckCircle className="w-8 h-8 text-green-500/20" />
                </div>
            </Tooltip>
            <Tooltip content={i18n.t('oversight.rejected_tooltip')} position="top" class_name="w-full">
                <div className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] p-4 rounded-lg flex items-center justify-between cursor-help w-full">
                    <div>
                        <p className="text-zinc-400 text-sm">{i18n.t('oversight.rejected_label')}</p>
                        <p className="text-2xl font-bold text-red-400">{stats.rejected}</p>
                    </div>
                    <XCircle className="w-8 h-8 text-red-500/20" />
                </div>
            </Tooltip>
            <Tooltip content={i18n.t('oversight.halt_agents_tooltip')} position="top" class_name="w-full">
                <button
                    onClick={handle_kill_switch}
                    className={`p-4 rounded-lg flex items-center justify-center gap-2 font-bold transition-colors cursor-pointer group border w-full ${is_online ? 'bg-amber-500/10 hover:bg-amber-500/20 border-amber-500/50 text-amber-400' : 'bg-zinc-800/10 border-zinc-700/50 text-zinc-600 opacity-50'}`}
                    disabled={!is_online}
                >
                    <Shield className="w-5 h-5 group-hover:scale-110 transition-transform" />
                    {is_online ? i18n.t('oversight.halt_agents_button') : i18n.t('oversight.offline_label')}
                </button>
            </Tooltip>
            <div className="flex flex-col gap-2">
                <Tooltip content={i18n.t('oversight.kill_engine_tooltip')} position="top" class_name="w-full">
                    <button
                        onClick={handle_kill_engine}
                        className={`p-4 rounded-lg flex items-center justify-center gap-2 font-bold transition-colors cursor-pointer group border w-full ${is_online ? 'bg-red-600/10 hover:bg-red-600/20 border-red-600/50 text-red-500' : 'bg-zinc-800/10 border-zinc-700/50 text-zinc-600 opacity-50'}`}
                        disabled={!is_online}
                    >
                        <WifiOff className="w-5 h-5 group-hover:scale-110 transition-transform" />
                        {is_online ? i18n.t('oversight.kill_engine_button') : i18n.t('oversight.offline_label')}
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('oversight.security_dashboard_tooltip')} position="top" class_name="w-full">
                    <button
                        onClick={on_navigate_security}
                        className="w-full p-2 bg-[color:var(--color-background)] border border-zinc-700 hover:border-green-500/50 hover:bg-green-500/5 text-zinc-300 hover:text-green-400 rounded-lg flex items-center justify-center gap-2 font-bold transition-all group"
                    >
                        <ShieldCheck className="w-4 h-4 group-hover:scale-110 transition-transform text-green-500" />
                        <span className="text-[10px] uppercase tracking-wider">{i18n.t('oversight.security_dashboard_button')}</span>
                    </button>
                </Tooltip>
            </div>
        </div>
    );
};

// Metadata: [Oversight_Stats]
