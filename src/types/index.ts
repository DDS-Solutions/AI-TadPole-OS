/**
 * @docs ARCHITECTURE:Types
 * 
 * ### AI Assist Note
 * **Root Type Registry**: Authoritative TypeScript interfaces for the Tadpole OS frontend. 
 * Orchestrates the shape of `Agent`, `Task`, and `Department` models, ensuring 1:1 structural parity with the Rust backend.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: TypeScript compilation error (due to union type mismatch), or runtime object property absence when backend DTOs evolve.
 * - **Telemetry Link**: Search for `interface Agent` or `Agent_Status` in codebase audits to verify schema alignment.
 */


import type { Mission } from './mission';
export type { Mission };
export type { Mission_Cluster } from '../stores/workspace_store';

// Consolidated Agent Contracts (Phase 2 Migration)
import type {
  Agent,
  AgentPatch,
  Agent_Connector_Config,
  Agent_Metadata,
  Agent_Model_Slot_Key,
  Agent_Model_Slot_State,
  Agent_Status,
  Agent_Stt_Engine,
  Agent_Voice_Engine,
  Department
} from '../contracts/agent';

export type {
  Agent,
  AgentPatch,
  Agent_Connector_Config,
  Agent_Metadata,
  Agent_Model_Slot_Key,
  Agent_Model_Slot_State,
  Agent_Status,
  Agent_Stt_Engine,
  Agent_Voice_Engine,
  Department
};

export type Message_Part = 
    | { type: 'text', content: string }
    | { type: 'thought', content: string, status: 'thinking' | 'done' }
    | { type: 'tool', name: string, input: unknown, output?: unknown };






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


// Metadata: [index]

// Metadata: [index]
