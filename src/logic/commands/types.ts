/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:Logic:Commands**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[types]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:Logic:Commands
 */

import type { Agent } from '../../types';

export interface Command_Context {
    parts: string[];
    args: string[];
    agents: Agent[];
    is_safe_mode: boolean;
    active_scope: 'agent' | 'cluster' | 'swarm';
    target_node?: string;
    telemetry_source: string;
}

export interface Command_Result {
    should_clear_logs: boolean;
    handled: boolean;
}

export type Command_Handler_Fn = (ctx: Command_Context) => Promise<Command_Result>;

export interface Command_Definition {
    command: string;
    description: string;
    handler: Command_Handler_Fn;
}

// Metadata: [types]
