import type { TadpoleOSModelConfig } from './tadpoleos';
import type { Mission } from './mission';
export type { Mission };
export type { MissionCluster } from '../stores/workspaceStore';

export type MessagePart = 
    | { type: 'text', content: string }
    | { type: 'thought', content: string, status: 'thinking' | 'done' }
    | { type: 'tool', name: string, input: unknown, output?: unknown };

/**
 * Represents an AI Agent within the Tadpole OS ecosystem.
 * Agents are the primary entities that perform tasks and interact with the system.
 */
export interface Agent {
  /** Unique identifier for the agent */
  id: string;
  /** Display name of the agent */
  name: string;
  /**
   * Functional role of the agent (e.g., 'CEO', 'Backend Dev').
   * Used for visual distinction and grouping.
   */
  role: string;
  department: Department;
  /** Current operational status of the agent (Tadpole extended status) */
  status: AgentStatus;
  /** Total tokens consumed by this agent in the current session */
  tokensUsed: number;
  /** Description of the task currently being executed, if any */
  currentTask?: string;
  /** Primary Model ID (Legacy string for UI) */
  model: string;
  /** Secondary Model ID (Legacy) */
  model2?: string;
  /** Tertiary Model ID (Legacy) */
  model3?: string;
  /** TadpoleOS Model Configuration (Strict Schema) */
  modelConfig?: TadpoleOSModelConfig;
  /** Secondary Model Configuration (Strict Schema) */
  modelConfig2?: TadpoleOSModelConfig;
  /** Tertiary Model Configuration (Strict Schema) */
  modelConfig3?: TadpoleOSModelConfig;
  /** ID of the agent this agent reports to (for org chart) */
  reportsTo?: string;
  /** List of available atomic skills (e.g., 'Search', 'Read File') */
  skills?: string[];
  /** List of available workflows (e.g., 'Deploy', 'Audit') */
  workflows?: string[];
  /** List of available external MCP tools */
  mcpTools?: string[];
  /** Currently running workflow (if any) */
  activeWorkflow?: string;
  /** TadpoleOS Workspace Path */
  workspacePath?: string;
  /** Currently running mission (Option C: Workspace-centric) */
  activeMission?: Mission;
  activeModelSlot?: 1 | 2 | 3;
  /** UI Extension: Custom theme color (HEX) */
  themeColor?: string;
  /** Primary budget limit for this agent (USD) */
  budgetUsd?: number;
  /** Current total cost accrued by this agent (USD) */
  costUsd?: number;
  /** Whether the agent requires human oversight for high-risk actions */
  requiresOversight?: boolean;
  /** Voice identifier for TTS (e.g., 'alloy', 'echo', or browser-specific name) */
  voiceId?: string;
  /** Voice engine to use ('browser' | 'openai' | 'groq' | 'piper' | 'gemini-live') */
  voiceEngine?: 'browser' | 'openai' | 'groq' | 'piper' | 'gemini-live';
  /** Speech-to-text engine to use ('groq' | 'whisper') */
  sttEngine?: 'groq' | 'whisper';
  /** Emotional valence indicator for UI pulsing highlights (-1.0 to 1.0) */
  valence?: number;
  /** ISO timestamp of last pulse/activity */
  lastPulse?: string | null;
  /** UI state: whether the agent is currently loading/initializing */
  isLoading?: boolean;
  /** ISO timestamp of creation (Authoritative from backend) */
  createdAt?: string;
  /** Tokens consumed in the last turn prompt */
  inputTokens?: number;
  /** Tokens consumed in the last turn completion */
  outputTokens?: number;
  /** Number of consecutive mission failures */
  failureCount?: number;
  /** ISO timestamp of the last mission failure */
  lastFailureAt?: string;
  /** Categorization for UI filtering ('user' | 'ai') */
  category: string;
  /** Background data sync sources (Phase 2) */
  connectorConfigs?: { type: string; uri: string }[];
}

/**
 * Represents a unit of work assigned to an agent.
 */
export interface Task {
  /** Unique identifier for the task */
  id: string;
  /** Title or short description of the task */
  title: string;
  /** ID of the agent assigned to this task */
  assignedTo: string;
  /** Current execution status of the task */
  status: 'pending' | 'in-progress' | 'completed' | 'failed';
  /** Priority level of the task */
  priority: 'low' | 'medium' | 'high';
  /** ISO timestamp of creation */
  createdAt: string;
  /** History of activity logs related to this task */
  logs: string[];
}

/**
 * Configuration updates for an agent.
 */
export interface AgentConfig {
  name?: string;
  role?: string;
  department?: Department;
  description?: string;
  modelId?: string;
  provider?: string;
  apiKey?: string;
  baseUrl?: string;
  systemPrompt?: string;
  temperature?: number;
  skills?: string[];
  workflows?: string[];
  mcpTools?: string[];
  themeColor?: string;
  budgetUsd?: number;
  externalId?: string;
  activeModelSlot?: 1 | 2 | 3;
  modelConfig2?: TadpoleOSModelConfig;
  modelConfig3?: TadpoleOSModelConfig;
  voiceId?: string;
  voiceEngine?: 'browser' | 'openai' | 'groq' | 'piper' | 'gemini-live';
  sttEngine?: 'groq' | 'whisper';
  failureCount?: number;
  lastFailureAt?: string;
  category?: string;
  requiresOversight?: boolean;
  connectorConfigs?: { type: string; uri: string }[];
  createdAt?: string;
  lastPulse?: string | null;
  inputTokens?: number;
  outputTokens?: number;
}

/**
 * Payload for sending a command/task to an agent.
 */
export interface TaskPayload {
  message: string;
  clusterId?: string;
  department?: string;
  provider?: string;
  modelId?: string;
  apiKey?: string;
  baseUrl?: string;
  rpm?: number;
  tpm?: number;
  rpd?: number;
  tpd?: number;
  budgetUsd?: number;
  externalId?: string;
  safeMode?: boolean;
  analysis?: boolean;
  swarmDepth?: number;
  swarmLineage?: string[];
  recentFindings?: string;
  traceparent?: string;
}

export type Department = 'Executive' | 'Engineering' | 'Marketing' | 'Sales' | 'Product' | 'Operations' | 'Quality Assurance' | 'Design' | 'Research' | 'Support';

/** Operational status of an agent. Consistent with TadpoleOS core statuses. */
export type AgentStatus = 'idle' | 'working' | 'active' | 'thinking' | 'speaking' | 'coding' | 'paused' | 'offline' | 'online' | 'busy' | 'suspended';

/**
 * Represents a Bunker node in the Swarm network.
 */
export interface SwarmNode {
  id: string;
  name: string;
  address: string;
  status: 'online' | 'offline' | 'deploying';
  lastSeen: string;
  metadata: Record<string, string>;
  /** IDs of agents currently running on this node */
  runningAgents?: string[];
}

