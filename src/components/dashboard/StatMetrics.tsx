import React from 'react';
import { Users, Zap, Clock, Shield, UserPlus } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

interface StatMetricsProps {
    isOnline: boolean;
    activeAgents: number;
    agentsCount: number;
    totalCost: number;
    budgetUtil: number;
    recruitVelocity: number;
}

export const StatMetrics: React.FC<StatMetricsProps> = ({
    isOnline, activeAgents, agentsCount, totalCost, budgetUtil, recruitVelocity
}): React.ReactElement => {
    const stats = [
        { 
            label: i18n.t('stats.active_swarm'), 
            value: `${activeAgents}/${agentsCount}`, 
            icon: Users, 
            color: 'text-blue-400', 
            bg: 'bg-blue-900/10', 
            tooltip: i18n.t('stats.active_swarm_tooltip')
        },
        { 
            label: i18n.t('stats.swarm_cost'), 
            value: `${i18n.t('agent_config.label_currency_symbol')}${totalCost.toFixed(3)}`, 
            icon: Zap, 
            color: 'text-blue-500', 
            bg: 'bg-blue-900/10', 
            tooltip: i18n.t('stats.swarm_cost_tooltip')
        },
        { 
            label: i18n.t('stats.swarm_capacity'), 
            value: `${budgetUtil.toFixed(1)}%`, 
            icon: Clock, 
            color: 'text-amber-400', 
            bg: 'bg-amber-900/10', 
            tooltip: i18n.t('stats.swarm_capacity_tooltip')
        },
        { 
            label: i18n.t('stats.recruitment_velocity'), 
            value: `${recruitVelocity}`, 
            icon: UserPlus, 
            color: 'text-cyan-400', 
            bg: 'bg-cyan-900/10', 
            tooltip: i18n.t('stats.recruitment_velocity_tooltip')
        },
        { 
            label: i18n.t('stats.system_health'), 
            value: isOnline ? i18n.t('common.online') : i18n.t('common.offline'), 
            icon: Shield, 
            color: isOnline ? 'text-emerald-400' : 'text-zinc-600', 
            bg: isOnline ? 'bg-emerald-900/10' : 'bg-zinc-900/10', 
            tooltip: i18n.t('stats.system_health_tooltip')
        },
    ];

    return (
        <div className="grid grid-cols-1 md:grid-cols-5 gap-4 flex-shrink-0">
            {stats.map((stat, i): React.ReactElement => (
                <Tooltip key={i} content={stat.tooltip} position="top">
                    <div className="sovereign-card flex items-center justify-between relative overflow-hidden group">
                        <div className="neural-grid opacity-[0.03]" />
                        <div className="relative z-10">
                            <div className="text-zinc-500 text-[10px] uppercase tracking-wider font-bold">{stat.label}</div>
                            <div className={`text-2xl font-bold mt-1 ${stat.color} font-mono`}>{stat.value}</div>
                        </div>
                        <div className={`relative z-10 p-2.5 rounded-lg ${stat.bg} ${stat.color} border border-white/5 transition-transform duration-300 group-hover:scale-110`}>
                            <stat.icon size={20} />
                        </div>
                    </div>
                </Tooltip>
            ))}
        </div>
    );
};
