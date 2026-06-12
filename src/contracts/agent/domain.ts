/**
 * @docs ARCHITECTURE:Contracts
 * 
 * ### AI Assist Note
 * **Domain Layer**: Authoritative normalized model for AI Agents.
 * Standardized on snake_case. This is the primary model used by stores and UI.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Field mismatches during store hydration or API serialization drift.
 * - **Telemetry Link**: Not tracked (Static Domain Model).
 */

import type { ModelConfigDto } from './wire';
import type { 
    Agent_Status, 
    Department, 
    Agent_Metadata, 
    Agent_Connector_Config,
    Agent_Voice_Engine,
    Agent_Stt_Engine
} from './shared';

export interface Agent {
    id: string;
    name: string;
    role: string;
    department: Department;
    description?: string;
    status: Agent_Status;
    tokens_used?: number;
    current_task?: string;
    model: string; 
    model_2?: string;
    model_3?: string;
    model_config?: ModelConfigDto;
    model_config2?: ModelConfigDto;
    model_config3?: ModelConfigDto;
    active_model_slot?: 1 | 2 | 3;
    skills?: string[];
    workflows?: string[];
    mcp_tools?: string[];
    theme_color?: string;
    budget_usd?: number;
    cost_usd?: number;
    requires_oversight?: boolean;
    voice_id?: string;
    voice_engine?: Agent_Voice_Engine;
    stt_engine?: Agent_Stt_Engine;
    last_pulse?: string | null;
    created_at?: string;
    input_tokens?: number;
    output_tokens?: number;
    failure_count?: number;
    last_failure_at?: string;
    category: string; 
    connector_configs?: Agent_Connector_Config[];
    metadata?: Agent_Metadata;
    current_reasoning_turn?: number;
    reasoning_depth?: number;
    workspace_path?: string;
    _local_timestamp?: number;
    active_mission?: { 
        id: string;
        objective?: string;
        constraints?: string[];
        priority?: string;
        is_degraded?: boolean;
    };
    valence?: number;
    reports_to?: string;
}

export interface Role_Definition {
    id: string;
    name: string;
    department: Department;
    description: string;
    skills: string[];
    workflows: string[];
    mcp_tools: string[];
    requires_oversight: boolean;
    model_id: string;
    created_at?: string;
}

export interface Agent_Memory_Entry {
    id: string;
    text: string;
    mission_id: string;
    timestamp: number;
    metadata?: Record<string, unknown>;
}

/**
 * Task
...


/**
 * Task
 * Represents a unit of work assigned to an agent.
 */
export interface Task {
  id: string;
  title: string;
  assigned_to: string;
  status: 'pending' | 'in-progress' | 'completed' | 'failed';
  priority: 'low' | 'medium' | 'high';
  created_at: string;
  logs: string[];
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

export type AgentPatch = Partial<Agent>;

// Metadata: [domain]

// Metadata: [domain]
