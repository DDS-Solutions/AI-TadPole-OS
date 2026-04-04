import type { Tadpole_OS_Model_Config } from './tadpoleos';
import type { Mission } from './mission';
export type { Mission };
export type { Mission_Cluster } from '../stores/workspace_store';

export type Message_Part = 
    | { type: 'text', content: string }
    | { type: 'thought', content: string, status: 'thinking' | 'done' }
    | { type: 'tool', name: string, input: unknown, output?: unknown };

/**
 * Agent
 * Represents an AI Agent within the Tadpole OS ecosystem.
 * Refactored for strict snake_case compliance for backend parity.
 */
export interface Agent {
  /** Unique identifier for the agent */
  id: string;
  /** Display name of the agent */
  name: string;
  /**
   * functional_role
   * Functional role of the agent (e.g., 'CEO', 'Backend Dev').
   */
  role: string;
  department: Department;
  /** status - Current operational status of the agent */
  status: Agent_Status;
  /** tokens_used - Total tokens consumed by this agent in the current session */
  tokens_used: number;
  /** current_task - Description of the task currently being executed, if any */
  current_task?: string;
  /** model - Primary Model ID */
  model: string;
  /** model2 - Secondary Model ID */
  model2?: string;
  /** model3 - Tertiary Model ID */
  model3?: string;
  /** model_config - TadpoleOS Model Configuration (Strict Schema) */
  model_config?: Tadpole_OS_Model_Config;
  /** model_config2 - Secondary Model Configuration (Strict Schema) */
  model_config2?: Tadpole_OS_Model_Config;
  /** model_config3 - Tertiary Model Configuration (Strict Schema) */
  model_config3?: Tadpole_OS_Model_Config;
  /** reports_to - ID of the agent this agent reports to (for org chart) */
  reports_to?: string;
  /** skills - List of available atomic skills */
  skills?: string[];
  /** workflows - List of available workflows */
  workflows?: string[];
  /** mcp_tools - List of available external MCP tools */
  mcp_tools?: string[];
  /** active_workflow - Currently running workflow (if any) */
  active_workflow?: string;
  /** workspace_path - TadpoleOS Workspace Path */
  workspace_path?: string;
  /** active_mission - Currently running mission */
  active_mission?: Mission;
  active_model_slot?: 1 | 2 | 3;
  /** theme_color - UI Extension: Custom theme color (HEX) */
  theme_color?: string;
  /** budget_usd - Primary budget limit for this agent (USD) */
  budget_usd?: number;
  /** cost_usd - Current total cost accrued by this agent (USD) */
  cost_usd?: number;
  /** requires_oversight - Whether the agent requires human oversight */
  requires_oversight?: boolean;
  /** voice_id - Voice identifier for TTS */
  voice_id?: string;
  /** voice_engine - Voice engine to use ('browser' | 'openai' | 'groq' | 'piper' | 'gemini-live') */
  voice_engine?: 'browser' | 'openai' | 'groq' | 'piper' | 'gemini-live';
  /** stt_engine - Speech-to-text engine to use ('groq' | 'whisper') */
  stt_engine?: 'groq' | 'whisper';
  /** valence - Emotional valence indicator for UI pulsing highlights (-1.0 to 1.0) */
  valence?: number;
  /** last_pulse - ISO timestamp of last pulse/activity */
  last_pulse?: string | null;
  /** is_loading - UI state: whether the agent is currently loading/initializing */
  is_loading?: boolean;
  /** created_at - ISO timestamp of creation (Authoritative from backend) */
  created_at?: string;
  /** input_tokens - Tokens consumed in the last turn prompt */
  input_tokens?: number;
  /** output_tokens - Tokens consumed in the last turn completion */
  output_tokens?: number;
  /** failure_count - Number of consecutive mission failures */
  failure_count?: number;
  /** last_failure_at - ISO timestamp of the last mission failure */
  last_failure_at?: string;
  /** category - Categorization for UI filtering ('user' | 'ai') */
  category: string;
  /** connector_configs - Background data sync sources */
  connector_configs?: { type: string; uri: string }[];
}

/**
 * Task
 * Represents a unit of work assigned to an agent.
 */
export interface Task {
  /** Unique identifier for the task */
  id: string;
  /** Title or short description of the task */
  title: string;
  /** assigned_to - ID of the agent assigned to this task */
  assigned_to: string;
  /** status - Current execution status of the task */
  status: 'pending' | 'in-progress' | 'completed' | 'failed';
  /** priority - Priority level of the task */
  priority: 'low' | 'medium' | 'high';
  /** created_at - ISO timestamp of creation */
  created_at: string;
  /** logs - History of activity logs related to this task */
  logs: string[];
}

/**
 * Agent_Config
 * Configuration updates for an agent.
 */
export interface Agent_Config {
  name?: string;
  role?: string;
  department?: Department;
  description?: string;
  model_id?: string;
  provider?: string;
  api_key?: string;
  base_url?: string;
  system_prompt?: string;
  temperature?: number;
  skills?: string[];
  workflows?: string[];
  mcp_tools?: string[];
  theme_color?: string;
  budget_usd?: number;
  external_id?: string;
  active_model_slot?: 1 | 2 | 3;
  model_config2?: Tadpole_OS_Model_Config;
  model_config3?: Tadpole_OS_Model_Config;
  voice_id?: string;
  voice_engine?: 'browser' | 'openai' | 'groq' | 'piper' | 'gemini-live';
  stt_engine?: 'groq' | 'whisper';
  failure_count?: number;
  last_failure_at?: string;
  category?: string;
  requires_oversight?: boolean;
  connector_configs?: { type: string; uri: string }[];
  created_at?: string;
  last_pulse?: string | null;
  input_tokens?: number;
  output_tokens?: number;
}

/**
 * Task_Payload
 * Payload for sending a command/task to an agent.
 */
export interface Task_Payload {
  message: string;
  cluster_id?: string;
  department?: string;
  provider?: string;
  model_id?: string;
  api_key?: string;
  base_url?: string;
  rpm?: number;
  tpm?: number;
  rpd?: number;
  tpd?: number;
  budget_usd?: number;
  external_id?: string;
  safe_mode?: boolean;
  analysis?: boolean;
  swarm_depth?: number;
  swarm_lineage?: string[];
  recent_findings?: string;
  traceparent?: string;
}

export type Department = 'Executive' | 'Engineering' | 'Marketing' | 'Sales' | 'Product' | 'Operations' | 'Quality Assurance' | 'Design' | 'Research' | 'Support' | 'Intelligence' | 'Finance' | 'Growth' | 'Success';

/** Agent_Status - Operational status of an agent. Consistent with TadpoleOS core statuses. */
export type Agent_Status = 'idle' | 'working' | 'active' | 'thinking' | 'speaking' | 'coding' | 'paused' | 'offline' | 'online' | 'busy' | 'suspended';

/**
 * Swarm_Node
 * Represents a Bunker node in the Swarm network.
 */
export interface Swarm_Node {
  id: string;
  name: string;
  address: string;
  status: 'online' | 'offline' | 'deploying';
  last_seen: string;
  metadata: Record<string, string>;
  /** running_agents - IDs of agents currently running on this node */
  running_agents?: string[];
}
/**
 * Swarm_Pulse
 * High-speed binary telemetry for real-time swarm visualization.
 * Mirrored from server-rs/src/telemetry/pulse_types.rs for 1:1 parity.
 */
export interface Pulse_Node {
  id: string;
  x: number;
  y: number;
  status: number; // 0: idle, 1: busy, 2: error, 3: degraded
  battery: number;
  signal: number;
  progress: number;
}

export interface Pulse_Connection {
  source: string;
  target: string;
}

export interface Swarm_Pulse {
  timestamp: number;
  nodes: Pulse_Node[];
  edges: Pulse_Connection[];
}
