import { useSettingsStore } from '../stores/settingsStore';
import { useWorkspaceStore } from '../stores/workspaceStore';
import { useEngineStatus } from './useEngineStatus';
import { Activity, Cpu, Target, Repeat, Zap, DollarSign } from 'lucide-react';

/**
 * useSwarmMetrics
 * Custom hook to aggregate swarm telemetry metrics for the header.
 */
export const useSwarmMetrics = () => {
    const { settings } = useSettingsStore();
    const { clusters } = useWorkspaceStore();
    const { isOnline, activeAgents: agentsCount } = useEngineStatus();

    const activeAgentsCount = clusters.reduce((acc, c) => acc + c.collaborators.length, 0);
    const activeClustersCount = clusters.length;

    return [
        {
            label: 'Active Agents',
            value: `${activeAgentsCount}/${settings.maxAgents}`,
            icon: Cpu,
            tooltip: 'Total agents assigned to active mission clusters vs system capacity.',
            color: 'text-emerald-400'
        },
        {
            label: 'Active Clusters',
            value: `${activeClustersCount}/${settings.maxClusters}`,
            icon: Target,
            tooltip: 'Currently deployed mission clusters vs system limit.',
            color: 'text-blue-400'
        },
        {
            label: 'Nodes Online',
            value: isOnline ? agentsCount : 'OFFLINE',
            icon: Activity,
            tooltip: 'Real-time telemetry of neural nodes connected to the Tadpole Engine.',
            color: isOnline ? 'text-cyan-400' : 'text-zinc-600'
        },
        {
            label: 'Max Depth',
            value: settings.maxSwarmDepth,
            icon: Repeat,
            tooltip: 'Maximum recursion depth allowed for autonomous agent delegation.',
            color: 'text-zinc-400'
        },
        {
            label: 'Task Limit',
            value: `${Math.round(settings.maxTaskLength / 1024)}k`,
            icon: Zap,
            tooltip: 'Maximum context length (tokens) per neural inference cycle.',
            color: 'text-amber-400'
        },
        {
            label: 'Base Budget',
            value: `$${settings.defaultBudgetUsd.toFixed(2)}`,
            icon: DollarSign,
            tooltip: 'Default neural credit allocation for new swarm branches.',
            color: 'text-emerald-500'
        }
    ];
};
