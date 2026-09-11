/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / use_swarm_metrics.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useSwarmMetrics } from './use_swarm_metrics';
import { use_settings_store } from '../stores/settings_store';
import { use_workspace_store } from '../stores/workspace_store';
import { i18n } from '../i18n';

vi.mock('./use_engine_status', () => ({
    useEngineStatus: vi.fn().mockReturnValue({
        is_online: true,
        agent_count: 5
    })
}));

describe('useSwarmMetrics', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        use_settings_store.setState({
            settings: {
                max_agents: 10,
                max_clusters: 4,
                max_swarm_depth: 3,
                max_task_length: 4096,
                default_budget_usd: 1.5
            } as any
        });
        use_workspace_store.setState({
            clusters: [
                { id: 'c1', name: 'Cluster 1', is_active: true, collaborators: ['a1', 'a2'], alpha_id: 'a1' } as any,
                { id: 'c2', name: 'Cluster 2', is_active: false, collaborators: ['a3'], alpha_id: 'a4' } as any
            ]
        });
    });

    it('aggregates unique active agents and active clusters correctly', () => {
        const { result } = renderHook(() => useSwarmMetrics());
        const metrics = result.current;

        // Unique agents in all clusters: a1, a2, a3, a4 = 4
        const activeAgentsMetric = metrics.find(m => m.label === i18n.t('metrics.active_agents'));
        expect(activeAgentsMetric?.value).toBe('4/10');

        // Truly active clusters (is_active === true): c1 only = 1
        const activeClustersMetric = metrics.find(m => m.label === i18n.t('metrics.active_clusters'));
        expect(activeClustersMetric?.value).toBe('1/4');

        // Budget metric
        const budgetMetric = metrics.find(m => m.label === i18n.t('metrics.base_budget'));
        expect(budgetMetric?.value).toBe('$1.50');
    });

    it('handles empty cluster and settings states gracefully', () => {
        use_workspace_store.setState({ clusters: [] });
        use_settings_store.setState({
            settings: {
                max_agents: 5,
                max_clusters: 2,
                max_swarm_depth: 1,
                max_task_length: 1024,
                default_budget_usd: 0
            } as any
        });

        const { result } = renderHook(() => useSwarmMetrics());
        const metrics = result.current;

        const activeAgentsMetric = metrics.find(m => m.label === i18n.t('metrics.active_agents'));
        expect(activeAgentsMetric?.value).toBe('0/5');

        const activeClustersMetric = metrics.find(m => m.label === i18n.t('metrics.active_clusters'));
        expect(activeClustersMetric?.value).toBe('0/2');
    });
});
