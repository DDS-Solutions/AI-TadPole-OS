/**
 * @docs ARCHITECTURE:State
 *
 * ### AI Assist Note
 * - Keep workspace contracts and seeded state aligned with persisted and UI-facing data.
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / workspace model
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import type { StateCreator } from 'zustand';

export interface Mission_Cluster {
  /** Unique identifier for the mission cluster (e.g. 'cl-command'). */
  id: string;
  /** Human-readable name of the workspace sector. */
  name: string;
  /** The organizational department governing this cluster's AI nodes. */
  department:
    | 'Executive'
    | 'Engineering'
    | 'Product'
    | 'Sales'
    | 'Operations'
    | 'Quality Assurance'
    | 'Design'
    | 'Research'
    | 'Support'
    | 'Marketing'
    | 'Intelligence'
    | 'Finance'
    | 'Growth'
    | 'Success'
    | 'Swarm Core'
    | (string & {});
  path: string;
  collaborators: string[]; // Agent IDs
  alpha_id?: string; // The leader of the cluster
  objective?: string; // High-level mission objective
  theme: 'cyan' | 'zinc' | 'amber' | 'blue';
  pending_tasks: Task_Branch[];
  is_active?: boolean;
  budget_usd?: number;
  cost_usd?: number;
  analysis_enabled?: boolean;
  privacy_mode?: boolean;
  phase?: 'waiting_for_model' | 'streaming_reasoning' | 'tool_execution' | 'completed' | 'failed';
  token_burn_rate?: number;
  is_team?: boolean;
  team_badge?: string;
}

export interface Team_Cluster_Preset {
  id: string;
  name: string;
  description: string;
  department: Mission_Cluster['department'];
  theme: Mission_Cluster['theme'];
  default_budget_usd: number;
  badge_label: string;
  collaborators: string[]; // Agent IDs
}

export interface Swarm_Proposal {
  cluster_id: string;
  reasoning: string;
  changes: {
    agent_id: string;
    proposed_role?: string;
    proposed_model?: string;
    added_skills?: string[];
    added_workflows?: string[];
  }[];
  timestamp: number;
}

export interface Task_Branch {
  id: string;
  agent_id: string;
  description: string;
  target_path: string;
  status: 'pending' | 'merging' | 'completed' | 'rejected';
  phase?: 'waiting_for_model' | 'streaming_reasoning' | 'tool_execution' | 'completed' | 'failed';
  permission_choice?: 'approve' | 'deny' | 'approve_once' | 'always_allow';
  timestamp: number;
}

export interface Workspace_State {
  clusters: Mission_Cluster[];
  team_presets: Team_Cluster_Preset[];
  active_proposals: Record<string, Swarm_Proposal>; // cluster_id -> proposal

  // Actions
  load_quotas: () => Promise<void>;
  create_cluster: (mission: Partial<Mission_Cluster>) => Promise<void>;
  assign_agent_to_cluster: (agent_id: string, cluster_id: string) => void;
  unassign_agent_from_cluster: (agent_id: string, cluster_id: string) => void;
  update_cluster_objective: (cluster_id: string, objective: string) => void;
  update_cluster_department: (
    cluster_id: string,
    department: Mission_Cluster['department'],
  ) => void;
  update_cluster_budget: (cluster_id: string, budget: number) => Promise<void>;
  add_team_preset: (preset: Team_Cluster_Preset) => void;
  update_team_preset: (id: string, updates: Partial<Team_Cluster_Preset>) => void;
  delete_team_preset: (id: string) => void;
  generate_proposal: (cluster_id: string, immediate?: boolean) => Promise<Swarm_Proposal | null>;
  apply_proposal: (cluster_id: string) => void;
  dismiss_proposal: (cluster_id: string) => void;
  set_alpha_node: (cluster_id: string, agent_id: string) => void;
  delete_cluster: (cluster_id: string) => void;
  toggle_cluster_active: (cluster_id: string) => void;
  toggle_mission_analysis: (cluster_id: string) => void;
  toggle_cluster_privacy: (cluster_id: string) => Promise<void>;
  add_branch: (
    cluster_id: string,
    branch: Omit<Task_Branch, 'id' | 'status' | 'timestamp'>,
  ) => void;
  approve_branch: (cluster_id: string, branch_id: string) => void;
  reject_branch: (cluster_id: string, branch_id: string) => void;
  receive_handoff: (
    source_cluster_id: string,
    target_cluster_id: string,
    description: string,
  ) => void;

  // Internal path calculation
  get_agent_path: (agent_id: string) => string;

  // Sync Telemetry
  sync_status: Record<
    string,
    {
      status: string;
      last_sync_at: string | null;
      file_count: number;
      total_bytes: number;
      detected_environments?: string[];
      mounted_okf_nodes?: Array<{
        id: string;
        title: string;
        concept_type: string;
        security_tier?: string;
        parent_id?: string;
      }>;
      okf_validation?: {
        status: 'nominal' | 'warning' | 'critical';
        message?: string;
      };
    }
  >;
  refresh_sync_status: () => Promise<void>;
}

export type Workspace_Set = Parameters<StateCreator<Workspace_State>>[0];
export type Workspace_Get = Parameters<StateCreator<Workspace_State>>[1];

export const MAX_TASK_BRANCHES = 50;

export const DEFAULT_TEAM_PRESETS: Team_Cluster_Preset[] = [
  {
    id: 'preset-sec-audit',
    name: 'Security & Audit Cluster',
    description:
      'Multi-role unit specialized in vulnerability scans, code reviews, and threat analysis.',
    department: 'Quality Assurance',
    theme: 'amber',
    default_budget_usd: 1500,
    badge_label: 'SEC-AUDIT',
    collaborators: ['12', '26'],
  },
  {
    id: 'preset-eng-swarm',
    name: 'Full-Stack Engineering Swarm',
    description:
      'High-velocity development cluster with architectural oversight and automated refactoring.',
    department: 'Engineering',
    theme: 'blue',
    default_budget_usd: 2500,
    badge_label: 'ENG-SWARM',
    collaborators: ['3', '8', '13'],
  },
  {
    id: 'preset-research-pub',
    name: 'Research & Intelligence Cluster',
    description:
      'Data ingestion and documentation group for knowledge synthesis and repository analytics.',
    department: 'Research',
    theme: 'cyan',
    default_budget_usd: 1000,
    badge_label: 'RESEARCH-PUB',
    collaborators: ['16', '17'],
  },
];

export const DEFAULT_CLUSTERS: Mission_Cluster[] = [
  {
    id: 'cl-command',
    name: 'Strategic Command',
    department: 'Executive',
    path: '/workspaces/strategic-command',
    collaborators: ['1', '2'],
    alpha_id: '1',
    objective: 'Global swarm oversight and strategic mission planning.',
    theme: 'blue',
    pending_tasks: [],
    is_active: true,
    is_team: true,
    team_badge: 'STRAT-CMD',
  },
  {
    id: 'cl-chain-a',
    name: 'Strategic Ops (Chain A)',
    department: 'Operations',
    path: '/workspaces/strategic-ops',
    collaborators: ['3', '4', '5', 'alpha'],
    alpha_id: '3',
    objective: 'Optimize swarm coordination and strategic resource allocation.',
    theme: 'cyan',
    pending_tasks: [],
    is_active: false,
    is_team: true,
    team_badge: 'SWARM-OPS',
  },
  {
    id: 'cl-chain-b',
    name: 'Core Intelligence (Chain B)',
    department: 'Engineering',
    path: '/workspaces/core-intelligence',
    collaborators: ['7', 'Tadpole_OS_Specialist', 'ResearcherAgent', 'GitHubResearcher'],
    alpha_id: '7',
    objective: 'Enhance neural processing efficiency and knowledge synthesis.',
    theme: 'zinc',
    pending_tasks: [],
    is_team: true,
    team_badge: 'CORE-INT',
  },
  {
    id: 'cl-chain-c',
    name: 'Applied Growth (Chain C)',
    department: 'Product',
    path: '/workspaces/applied-growth',
    collaborators: ['99', '23', '26', 'Risk_Analyzer'],
    alpha_id: '99',
    objective: 'Iterate on user-facing features and scale operational impact.',
    theme: 'amber',
    pending_tasks: [],
    is_team: true,
    team_badge: 'APPLIED-DEV',
  },
];
