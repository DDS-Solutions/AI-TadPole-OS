import React from 'react';
import { Users, Zap, Clock, Database, UserPlus } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

interface StatMetricsProps {
    active_agents: number;
    agents_count: number;
    total_cost: number;
    total_tokens: number;
    budget_util: number;
    recruit_velocity: number;
}

export const Stat_Metrics: React.FC<StatMetricsProps> = ({
    active_agents, agents_count, total_cost, total_tokens, budget_util, recruit_velocity
}): React.ReactElement => {
    const stats = [
        { 
            label: i18n.t('stats.active_swarm'), 
            value: `${active_agents}/${agents_count}`, 
            icon: Users, 
            color: 'text-blue-400', 
            bg: 'bg-blue-900/10', 
            tooltip: i18n.t('stats.active_swarm_tooltip')
        },
        { 
            label: i18n.t('stats.swarm_cost'), 
            value: `${i18n.t('agent_config.label_currency_symbol')}${total_cost.toFixed(3)}`, 
            icon: Zap, 
            color: 'text-blue-500', 
            bg: 'bg-blue-900/10', 
            tooltip: i18n.t('stats.swarm_cost_tooltip')
        },
        { 
            label: i18n.t('stats.swarm_capacity'), 
            value: `${budget_util.toFixed(1)}%`, 
            icon: Clock, 
            color: 'text-amber-400', 
            bg: 'bg-amber-900/10', 
            tooltip: i18n.t('stats.swarm_capacity_tooltip')
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
            value: total_tokens > 1000 ? `${(total_tokens / 1000).toFixed(1)}k` : `${total_tokens}`, 
            icon: Database, 
            color: 'text-emerald-400', 
            bg: 'bg-emerald-900/10', 
            tooltip: i18n.t('stats.swarm_tokens_tooltip')
        },
    ];

    return (
        <div className="grid grid-cols-1 md:grid-cols-5 gap-4 flex-shrink-0">
            {stats.map((stat, i): React.ReactElement => (
                <Tooltip key={i} content={stat.tooltip} position="top">
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
