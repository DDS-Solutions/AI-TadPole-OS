/**
 * @docs ARCHITECTURE:State
 *
 * ### AI Assist Note
 * - Keep persisted workspace state composition separate from contracts, defaults, and action groups.
 *
 * ### Persistence Lifecycle
 * ```mermaid
 * stateDiagram-v2
 *     [*] --> InMemory: create_workspace_state
 *     InMemory --> StoredSnapshot: persist after store updates
 *     StoredSnapshot --> InMemory: hydrate on startup or migrate stored version
 * ```
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / workspace_store
 * - **Primary Entrypoints**: use_workspace_store
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { create } from 'zustand';
import type { StateCreator } from 'zustand';
import { persist } from 'zustand/middleware';
import { create_workspace_actions } from './workspace_store/actions';
import { DEFAULT_CLUSTERS, DEFAULT_TEAM_PRESETS } from './workspace_store/model';
import type { Workspace_State } from './workspace_store/model';

export type {
  Mission_Cluster,
  Task_Branch,
  Team_Cluster_Preset,
  Swarm_Proposal,
  Workspace_State,
} from './workspace_store/model';

const create_workspace_state: StateCreator<Workspace_State> = (set, get) => ({
  clusters: DEFAULT_CLUSTERS,
  team_presets: DEFAULT_TEAM_PRESETS,
  active_proposals: {},
  sync_status: {},
  ...create_workspace_actions(set, get),
});

export const use_workspace_store = create<Workspace_State>()(
  persist(create_workspace_state, {
    name: 'tadpole-workspaces-v4',
    version: 2,
    migrate: (persisted_state: unknown, version: number) => {
      if (version < 2) {
        return {
          clusters: DEFAULT_CLUSTERS,
          active_proposals: {},
          sync_status: {},
        };
      }
      return persisted_state as Record<string, unknown>;
    },
  }),
);


import { register_store_for_test_reset } from '../utils/store_registry';

register_store_for_test_reset(() => {
    use_workspace_store.setState(use_workspace_store.getInitialState(), true);
});
