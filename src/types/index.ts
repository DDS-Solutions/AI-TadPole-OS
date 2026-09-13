/**
 * @docs ARCHITECTURE:Types
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / index
 * - **Primary Entrypoints**: `Swarm_Node`, `Pulse_Node`, `Pulse_Connection`, `Swarm_Pulse`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export type { Mission } from './mission';
export type { Mission_Cluster } from '../stores/workspace_store';

// Consolidated Agent Contracts (Direct export for 1:1 parity)
export type * from '../contracts/agent';

// GAP-TYPE-02: All variants now carry an optional `status` for consistent
// discriminated union narrowing across consumers.
export type Message_Part = 
    | { type: 'text', content: string, status?: 'complete' | 'streaming' }
    | { type: 'thought', content: string, status: 'thinking' | 'done' }
    | { type: 'tool', name: string, input: unknown, output?: unknown, status?: 'pending' | 'success' | 'error' };






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
export type { 
    Trace_Span, 
    Trace_Node 
} from './tadpoleos';
