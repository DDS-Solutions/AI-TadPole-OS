/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / types
 * - **Primary Entrypoints**: `Command_Context`, `Command_Result`, `Command_Definition`, `Command_Handler_Fn`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
