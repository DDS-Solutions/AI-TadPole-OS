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
 */

export type { Mission } from './mission';
export type { Mission_Cluster } from '../stores/workspace_store';

// Consolidated Agent Contracts (Direct export for 1:1 parity)
export type * from '../contracts/agent';

// GAP-TYPE-02: All variants now carry an optional `status` for consistent
// discriminated union narrowing across consumers.
export interface SocraticEnvelope {
    status: string;
    scope_contract: {
        target_paths?: string[];
        blast_radius_level?: number;
        mutation_allowed?: boolean;
        database_mutation_allowed?: boolean;
        mission_vector?: string;
        [key: string]: unknown;
    };
    performance_threshold: {
        budget_usd_cap?: number;
        max_swarm_depth?: number;
        target_turn_latency_ms?: number;
        active_model_slot?: number;
        timeout_seconds?: number;
        [key: string]: unknown;
    };
    architecture_mode: {
        mode_name?: string;
        privacy_shield_enforced?: boolean;
        [key: string]: unknown;
    };
    pre_cleared_failure_modes: string[];
}

export type Message_Part = 
    | { type: 'text', content: string, status?: 'complete' | 'streaming' }
    | { type: 'thought', content: string, status: 'thinking' | 'done' }
    | { type: 'tool', name: string, input: unknown, output?: unknown, status?: 'pending' | 'success' | 'error' }
    | { 
        type: 'question', 
        question: string, 
        options: string[], 
        context?: string, 
        question_id?: string, 
        selected_option?: string, 
        status?: 'pending' | 'answered',
        /** approve_option: the option value that maps to the 'approved' decision.
         *  If unset, defaults to options[0]. Any other selected option maps to 'rejected'. */
        approve_option?: string
      }
    | {
        type: 'openui',
        /** The DSL payload describing the UI to render */
        dsl: OpenUI_DSL,
        status?: 'rendering' | 'complete'
      }
    | {
        type: 'socratic',
        envelope: SocraticEnvelope,
        status?: 'complete' | 'evaluating'
      };

// ── OpenUI DSL Types ─────────────────────────────────────────

export type OpenUI_DSL = OpenUI_KPI_Card | OpenUI_Bar_Chart | OpenUI_Table | OpenUI_Layout;

export interface OpenUI_KPI_Card {
    kind: 'kpi_card';
    title: string;
    value: string | number;
    unit?: string;
    delta?: number;
    delta_label?: string;
    icon?: string;
}

export interface OpenUI_Bar_Chart {
    kind: 'bar_chart';
    title: string;
    labels: string[];
    datasets: {
        label: string;
        data: number[];
        color?: string;
    }[];
}

export interface OpenUI_Table {
    kind: 'table';
    title?: string;
    columns: { key: string; label: string; align?: 'left' | 'center' | 'right' }[];
    rows: Record<string, string | number | boolean>[];
    sortable?: boolean;
}

export interface OpenUI_Layout {
    kind: 'layout';
    direction: 'row' | 'column';
    children: OpenUI_DSL[];
}







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


export type { 
    Trace_Span, 
    Trace_Node,
    Span_Error_Details
} from './tadpoleos';
export { get_span_error_details } from './tadpoleos';
