/**
 * @docs ARCHITECTURE:Logic
 * @docs OPERATIONS_MANUAL:Telemetry
 * 
 * ### AI Assist Note
 * **Custom Hook**: Aggregates and normalizes swarm telemetry metrics for the Global Header. 
 * Combines settings, workspace cluster data, and engine status into a unified visual schema.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Division by zero if `max_agents` is uninitialized, stale cluster counts if `workspace_store` sync lags, or "OFFLINE" status if `useEngineStatus` heartbeat fails.
 * - **Telemetry Link**: Check `Swarm_Status_Header` usages or search `[use_swarm_metrics]` in component tracing.
 */

import { useMemo } from 'react';
import { use_settings_store } from '../stores/settings_store';
import { use_workspace_store } from '../stores/workspace_store';
import { useEngineStatus } from './use_engine_status';
import { Activity, Cpu, Target, Repeat, Zap, DollarSign, type LucideIcon } from 'lucide-react';
import { i18n } from '../i18n';

/**
 * Interface for localized swarm telemetry metric
 */
export interface SwarmMetric {
    label: string;
    value: string | number;
    icon: LucideIcon;
    tooltip: string;
    color: string;
}

/**
 * useSwarmMetrics
 * Custom hook to aggregate swarm telemetry metrics for the header.
 */
export const useSwarmMetrics = (): SwarmMetric[] => {
    const { settings } = use_settings_store();
    const { clusters } = use_workspace_store();
    const { is_online, agent_count: agents_count } = useEngineStatus();

    const clusters_list = clusters || [];
    const active_agents_count = clusters_list.reduce((acc, c) => acc + (c.collaborators || []).length, 0);
    const active_clusters_count = clusters_list.length;

    return useMemo(() => [
        {
            label: i18n.t('metrics.active_agents'),
            value: `${active_agents_count}/${settings.max_agents}`,
            icon: Cpu,
            tooltip: i18n.t('metrics.active_agents_tooltip'),
            color: 'text-emerald-400'
        },
        {
            label: i18n.t('metrics.active_clusters'),
            value: `${active_clusters_count}/${settings.max_clusters}`,
            icon: Target,
            tooltip: i18n.t('metrics.active_clusters_tooltip'),
            color: 'text-green-400'
        },
        {
            label: i18n.t('metrics.nodes_online'),
            value: is_online ? agents_count : i18n.t('common.offline'),
            icon: Activity,
            tooltip: i18n.t('metrics.nodes_online_tooltip'),
            color: is_online ? 'text-cyan-400' : 'text-zinc-600'
        },
        {
            label: i18n.t('metrics.max_depth'),
            value: settings.max_swarm_depth,
            icon: Repeat,
            tooltip: i18n.t('metrics.max_depth_tooltip'),
            color: 'text-zinc-400'
        },
        {
            label: i18n.t('metrics.task_limit'),
            value: `${Math.round(settings.max_task_length / 1024)}k`,
            icon: Zap,
            tooltip: i18n.t('metrics.task_limit_tooltip'),
            color: 'text-amber-400'
        },
        {
            label: i18n.t('metrics.base_budget'),
            value: `$${settings.default_budget_usd.toFixed(2)}`,
            icon: DollarSign,
            tooltip: i18n.t('metrics.base_budget_tooltip'),
            color: 'text-emerald-500'
        }
    ], [
        active_agents_count, settings.max_agents, 
        active_clusters_count, settings.max_clusters, 
        is_online, agents_count, 
        settings.max_swarm_depth, settings.max_task_length, settings.default_budget_usd
    ]);
};

// Metadata: [use_swarm_metrics]
