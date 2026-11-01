/**
 * @docs ARCHITECTURE:State
 *
 * ### AI Assist Note
 * - Group workspace mutations by domain while preserving the existing store action contracts.
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / workspace actions
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: [WorkspaceStore]
 */

import { workspace_service } from '../../services/workspace_service';
import { MAX_TASK_BRANCHES } from './model';
import type { Workspace_Get, Workspace_Set, Workspace_State } from './model';

export function create_workspace_service_actions(): Pick<
  Workspace_State,
  'load_quotas' | 'create_cluster' | 'refresh_sync_status' | 'update_cluster_budget'
> {
  return {
    // Actions - Delegated to Workspace_Service
    load_quotas: async () => workspace_service.sync_quotas(),
    create_cluster: async (mission) => workspace_service.create_mission_cluster(mission),
    refresh_sync_status: async () => workspace_service.refresh_telemetry(),
    update_cluster_budget: async (id, budget) => workspace_service.update_budget(id, budget),
  };
}

export function create_workspace_preset_actions(
  set: Workspace_Set,
): Pick<Workspace_State, 'add_team_preset' | 'update_team_preset' | 'delete_team_preset'> {
  return {
    add_team_preset: (preset) => {
      set((state) => ({
        team_presets: [...state.team_presets, preset],
      }));
    },

    update_team_preset: (id, updates) => {
      set((state) => ({
        team_presets: state.team_presets.map((p) => (p.id === id ? { ...p, ...updates } : p)),
      }));
    },

    delete_team_preset: (id) => {
      set((state) => ({
        team_presets: state.team_presets.filter((p) => p.id !== id),
      }));
    },

    // Pure State Actions
  };
}

export function create_workspace_cluster_actions(
  set: Workspace_Set,
  get: Workspace_Get,
): Pick<
  Workspace_State,
  | 'assign_agent_to_cluster'
  | 'unassign_agent_from_cluster'
  | 'update_cluster_objective'
  | 'update_cluster_department'
  | 'generate_proposal'
  | 'apply_proposal'
  | 'dismiss_proposal'
  | 'set_alpha_node'
  | 'delete_cluster'
  | 'toggle_cluster_active'
  | 'toggle_mission_analysis'
  | 'toggle_cluster_privacy'
> {
  return {
    assign_agent_to_cluster: (agent_id, cluster_id) => {
      set((state) => ({
        clusters: state.clusters.map((c) =>
          c.id === cluster_id
            ? { ...c, collaborators: [...new Set([...c.collaborators, agent_id])] }
            : c,
        ),
      }));
    },

    unassign_agent_from_cluster: (agent_id, cluster_id) => {
      set((state) => ({
        clusters: state.clusters.map((c) =>
          c.id === cluster_id
            ? {
                ...c,
                collaborators: c.collaborators.filter((id) => id !== agent_id),
                alpha_id: c.alpha_id === agent_id ? undefined : c.alpha_id,
              }
            : c,
        ),
      }));
    },

    update_cluster_objective: (cluster_id, objective) => {
      set((state) => ({
        clusters: state.clusters.map((c) => (c.id === cluster_id ? { ...c, objective } : c)),
      }));
      workspace_service.request_proposal(cluster_id).catch((error: unknown) => {
        console.error('[WorkspaceStore] Proposal generation failed:', error);
      });
    },

    update_cluster_department: (cluster_id, department) => {
      set((state) => ({
        clusters: state.clusters.map((c) => (c.id === cluster_id ? { ...c, department } : c)),
      }));
    },

    generate_proposal: (cluster_id, immediate = false) =>
      workspace_service.request_proposal(cluster_id, immediate),

    apply_proposal: (cluster_id) => {
      set((state) => {
        const next_proposals = { ...state.active_proposals };
        delete next_proposals[cluster_id];
        return { active_proposals: next_proposals };
      });
    },

    dismiss_proposal: (cluster_id) => {
      set((state) => {
        const next_proposals = { ...state.active_proposals };
        delete next_proposals[cluster_id];
        return { active_proposals: next_proposals };
      });
    },

    set_alpha_node: (cluster_id, agent_id) => {
      set((state) => ({
        clusters: state.clusters.map((c) =>
          c.id === cluster_id ? { ...c, alpha_id: agent_id } : c,
        ),
      }));
    },

    delete_cluster: (cluster_id) => {
      set((state) => {
        const next_proposals = { ...state.active_proposals };
        delete next_proposals[cluster_id];
        const next_sync_status = { ...state.sync_status };
        delete next_sync_status[cluster_id];

        return {
          clusters: state.clusters.filter((c) => c.id !== cluster_id),
          active_proposals: next_proposals,
          sync_status: next_sync_status,
        };
      });
    },

    toggle_cluster_active: (cluster_id) => {
      set((state) => ({
        clusters: state.clusters.map((c) =>
          c.id === cluster_id ? { ...c, is_active: !c.is_active } : c,
        ),
      }));
    },

    toggle_mission_analysis: (cluster_id) => {
      set((state) => ({
        clusters: state.clusters.map((c) =>
          c.id === cluster_id ? { ...c, analysis_enabled: !c.analysis_enabled } : c,
        ),
      }));
    },

    toggle_cluster_privacy: async (cluster_id) => {
      const cluster = get().clusters.find((c) => c.id === cluster_id);
      if (!cluster) return;
      const target_mode = !cluster.privacy_mode;
      await workspace_service.update_cluster_privacy(cluster_id, target_mode);
    },
  };
}

let fallback_task_sequence = 0;

function create_task_id(prefix: string): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return prefix + '-' + crypto.randomUUID();
  }
  fallback_task_sequence += 1;
  return (
    prefix +
    '-' +
    Date.now().toString(36) +
    '-' +
    fallback_task_sequence.toString(36) +
    '-' +
    Math.random().toString(36).slice(2, 8)
  );
}

export function create_workspace_branch_actions(
  set: Workspace_Set,
  get: Workspace_Get,
): Pick<
  Workspace_State,
  'add_branch' | 'approve_branch' | 'reject_branch' | 'receive_handoff' | 'get_agent_path'
> {
  return {
    add_branch: (cluster_id, branch) => {
      set((state) => ({
        clusters: state.clusters.map((c) => {
          if (c.id !== cluster_id) return c;
          const next_tasks = [
            ...c.pending_tasks,
            {
              ...branch,
              id: create_task_id('br'),
              status: 'pending' as const,
              timestamp: Date.now(),
            },
          ];
          return {
            ...c,
            pending_tasks:
              next_tasks.length > MAX_TASK_BRANCHES
                ? next_tasks.slice(-MAX_TASK_BRANCHES)
                : next_tasks,
          };
        }),
      }));
    },

    approve_branch: (cluster_id, branch_id) => {
      set((state) => ({
        clusters: state.clusters.map((c) =>
          c.id === cluster_id
            ? {
                ...c,
                pending_tasks: c.pending_tasks.map((t) =>
                  t.id === branch_id ? { ...t, status: 'completed' } : t,
                ),
              }
            : c,
        ),
      }));
    },

    reject_branch: (cluster_id, branch_id) => {
      set((state) => ({
        clusters: state.clusters.map((c) =>
          c.id === cluster_id
            ? {
                ...c,
                pending_tasks: c.pending_tasks.map((t) =>
                  t.id === branch_id ? { ...t, status: 'rejected' } : t,
                ),
              }
            : c,
        ),
      }));
    },

    receive_handoff: (source_cluster_id, target_cluster_id, description) => {
      set((state) => ({
        clusters: state.clusters.map((c) => {
          if (c.id !== target_cluster_id) return c;
          const next_tasks = [
            ...c.pending_tasks,
            {
              id: create_task_id('ho'),
              agent_id: 'System (Handoff)',
              description: `[HANDOFF FROM ${source_cluster_id}] ${description}`,
              target_path: c.path,
              status: 'pending' as const,
              timestamp: Date.now(),
            },
          ];
          return {
            ...c,
            pending_tasks:
              next_tasks.length > MAX_TASK_BRANCHES
                ? next_tasks.slice(-MAX_TASK_BRANCHES)
                : next_tasks,
          };
        }),
      }));
    },

    get_agent_path: (agent_id) => {
      const clusters = get().clusters || [];
      const cluster = clusters.find((c) => (c.collaborators || []).includes(agent_id));
      return cluster ? cluster.path : `/workspaces/agent-silo-${agent_id}`;
    },
  };
}

export function create_workspace_actions(set: Workspace_Set, get: Workspace_Get) {
  return {
    ...create_workspace_service_actions(),
    ...create_workspace_preset_actions(set),
    ...create_workspace_cluster_actions(set, get),
    ...create_workspace_branch_actions(set, get),
  };
}
