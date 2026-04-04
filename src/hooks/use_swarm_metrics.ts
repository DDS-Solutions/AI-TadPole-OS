import { use_settings_store } from '../stores/settings_store';
import { use_workspace_store } from '../stores/workspace_store';
import { use_engine_status } from './use_engine_status';
import { Activity, Cpu, Target, Repeat, Zap, DollarSign } from 'lucide-react';

/**
 * useSwarmMetrics
 * Custom hook to aggregate swarm telemetry metrics for the header.
 */
export const use_swarm_metrics = () => {
    const { settings } = use_settings_store();
    const { clusters } = use_workspace_store();
    const { is_online, active_agents: agents_count } = use_engine_status();

    const active_agents_count = clusters.reduce((acc, c) => acc + c.collaborators.length, 0);
    const active_clusters_count = clusters.length;

    return [
        {
            label: 'Active Agents',
            value: `${active_agents_count}/${settings.max_agents}`,
            icon: Cpu,
            tooltip: 'Total agents assigned to active mission clusters vs system capacity.',
            color: 'text-emerald-400'
        },
        {
            label: 'Active Clusters',
            value: `${active_clusters_count}/${settings.max_clusters}`,
            icon: Target,
            tooltip: 'Currently deployed mission clusters vs system limit.',
            color: 'text-blue-400'
        },
        {
            label: 'Nodes Online',
            value: is_online ? agents_count : 'OFFLINE',
            icon: Activity,
            tooltip: 'Real-time telemetry of neural nodes connected to the Tadpole Engine.',
            color: is_online ? 'text-cyan-400' : 'text-zinc-600'
        },
        {
            label: 'Max Depth',
            value: settings.max_swarm_depth,
            icon: Repeat,
            tooltip: 'Maximum recursion depth allowed for autonomous agent delegation.',
            color: 'text-zinc-400'
        },
        {
            label: 'Task Limit',
            value: `${Math.round(settings.max_task_length / 1024)}k`,
            icon: Zap,
            tooltip: 'Maximum context length (tokens) per neural inference cycle.',
            color: 'text-amber-400'
        },
        {
            label: 'Base Budget',
            value: `$${settings.default_budget_usd.toFixed(2)}`,
            icon: DollarSign,
            tooltip: 'Default neural credit allocation for new swarm branches.',
            color: 'text-emerald-500'
        }
    ];
};
