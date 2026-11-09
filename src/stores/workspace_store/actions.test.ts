/**
 * @docs ARCHITECTURE:State
 *
 * ### AI Assist Note
 * - Verify workspace handoffs produce distinct identifiers under same-tick creation.
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Tests / workspace actions
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { DEFAULT_CLUSTERS } from './model';
import { use_workspace_store } from '../workspace_store';

describe('workspace actions', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    use_workspace_store.setState({
      clusters: DEFAULT_CLUSTERS.map((cluster) => ({ ...cluster, pending_tasks: [] })),
      team_presets: [],
      active_proposals: {},
      sync_status: {},
    });
  });

  it('assigns distinct IDs to handoffs created in the same millisecond', () => {
    vi.spyOn(Date, 'now').mockReturnValue(1234);
    const store = use_workspace_store.getState();
    const target = store.clusters[0];
    store.receive_handoff('source', target.id, 'first');
    use_workspace_store.getState().receive_handoff('source', target.id, 'second');

    const tasks = use_workspace_store.getState().clusters[0].pending_tasks;
    expect(tasks).toHaveLength(2);
    expect(tasks[0].id).not.toBe(tasks[1].id);
  });
});
