/**
 * @docs ARCHITECTURE:Logic
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / use_agent_metrics
 * - **Primary Entrypoints**: `useAgentMetrics`, `UseAgentMetricsOptions`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useMemo } from 'react';
import type { Agent } from '../types';

export interface UseAgentMetricsOptions {
  agents_list: Agent[];
  assigned_agent_ids: Set<string>;
}

export function useAgentMetrics({ agents_list, assigned_agent_ids }: UseAgentMetricsOptions) {
  const safe_agents = useMemo(
    () => (Array.isArray(agents_list) ? agents_list : []),
    [agents_list]
  );

  const active_agents = useMemo(
    () =>
      safe_agents.filter(
        (a: Agent) =>
          ['active', 'thinking', 'coding', 'speaking'].includes(a.status) &&
          assigned_agent_ids.has(a.id)
      ).length,
    [safe_agents, assigned_agent_ids]
  );

  const online_count = useMemo(
    () => safe_agents.filter((a: Agent) => a.status !== 'offline').length,
    [safe_agents]
  );

  const total_cost = useMemo(
    () => safe_agents.reduce((acc: number, curr: Agent) => acc + (curr.cost_usd || 0), 0),
    [safe_agents]
  );

  const total_budget = useMemo(
    () => safe_agents.reduce((acc: number, curr: Agent) => acc + (curr.budget_usd || 0), 0),
    [safe_agents]
  );

  const budget_util = total_budget > 0 ? (total_cost / total_budget) * 100 : 0;

  const total_tokens = useMemo(
    () => safe_agents.reduce((acc: number, curr: Agent) => acc + (curr.tokens_used || 0), 0),
    [safe_agents]
  );

  const total_input_tokens = useMemo(
    () => safe_agents.reduce((acc: number, curr: Agent) => acc + (curr.input_tokens || 0), 0),
    [safe_agents]
  );

  const total_output_tokens = useMemo(
    () => safe_agents.reduce((acc: number, curr: Agent) => acc + (curr.output_tokens || 0), 0),
    [safe_agents]
  );

  return {
    active_agents,
    online_count,
    total_cost,
    total_budget,
    budget_util,
    total_tokens,
    total_input_tokens,
    total_output_tokens,
  };
}
