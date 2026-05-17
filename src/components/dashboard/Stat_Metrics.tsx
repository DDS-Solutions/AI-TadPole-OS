/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: High-fidelity metric scorecard for system health. 
 * Visualizes "Active Nodes", "Aggregated Tokens", and "Governance Status" using normalized data streams from the `agent_store`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Numeric overflow for large token sums, or desync between the "Active Node" count and the visible grid.
 * - **Telemetry Link**: Search for `[Stat_Metrics]` or `get_total_cost` in service logs.
 */

import React from 'react';
import { Users, Zap, Clock, Database, UserPlus } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

interface StatMetricsProps {
    active_agents: number;
    online_count: number;
    total_cost: number;
    total_tokens: number;
    total_input_tokens?: number;
    total_output_tokens?: number;
    total_budget?: number;
    budget_util: number;
    recruit_velocity: number;
}

export const Stat_Metrics: React.FC<StatMetricsProps> = ({
    active_agents, online_count, total_cost, total_tokens, 
    total_input_tokens, total_output_tokens, total_budget,
    budget_util, recruit_velocity
}): React.ReactElement => {
    const format_tokens = (val: number) => {
        if (val >= 1000000) return `${(val / 1000000).toFixed(2)}M`;
        if (val >= 1000) return `${(val / 1000).toFixed(1)}k`;
        return `${val}`;
    };

    const stats = React.useMemo(() => [
        { 
            label: i18n.t('stats.active_swarm'), 
            value: `${active_agents}/${online_count}`, 
            icon: Users, 
            color: active_agents > 0 ? 'text-blue-400' : 'text-zinc-500', 
            bg: active_agents > 0 ? 'bg-blue-900/10' : 'bg-[color:var(--color-surface)]/10', 
            tooltip: i18n.t('stats.active_swarm_tooltip')
        },
        { 
            label: i18n.t('stats.swarm_cost'), 
            value: total_cost > 1000 
                ? `${i18n.t('agent_config.label_currency_symbol')}${(total_cost / 1000).toFixed(2)}k` 
                : `${i18n.t('agent_config.label_currency_symbol')}${total_cost.toFixed(total_cost < 1 ? 3 : 2)}`, 
            icon: Zap, 
            color: 'text-green-500', 
            bg: 'bg-blue-900/10', 
            tooltip: `${i18n.t('stats.swarm_cost_tooltip')} [Spend: ${i18n.t('agent_config.label_currency_symbol')}${total_cost.toFixed(3)} / Budget: ${i18n.t('agent_config.label_currency_symbol')}${(total_budget || 0).toFixed(2)}]`
        },
        { 
            label: i18n.t('stats.swarm_capacity'), 
            value: `${budget_util.toFixed(1)}%`, 
            icon: Clock, 
            color: budget_util > 90 ? 'text-red-400' : budget_util > 50 ? 'text-amber-400' : 'text-green-400', 
            bg: budget_util > 90 ? 'bg-red-900/10' : budget_util > 50 ? 'bg-amber-900/10' : 'bg-green-900/10', 
            tooltip: `${i18n.t('stats.swarm_capacity_tooltip')} [${i18n.t('agent_config.label_currency_symbol')}${total_cost.toFixed(2)} / ${i18n.t('agent_config.label_currency_symbol')}${(total_budget || 0).toFixed(2)}]`
        },
        { 
            label: i18n.t('stats.recruitment_velocity'), 
            value: `${recruit_velocity}`, 
            icon: UserPlus, 
            color: 'text-cyan-400', 
            bg: 'bg-cyan-900/10', 
            tooltip: i18n.t('stats.recruitment_velocity_tooltip')
        },
        { 
            label: i18n.t('stats.swarm_tokens'), 
            value: format_tokens(total_tokens), 
            icon: Database, 
            color: 'text-emerald-400', 
            bg: 'bg-emerald-900/10', 
            tooltip: `${i18n.t('stats.swarm_tokens_tooltip')} [In: ${format_tokens(total_input_tokens || 0)} / Out: ${format_tokens(total_output_tokens || 0)}]`
        },
    ], [active_agents, online_count, total_cost, total_tokens, total_input_tokens, total_output_tokens, total_budget, budget_util, recruit_velocity]);

    return (
        <div className="grid grid-cols-1 md:grid-cols-5 gap-4 flex-shrink-0">
            {stats.map((stat): React.ReactElement => (
                <Tooltip key={stat.label} content={stat.tooltip} position="top">
                    <div className="sovereign-card flex items-center justify-between relative overflow-hidden group">
                        <div className="neural-grid opacity-[0.03]" />
                        <div className="relative z-10">
                            <div className="sovereign-header-text !text-zinc-500 mb-1">{stat.label}</div>
                            <div className={`text-2xl font-bold mt-1 ${stat.color} font-mono`}>{stat.value}</div>
                        </div>
                        <div className={`relative z-10 p-2.5 rounded-lg ${stat.bg} ${stat.color} border border-white/5 transition-all duration-300 group-hover:scale-110 ${stat.label === i18n.t('stats.swarm_tokens') ? 'shadow-[0_0_15px_rgba(16,185,129,0.1)] group-hover:shadow-[0_0_20px_rgba(16,185,129,0.2)]' : ''}`}>
                            <stat.icon size={20} />
                        </div>
                    </div>
                </Tooltip>
            ))}
        </div>
    );
};


// Metadata: [Stat_Metrics]
